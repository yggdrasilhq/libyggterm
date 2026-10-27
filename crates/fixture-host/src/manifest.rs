// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The SCENARIO manifest (`scenario.host.json`, beside the scenario): the
//! DECLARED executable behavior of the falsifiable unit. Portable across
//! arms — an emd component page (the emd arm in `emd-page-host`) and a
//! plain fixture app (the miniapp pattern, mounted through the runner
//! command contract in [`crate::plain`]) carry the SAME manifest model.
//!
//! Structure (chain 0029 Q2, the minimal fixture contract):
//! scenario identity · deterministic inputs (declared fixture + mutable
//! keys) · reset (restores EXACTLY the declared initial) · readiness
//! through the arm's mounted scene · failure (mask digests with reasons,
//! declared plants) · revision (scenario revision + pinned dependencies)
//! · launch command (how the plain arm mounts the fixture app).
//!
//! Schema (contract rev = [`MANIFEST_CONTRACT_REV`], currently 1):
//!
//! ```json
//! {
//!   "contract_rev": 1,
//!   "kind": "plain",
//!   "launch": "corpus/fixture-miniapp.sh --fixture-server",
//!   "revision": "2026-10-27-r1",
//!   "dependencies": {
//!     "implementation": [{ "crate": "fixture-miniapp", "revision": "script:1" }],
//!     "design": { "profile": "t3-base", "digest": "sha256:<hex>" }
//!   },
//!   "design":   { "profiles": { "t3-base": "sha256:<hex>" } },
//!   "fixture":  { "values": { "seed": 7 }, "mutable_keys": ["seed"] },
//!   "masks":    [{ "reason": "observed_at demo field is wall-clock" }],
//!   "selftest": { "plants": ["item-drop"] }
//! }
//! ```

use serde::{Deserialize, Serialize};

pub const SCENARIO_MANIFEST_FILENAME: &str = "scenario.host.json";
pub const MANIFEST_CONTRACT_REV: u64 = 1;

/// The emd arm's LEGACY spelling (`page.host.json` beside a page) — still
/// read, never silently preferred; see the migration behavior docs.
pub const LEGACY_PAGE_MANIFEST_FILENAME: &str = "page.host.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostManifest {
    /// Version of THIS manifest contract; a host refuses an unknown rev.
    #[serde(default)]
    pub contract_rev: u64,
    /// Which arm the scenario runs in: "emd-page" (the emd-page-host arm)
    /// or "plain" (a plain fixture app mounted through the launch command).
    /// A declared kind is validated against the mounting arm — a page
    /// mounted through the wrong arm is a wrong-mount error, never a
    /// silent reinterpretation.
    #[serde(default)]
    pub kind: Option<String>,
    /// The plain arm's entry: how the fixture app is LAUNCHED (the runner
    /// command contract). Required when kind="plain"; irrelevant to emd.
    #[serde(default)]
    pub launch: Option<String>,
    /// Free-form scenario revision label; joins evidence provenance.
    #[serde(default)]
    pub revision: Option<String>,
    /// Implementation dependency: the component/fixture crate + pinned
    /// revision. Arm-resolved; anything unresolved is actionable.
    #[serde(default)]
    pub dependencies: Dependencies,
    /// Design dependency: semantic tokens/brand of the receiving app,
    /// resolved against the pinned profiles below (chain 0028 Q4).
    #[serde(default)]
    pub design: DesignProfiles,
    /// The synthetic fixture this scenario hosts. These values are the ONLY
    /// initial state `reset` may restore.
    #[serde(default)]
    pub fixture: FixtureDecl,
    /// Digest masks, declared WITH reasons, before any candidate is judged.
    #[serde(default)]
    pub masks: Vec<DigestMask>,
    /// Declared planted defects the selftest must detect on a SYNTHETIC
    /// candidate copy of the scenario. Declared before inspection.
    #[serde(default)]
    pub selftest: SelftestDecl,
}

impl Default for HostManifest {
    fn default() -> Self {
        HostManifest {
            contract_rev: MANIFEST_CONTRACT_REV,
            kind: None,
            launch: None,
            revision: None,
            dependencies: Dependencies::default(),
            design: DesignProfiles::default(),
            fixture: FixtureDecl::default(),
            masks: Vec::new(),
            selftest: SelftestDecl::default(),
        }
    }
}

impl HostManifest {
    /// Manifest-only validation (shape and internal consistency). Checks
    /// that reference the mounted scene structure — mask item spans —
    /// happen at mounting, where the real item count lives.
    pub fn validate(&self) -> Result<(), String> {
        if self.contract_rev != MANIFEST_CONTRACT_REV {
            return Err(format!(
                "manifest: contract_rev {} unsupported; this host implements contract rev {}",
                self.contract_rev, MANIFEST_CONTRACT_REV
            ));
        }
        for name in &self.fixture.mutable_keys {
            if !self.fixture.values.contains_key(name) {
                return Err(format!(
                    "manifest: fixture.mutable_keys declares {name:?} but fixture.values has no such key — a mutable key WITHOUT initial state is undeclared behavior; declare the key's initial value",
                ));
            }
        }
        for (name, digest) in &self.design.profiles {
            if !digest.starts_with("sha256:") {
                return Err(format!(
                    "manifest: design.profiles.\"{name}\": digest must be \"sha256:<hex>\", got {digest:?}"
                ));
            }
        }
        if let Some(dep) = &self.dependencies.design {
            if !self.design.profiles.contains_key(&dep.profile) {
                return Err(format!(
                    "manifest: dependencies.design.profile \"{}\" is not a pinned profile — declare it under design.profiles",
                    dep.profile
                ));
            }
            let pinned = self
                .design
                .profiles
                .get(&dep.profile)
                .map(|key| key.as_str())
                .unwrap_or("");
            if pinned != dep.digest {
                return Err(format!(
                    "manifest: dependencies.design.digest \"{}\" does not match the pinned profile \"{}\" digest \"{}\" — the scenario consumes tokens it does not declare; update the dependency digest (a NEW scenario revision), never the pin silently",
                    dep.digest, dep.profile, self.design.profiles[&dep.profile]
                ));
            }
        }
        Ok(())
    }

    /// Item-span validation for masks — validated against the mounted
    /// scene (per-arm item counts; for emd, one item = one fence).
    pub fn validate_item_span(&self, item_count: usize) -> Result<(), String> {
        for m in &self.masks {
            if let Some(i) = m.item {
                if i >= item_count {
                    return Err(format!(
                        "manifest: masks[].item {i} names no item on this scenario ({} items)",
                        item_count
                    ));
                }
            }
        }
        Ok(())
    }

    /// The mask reason covering an item, if any. Covered items are excluded
    /// from strict digest equality (e.g. a by-design wall-clock field).
    pub fn item_masked(&self, item_index: usize) -> Option<&str> {
        self.masks.iter().find_map(|m| {
            m.item
                .filter(|&f| f == item_index)
                .map(|_| m.reason.as_str())
        })
    }

    pub fn fixture_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.fixture.values.get(key)
    }

    pub fn is_mutable_key(&self, key: &str) -> bool {
        self.fixture.mutable_keys.iter().any(|k| k == key)
    }

    /// The arm this manifest declares, as a simple string (None = unset).
    pub fn declared_kind(&self) -> Kind {
        match self.kind.as_deref() {
            Some("emd-page") => Kind::EmdPage,
            Some("plain") => Kind::Plain,
            Some(other) => Kind::Unknown(other.to_string()),
            None => Kind::Unset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    EmdPage,
    Plain,
    Unset,
    Unknown(String),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    pub implementation: Vec<ImplementationDep>,
    #[serde(default)]
    pub design: Option<DesignDep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplementationDep {
    /// The component/fixture crate the scenario's items must come from
    /// (e.g. "emd-renderer"). Typed: a typo here is an unresolved
    /// dependency, never a silent prose-only scenario.
    #[serde(rename = "crate", alias = "crate_name")]
    pub crate_name: String,
    /// Pin: "workspace:<semver>", "component:<n>", or an arm-declared pin
    /// shape ("script:<n>" for the plain arm). Unresolvable pins are
    /// actionable dependency failures.
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignDep {
    /// The token profile key (the semantic-token set of the receiving app).
    pub profile: String,
    /// Its recorded digest — must MATCH the pinned profile digest; a mismatch
    /// means the scenario consumes tokens it does not declare.
    pub digest: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DesignProfiles {
    /// profile key → recorded token digest ("sha256:<hex>"). The SSOT for
    /// machine-readable tokens lives in the libyggterm L1 layer; this
    /// manifest PINS which profile the scenario consumes and its exact
    /// digest at the time the scenario was authored.
    #[serde(default)]
    pub profiles: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FixtureDecl {
    /// Declared initial (synthetic) fixture values. Only these are the
    /// initial state reset restores.
    #[serde(default)]
    pub values: serde_json::Map<String, serde_json::Value>,
    /// Keys `fixture_set` may mutate; an undeclared key is a typed-boundary
    /// error, never a silent overwrite.
    #[serde(default)]
    pub mutable_keys: Vec<String>,
}

/// One digest mask: an item-scoped reason a digest may legitimately change
/// (a by-design wall-clock field, a live-tick animation). Declared in the
/// manifest with a REASON, never discovered after the fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestMask {
    pub reason: String,
    /// Item index this mask applies to; omitted = the whole scenario.
    /// The legacy spelling `"fence"` (emd pages) reads as an alias —
    /// migration behavior: both spellings parse under contract rev 1.
    #[serde(default, alias = "fence")]
    pub item: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelftestDecl {
    /// Plant ids the selftest must detect. The supported vocabulary is
    /// defined per arm; an unknown plant id is a declaration error.
    #[serde(default)]
    pub plants: Vec<String>,
}
