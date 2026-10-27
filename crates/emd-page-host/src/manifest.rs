// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The page-host manifest (`page.host.json`, beside the page): the DECLARED
//! executable behavior of a component page. Everything the host does beyond
//! prose reading is typed here, versioned, and validated BEFORE
//! inspection — declared masks, never discovered ones (ux-falsify law).
//!
//! Schema (contract rev = [`MANIFEST_CONTRACT_REV`], currently 1):
//!
//! ```json
//! {
//!   "contract_rev": 1,
//!   "revision": "2026-10-27-r1",
//!   "dependencies": {
//!     "implementation": [{ "crate": "emd-renderer", "revision": "workspace:0.14.4" }],
//!     "design": { "profile": "t3-base", "digest": "sha256:<hex>" }
//!   },
//!   "design":   { "profiles": { "t3-base": "sha256:<hex>" } },
//!   "fixture":  { "values": { "dataset": "synthetic-t3-buttons" },
//!                 "mutable_keys": ["dataset"] },
//!   "masks":    [{ "reason": "observed_at demo field is wall-clock", "fence": 0 }],
//!   "selftest": { "plants": ["null-to-zero", "tone-drift"] }
//! }
//! ```

use serde::{Deserialize, Serialize};

pub const MANIFEST_FILENAME: &str = "page.host.json";
pub const MANIFEST_CONTRACT_REV: u64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostManifest {
    /// Version of THIS manifest contract; a host refuses an unknown rev.
    #[serde(default)]
    pub contract_rev: u64,
    /// Free-form page revision label; joins evidence provenance.
    #[serde(default)]
    pub revision: Option<String>,
    /// Implementation dependency: the component crate + pinned revision.
    #[serde(default)]
    pub dependencies: Dependencies,
    /// Design dependency: semantic tokens/brand of the receiving app,
    /// resolved against the pinned profiles below (chain 0028 Q4).
    #[serde(default)]
    pub design: DesignProfiles,
    /// The synthetic fixture this page hosts. These values are the ONLY
    /// initial state `reset` may restore.
    #[serde(default)]
    pub fixture: FixtureDecl,
    /// Digest masks, declared WITH reasons, before any candidate is judged.
    #[serde(default)]
    pub masks: Vec<DigestMask>,
    /// Declared planted defects the selftest must detect on a SYNTHETIC
    /// candidate copy of this page. Declared before inspection.
    #[serde(default)]
    pub selftest: SelftestDecl,
}

impl Default for HostManifest {
    fn default() -> Self {
        HostManifest {
            contract_rev: MANIFEST_CONTRACT_REV,
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
    /// that reference page structure — mask fence spans — happen during page
    /// loading, where the real component count lives.
    pub fn validate(&self) -> Result<(), String> {
        if self.contract_rev != MANIFEST_CONTRACT_REV {
            return Err(format!(
                "page.host.json: contract_rev {} unsupported; this host implements contract rev {}",
                self.contract_rev, MANIFEST_CONTRACT_REV
            ));
        }
        for (name, digest) in &self.design.profiles {
            if !digest.starts_with("sha256:") {
                return Err(format!(
                    "page.host.json: design.profiles.\"{name}\": digest must be \"sha256:<hex>\", got {digest:?}"
                ));
            }
        }
        if let Some(dep) = &self.dependencies.design {
            if !self.design.profiles.contains_key(&dep.profile) {
                return Err(format!(
                    "page.host.json: dependencies.design.profile \"{}\" is not a pinned profile — declare it under design.profiles",
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
                    "page.host.json: dependencies.design.digest \"{}\" does not match the pinned profile \"{}\" digest \"{}\" — the page consumes tokens it does not declare; update the dependency digest (a NEW page revision), never the pin silently",
                    dep.digest, dep.profile, self.design.profiles[&dep.profile]
                ));
            }
        }
        Ok(())
    }

    /// Fence range validation for masks — validated against the parsed page.
    pub fn validate_fence_span(&self, component_count: usize) -> Result<(), String> {
        for m in &self.masks {
            if let Some(i) = m.fence {
                if i >= component_count {
                    return Err(format!(
                        "page.host.json: masks[].fence {i} names no component fence on this page ({} fences)",
                        component_count
                    ));
                }
            }
        }
        Ok(())
    }

    /// The mask reason covering a fence, if any. Covered fences are excluded
    /// from strict digest equality (e.g. a by-design wall-clock field).
    pub fn fence_masked(&self, fence_index: usize) -> Option<&str> {
        self.masks.iter().find_map(|m| {
            m.fence
                .filter(|&f| f == fence_index)
                .map(|_| m.reason.as_str())
        })
    }

    pub fn fixture_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.fixture.values.get(key)
    }

    pub fn is_mutable_key(&self, key: &str) -> bool {
        self.fixture.mutable_keys.iter().any(|k| k == key)
    }
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
    /// The component crate the page's widgets must come from
    /// (e.g. "emd-renderer"). Typed: a typo here is an unresolved dependency,
    /// never a silent prose-only page.
    #[serde(rename = "crate", alias = "crate_name")]
    pub crate_name: String,
    /// Pin: "workspace:<semver>" (resolved against the workspace's own
    /// published version) or "component:<n>" (the component wire version).
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignDep {
    /// The token profile key (the semantic-token set of the receiving app).
    pub profile: String,
    /// Its recorded digest — must MATCH the pinned profile digest; a mismatch
    /// means the page consumes tokens it does not declare.
    pub digest: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DesignProfiles {
    /// profile key → recorded token digest ("sha256:<hex>"). The SSOT for
    /// machine-readable tokens lives in the libyggterm L1 layer (the ydesign
    /// chain); this manifest PINS which profile the page consumes and its
    /// exact digest at the time the page was authored.
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestMask {
    pub reason: String,
    /// Fence this mask applies to; omitted = the whole page.
    #[serde(default)]
    pub fence: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelftestDecl {
    /// Plant ids the selftest must detect. The supported vocabulary is
    /// [`crate::selftest`]'s const plant set; an unknown plant id is a
    /// declaration error.
    #[serde(default)]
    pub plants: Vec<String>,
}
