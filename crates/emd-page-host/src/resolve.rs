// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Dependency resolution for the emd arm: the logic lives ON the arm's
//! `Mounted` impl ([`crate::arm::resolve_implementation`],
//! [`crate::arm::resolve_design`], both against the page's content
//! witness); this module re-exports the TYPES plus the resolution entry
//! over a parsed PAGE (the call shape old call sites use).

use crate::page::{Page, ParsedPage};
pub use fixture_host::resolution::{Resolution, Resolutions, Resolved};

/// Resolve everything the manifest declares, against a page (the emd
/// ecosystem shape; ONE resolution process — the arm's trait impl).
pub fn resolve_all(manifest: &crate::manifest::HostManifest, page: &Page) -> Resolutions {
    use crate::arm as arm_impl;
    Resolutions {
        implementation: arm_impl::resolve_implementation(manifest, "page.host.json"),
        design: arm_impl::resolve_design(manifest, &page.content_sha256(), "page.host.json"),
    }
}

/// Provenance digest for tests/bundles (kept: one join spelling).
pub use crate::arm::{HOST_WORKSPACE_VERSION, WIRE_VERSION, design_token_digest};

/// Page-shaped alias kept so tests exercising per-page resolution via
/// parse results keep a single spelling.
pub fn resolve_parsed(manifest: &crate::manifest::HostManifest, parsed: &ParsedPage) -> Resolutions {
    use crate::arm as arm_impl;
    Resolutions {
        implementation: arm_impl::resolve_implementation(manifest, "page.host.json"),
        design: arm_impl::resolve_design(manifest, &parsed.content_sha256, "page.host.json"),
    }
}
