// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The controlled scenario lifecycle (ARM-NEUTRAL):
//!
//! load (manifest validation + arm mounting + dependency resolution)
//!   → host (fixture set/read through the TYPED mutable surface)
//!   → reset (EXACTLY the manifest-declared initial fixture)
//!   → ready (a readiness predicate: valid mount, no pending fixture
//!     mutation without re-render, digests recomputable — computed TWICE
//!     identically)
//!   → unload (drop; no scenario-level state survives on the host object).
//!
//! REPRODUCIBILITY (the claim being tested, chain 0028 Q4): re-mounting
//! the same scenario reconstructs an IDENTICAL host — fixture state resets
//! to the declared values, digests recompute identically, nothing leaks
//! between navigations. The host object owns no process it did not
//! declare; a reload draws nothing from the previous one ("repeated
//! navigation/reload leaves no orphaned state and no duplicate handlers").

use crate::manifest::HostManifest;
use crate::mounted::Mounted;
use crate::resolution::Resolutions;
use sha2::Digest as _;

/// The fixture as the HOST sees it: declared values plus mutations.
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureState {
    /// The manifest-declared initial fixture (never stored as a reference —
    /// reset CLONES it back, so a corrupt reset can't alias host state).
    pub initial: serde_json::Map<String, serde_json::Value>,
    /// Declared mutable keys (the typed surface of mutation).
    pub mutable_keys: Vec<String>,
    /// Current values (initial at load/reset; mutated in between).
    pub current: serde_json::Map<String, serde_json::Value>,
}

impl FixtureState {
    pub fn initial_digest(&self) -> String {
        crate::digest_util::sha256_hex(
            serde_json::to_string(&self.initial)
                .unwrap_or_default()
                .as_bytes(),
        )
    }

    pub fn current_digest(&self) -> String {
        crate::digest_util::sha256_hex(
            serde_json::to_string(&self.current)
                .unwrap_or_default()
                .as_bytes(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostError {
    /// The scenario could not be MOUNTED (arm-specific parse/launch error).
    Mount(String),
    Manifest(String),
    Dependency(String),
    NotReady(String),
    Fixture(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::Mount(message) => write!(f, "mount: {message}"),
            HostError::Manifest(message) => write!(f, "{message}"),
            HostError::Dependency(message) => write!(f, "dependency: {message}"),
            HostError::NotReady(message) => write!(f, "not ready: {message}"),
            HostError::Fixture(message) => write!(f, "fixture: {message}"),
        }
    }
}

impl std::error::Error for HostError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Loaded,
    Hosted, // fixture mutated away from the declared initial state
    Reset,  // back at the declared initial state after a reset
    Unloaded,
    /// A staged mutation LOST readiness (digest would differ from render);
    /// explicit `reset` is required before claiming a state on this host.
    LostReadiness,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioHost<M: Mounted> {
    /// Scenario identity: subject path (page path or app identity).
    pub path: String,
    pub manifest: HostManifest,
    pub mounted: M,
    pub fixture: FixtureState,
    pub resolutions: Resolutions,
    pub lifecycle: Lifecycle,
    /// Loaded time counter, for reload-leak detection in tests/mock drivers
    /// (a reset must NOT advance it; but a RELOAD gets a fresh host).
    pub load_generation: u64,
}

impl<M: Mounted> ScenarioHost<M> {
    /// Load: mount THE ARM'S SCENE, validate the manifest, resolve deps,
    /// and validate that mask item spans name real items. A scenario with
    /// invalid items LOADS but is never `ready` — checking reports them;
    /// a host can render the visible error cards.
    pub fn load(
        path: impl Into<String>,
        manifest: HostManifest,
        mounted: M,
    ) -> Result<Self, HostError> {
        if let Err(message) = manifest.validate() {
            return Err(HostError::Manifest(message));
        }
        let resolutions = mounted.resolutions(&manifest);
        let items = mounted.item_digests().map_err(HostError::Mount)?;
        if let Err(message) = manifest.validate_item_span(items.len()) {
            return Err(HostError::Manifest(message));
        }
        let fixture = FixtureState {
            initial: manifest.fixture.values.clone(),
            mutable_keys: manifest.fixture.mutable_keys.clone(),
            current: manifest.fixture.values.clone(),
        };
        Ok(ScenarioHost {
            path: path.into(),
            manifest,
            mounted,
            fixture,
            resolutions,
            lifecycle: Lifecycle::Loaded,
            load_generation: 1, // one load
        })
    }

    /// Mutation via the typed surface. An undeclared key is a boundary
    /// violation the host REJECTS (the manifest is the boundary, not the
    /// driver).
    pub fn fixture_set(&mut self, key: &str, value: serde_json::Value) -> Result<(), HostError> {
        if !self.manifest.is_mutable_key(key) {
            return Err(HostError::Fixture(format!(
                "manifest: fixture.mutable_keys does not declare {key:?} — a host mutates ONLY declared keys (declare the key to make the mutation part of the scenario's declared behavior)"
            )));
        }
        self.fixture.current.insert(key.to_string(), value);
        self.lifecycle = Lifecycle::Hosted;
        Ok(())
    }

    pub fn fixture_read(&self, key: &str) -> Option<&serde_json::Value> {
        self.fixture.current.get(key)
    }

    /// Reset: restore EXACTLY the manifest-declared fixture and return to
    /// Readiness-recomputable lifecycle. Declared fixture state is cleared
    /// by construction (only the declared values remain).
    pub fn reset(&mut self) -> Result<(), HostError> {
        if self.lifecycle == Lifecycle::Unloaded {
            return Err(HostError::NotReady(
                "host already unloaded — reset on an unloaded host is a lifecycle violation; reload the scenario instead \
                 (a reset must never resurrect a dropped host)"
                    .to_string(),
            ));
        }
        self.fixture.current = self.manifest.fixture.values.clone();
        self.lifecycle = Lifecycle::Reset;
        Ok(())
    }

    /// Readiness predicate. A ready host: valid mount, resolved deps, no
    /// fixture violation, and its digests COMPUTABLE TWICE identically
    /// (per the determinism claim — the cheap local form of the same rule
    /// ux-falsify applies to captures).
    pub fn ready(&self) -> Result<bool, HostError> {
        if !self.mounted.is_valid() {
            return Ok(false);
        }
        if !self.resolutions.ok() {
            return Ok(false);
        }
        let digests = self.digests().map_err(|message| {
            HostError::NotReady(format!("digest computation failed: {message}"))
        })?;
        if digests
            != self.digests().map_err(|message| {
                HostError::NotReady(format!("digest computation failed: {message}"))
            })?
        {
            return Ok(false);
        }
        Ok(self.lifecycle != Lifecycle::LostReadiness)
    }

    /// Scenario digests: per-item digests + the scenario digest
    /// (fixture-joined — altered fixture ≠ unaltered scenario).
    pub fn digests(&self) -> Result<Digests, String> {
        let components = self.mounted.item_digests()?;
        let fixture_hash = self.fixture.current_digest();
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.mounted.content_sha256().as_bytes());
        for component in &components {
            hasher.update(component.index.to_le_bytes());
            hasher.update(component.digest.as_bytes());
        }
        hasher.update(fixture_hash.as_bytes());
        let page = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(Digests {
            components,
            fixture_hash,
            page,
        })
    }

    /// Unload — the controlled terminal state. Returning the host consumed
    /// proves nothing survives: every subsequent use is a fresh load.
    pub fn unload(self) -> Lifecycle {
        Lifecycle::Unloaded
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Digests {
    /// Per-item digests (fences for the emd arm; observation nodes for the
    /// plain arm).
    pub components: Vec<crate::mounted::ItemDigest>,
    pub fixture_hash: String,
    /// The scenario digest (fixture-joined).
    pub page: String,
}
