// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The controlled hosted-component lifecycle:
//!
//! load (parse + manifest validation + dependency resolution)
//!   → host (fixture set/read through the TYPED mutable surface)
//!   → reset (EXACTLY the manifest-declared initial fixture)
//!   → ready (a readiness predicate: validated, no pending fixture
//!     mutation without re-render, digests recomputable)
//!   → unload (drop; no page-level state survives on the host object).
//!
//! REPRODUCIBILITY (the claim being tested): re-loading the same page in
//! the same workspace reconstructs an IDENTICAL host — fixture state resets
//! to the declared values, digests recompute identically, nothing leaks
//! between navigations. The `reload_is_clean` test is the repairable
//! stated guarantee behind "repeated navigation/reload leaves no orphaned
//! state and no duplicate handlers": the host object owns no process and
//! registers nothing global — loading twice CANNOT duplicate anything.

use crate::error::PageError;
use crate::resolve::{Resolutions, resolve_all};
use crate::{HostManifest, Page, ParsedPage};
use sha2::Digest as _;

/// The fixture as the HOST sees it: declared values plus mutations.
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureState {
    /// The manifest-declared initial fixture (never stored as a reference —
    /// reset CLONES it back, so a corrupt reset can't alias page state).
    pub initial: serde_json::Map<String, serde_json::Value>,
    /// Declared mutable keys (the typed surface of mutation).
    pub mutable_keys: Vec<String>,
    /// Current values (initial at load/reset; mutated in between).
    pub current: serde_json::Map<String, serde_json::Value>,
}

impl FixtureState {
    pub fn initial_digest(&self) -> String {
        crate::page::sha256_hex(
            serde_json::to_string(&self.initial)
                .unwrap_or_default()
                .as_bytes(),
        )
    }

    pub fn current_digest(&self) -> String {
        crate::page::sha256_hex(
            serde_json::to_string(&self.current)
                .unwrap_or_default()
                .as_bytes(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostError {
    Page(PageError),
    Manifest(String),
    Dependency(String),
    NotReady(String),
    Fixture(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::Page(error) => write!(f, "page: {}", error.message),
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
pub struct PageHost {
    pub page: Page,
    pub manifest: HostManifest,
    pub parsed: ParsedPage,
    pub fixture: FixtureState,
    pub resolution: Resolutions,
    pub lifecycle: Lifecycle,
    /// Loaded time counter, for reload-leak detection in tests/mock drivers
    /// (a reset must NOT advance it; but a RELOAD gets a fresh host).
    pub load_generation: u64,
    _private: (),
}

impl PageHost {
    /// Load: parse the page, validate the manifest shape, resolve deps,
    /// validate that mask fence spans name real fences, and confirm the
    /// page is VALID (a page with fence errors loads but is never `ready` —
    /// checking reports them; a host can render the visible error cards).
    pub fn load(page: Page, manifest: HostManifest) -> Result<Self, HostError> {
        if let Err(message) = manifest.validate() {
            return Err(HostError::Manifest(message));
        }
        let parsed = crate::page::parse_page(&page).map_err(HostError::Page)?;
        if let Err(message) = manifest.validate_fence_span(parsed.components.len()) {
            return Err(HostError::Manifest(message));
        }
        let resolution = resolve_all(&manifest, &page);
        let fixture = FixtureState {
            initial: manifest.fixture.values.clone(),
            mutable_keys: manifest.fixture.mutable_keys.clone(),
            current: manifest.fixture.values.clone(),
        };
        Ok(PageHost {
            page,
            manifest,
            parsed,
            fixture,
            resolution,
            lifecycle: Lifecycle::Loaded,
            load_generation: 1, // one load
            _private: (),
        })
    }

    /// Mutation via the typed surface. An undeclared key is a boundary
    /// violation the host REJECTS (the page is the boundary, not the app).
    pub fn fixture_set(&mut self, key: &str, value: serde_json::Value) -> Result<(), HostError> {
        if !self.manifest.is_mutable_key(key) {
            return Err(HostError::Fixture(format!(
                "page.host.json: fixture.mutable_keys does not declare {key:?} — a page host mutates ONLY declared keys (declare the key to make the mutation part of the page's declared behavior)"
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
                "host already unloaded — reset on an unloaded host is a lifecycle violation; reload the page instead \
                 (a reset must never resurrect a dropped host)"
                    .to_string(),
            ));
        }
        self.fixture.current = self.manifest.fixture.values.clone();
        self.lifecycle = Lifecycle::Reset;
        Ok(())
    }

    /// Readiness predicate. A ready host: valid page, resolved deps, no
    /// fixture violation, and its digests COMPUTABLE TWICE identically
    /// (per the determinism claim — the cheap local form of the same rule
    /// ux-falsify applies to captures).
    pub fn ready(&self) -> Result<bool, HostError> {
        if !self.parsed.is_valid() {
            return Ok(false);
        }
        if !self.resolution.ok() {
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

    /// Page digests: per-component digests + the page digest (fixture-joined).
    pub fn digests(&self) -> Result<Digests, String> {
        let mut components = Vec::new();
        for component in &self.parsed.components {
            match &component.component {
                Some(document) => {
                    let value = crate::digest::component_digest(&document.component)
                        .map_err(|error| error.message)?;
                    components.push(ComponentDigest {
                        fence_index: component.location.fence_index,
                        kind: component.kind.clone().unwrap_or_default(),
                        digest: value,
                        error: None,
                    });
                }
                None => components.push(ComponentDigest {
                    fence_index: component.location.fence_index,
                    kind: component.kind.clone().unwrap_or_default(),
                    digest: "invalid".to_string(),
                    error: component.error.clone(),
                }),
            }
        }
        let fixture_hash = self.fixture.current_digest();
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.parsed.content_sha256.as_bytes());
        for component in &components {
            hasher.update(component.fence_index.to_le_bytes());
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
pub struct ComponentDigest {
    pub fence_index: usize,
    pub kind: String,
    pub digest: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Digests {
    pub components: Vec<ComponentDigest>,
    pub fixture_hash: String,
    pub page: String,
}
