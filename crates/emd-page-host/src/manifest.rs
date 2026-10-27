// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The manifest model is now the PORTABLE scenario contract
//! (`fixture_host`): this module stays as the emd ecosystem's door to it
//! so call sites keep their spellings. The emd pages' manifest place is
//! `page.host.json` beside the page (legacy spelling, still read); the
//! portable file name is `scenario.host.json`. Both parse identically
//! under contract rev 1.

pub use fixture_host::manifest::{
    Dependencies, DesignDep, DesignProfiles, DigestMask, FixtureDecl, HostManifest,
    ImplementationDep, Kind, LEGACY_PAGE_MANIFEST_FILENAME, MANIFEST_CONTRACT_REV,
    SCENARIO_MANIFEST_FILENAME, SelftestDecl,
};

/// The emd arm's manifest FILENAME (the documented place, kept so a
/// page-only call site keeps its spelling; the loader accepts both).
pub const MANIFEST_FILENAME: &str = LEGACY_PAGE_MANIFEST_FILENAME;
