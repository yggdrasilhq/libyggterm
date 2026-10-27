// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! emd-page-host — the EMD ARM of the portable scenario contract.
//! Portable half: `fixture_host` (one manifest model, one lifecycle, one
//! judge, one bundle schema — and the plain fixture-app arm). Spec:
//! `docs/spec-emd-components.md` §Hosted-component contract.
//!
//! The ydesign loop's claim (chain 0028 Q4, amended by chain 0029): the
//! falsifiable unit is the SCENARIO — implementation + fixture +
//! interaction + expected observations — and it must survive the notebook
//! renderer disappearing. This crate keeps the emd arm of that contract:
//!
//! - **the documented emd path** — a page is markdown parsed by
//!   `emd_renderer::parse_markdown_blocks`; every ```emd fence validates
//!   against the component wire version (version-first, the REJECTED
//!   version is named), with bounds and evidence rules preserved;
//! - **typed component boundaries** — behavior declared in a manifest
//!   (`page.host.json` beside the page; the portable `scenario.host.json`
//!   spelling also reads — see migration behavior);
//! - **dependency resolution with actionable failures** — implementation
//!   deps resolve against the local component workspace; design deps
//!   against the manifest's pinned token profiles;
//! - **deterministic render evidence** — per-fence scene digests,
//!   identical input ⇒ identical digest, fixture state joined in;
//! - **controlled lifecycle** — load → host → reset → unload through
//!   `fixture_host::ScenarioHost`; a reload draws NO residual state;
//! - **error presentation with actionable locations** — fence index,
//!   byte range, line:col, tree path (`root.grid.children[2].plot.series`);
//! - **ux-falsify evidence bundles** — `check`/`render`/`run`/`selftest`
//!   emit the one shared schema.

pub mod arm;
pub mod bundle;
pub mod digest;
pub mod error;
pub mod host;
pub mod manifest;
pub mod page;
pub mod resolve;
pub mod selftest;

pub use digest::component_digest;
pub use error::{PageError, PageItemError};
pub use fixture_host::bundle::Comparison;
pub use fixture_host::selftest::SelftestReport;
pub use fixture_host::{
    ArmPlants, Digests, FixtureState, HostError, ItemDigest, Lifecycle, ScenarioHost,
};
pub use host::PageHost;
pub use manifest::{
    HostManifest, MANIFEST_CONTRACT_REV, MANIFEST_FILENAME, SCENARIO_MANIFEST_FILENAME,
};
pub use page::{Page, PageComponent, ParsedPage};
pub use resolve::{Resolution, Resolutions, Resolved};

pub const HOST_VERSION: &str = concat!(
    "emd-page-host/0.1 (workspace ",
    env!("CARGO_PKG_VERSION"),
    ")"
);
