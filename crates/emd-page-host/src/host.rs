// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The emd PAGE HOST: the portable scenario lifecycle (`fixture_host::
//! ScenarioHost` over a parsed page) plus the emd arm's own identity —
//! the `Page`/`ParsedPage` types a page is loaded from. Parsing lives in
//! [`crate::page`]; arm trait work in [`crate::arm`]; the HOST lifecycle
//! laws (reset/readiness/reproducibility, the controlled surface) live
//! ONCE in fixture-host. This module keeps the emd ecosystem's names
//! (`PageHost`, `Page`) so call sites and evidence keep one spelling.

use crate::error::PageError;
use crate::page::{Page, parse_page};
use crate::resolve::Resolutions;
use fixture_host::manifest::HostManifest;
pub use fixture_host::HostError;
use fixture_host::{Lifecycle, ScenarioHost};
use std::ops::{Deref, DerefMut};

pub struct PageHost {
    /// The page identity (path + source + revision label).
    pub page: Page,
    pub parsed: ParsedPageAlias,
    /// The controlled LIFECYCLE owning the manifest, fixture state,
    /// reset, and readiness — the portable contract. The emd arm = a
    /// scene of fences (`ParsedPage` mounted as `fixture_host::Mounted`).
    pub scenario: ScenarioHost<crate::page::ParsedPage>,
}

// The parsed page duplicates as the ARM type; the alias keeps doc names.
pub type ParsedPageAlias = crate::page::ParsedPage;

impl Deref for PageHost {
    // Deref makes the scenario's OWN surface the page host's: host.manifest,
    // host.fixture, host.fixture_set(..), host.reset(), host.ready(),
    // host.digests(), host.unload(), host.load_generation.
    type Target = ScenarioHost<crate::page::ParsedPage>;
    fn deref(&self) -> &Self::Target {
        &self.scenario
    }
}

impl DerefMut for PageHost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scenario
    }
}

/// Resolutions under their arm spelling: a page-alias re-export (kept for
/// call sites that pattern-match on the resolution table).
pub fn resolutions_of(scenario: &ScenarioHost<crate::page::ParsedPage>) -> &Resolutions {
    &scenario.resolutions
}

impl std::fmt::Debug for PageHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageHost")
            .field("page", &self.page.path)
            .field("lifecycle", &self.scenario.lifecycle)
            .field("load_generation", &self.scenario.load_generation)
            .finish()
    }
}

impl PageHost {
    /// Load: parse the page through the documented path, validate the
    /// manifest, resolve deps (through the ARM trait impls), and mount.
    /// A page with fence errors LOADS but is never `ready` — checking
    /// reports them; a host can render the visible error cards.
    pub fn load(page: Page, manifest: HostManifest) -> Result<Self, HostError> {
        if let Err(message) = manifest.validate() {
            return Err(HostError::Manifest(message));
        }
        let parsed = parse_page(&page).map_err(|error| HostError::Mount(error.message.clone()))?;
        let scenario = ScenarioHost::load(&page.path, manifest, parsed.clone())?;
        Ok(PageHost {
            page,
            parsed,
            scenario,
        })
    }

    /// The emd arm's manifest spelling: accept BOTH `page.host.json`
    /// (beside a page — the documented emd place) and the portable
    /// `scenario.host.json`. MIGRATION: both parse identically (the
    /// manifest model is shared); the portable spelling is looked for
    /// FIRST, and either may be declared explicitly.
    pub fn load_manifest_beside(
        page_path: &std::path::Path,
    ) -> Option<(std::path::PathBuf, HostManifest)> {
        for name in [
            fixture_host::SCENARIO_MANIFEST_FILENAME,
            fixture_host::LEGACY_PAGE_MANIFEST_FILENAME,
        ] {
            let path = page_path
                .parent()
                .map(|p| p.join(name))
                .unwrap_or_else(|| std::path::PathBuf::from(name));
            if path.exists() {
                return std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|text| serde_json::from_str(&text).ok())
                    .map(|manifest| (path, manifest));
            }
        }
        None
    }

    /// Unload — the controlled terminal state (the scenario lifecycle,
    /// reachable by the ecosystem name here).
    pub fn unload(self) -> Lifecycle {
        self.scenario.unload()
    }
}

fn _keep_error_in_scope(error: PageError) -> String {
    error.message
}
