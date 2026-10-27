// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! emd-page-host — the reproducible executable test host for emd component
//! pages. Spec: `docs/spec-emd-components.md` §Hosted-component contract.
//!
//! The ydesign loop's load-bearing claim (chain 0028 Q4) is that a design
//! page is an EXECUTABLE HOST of cases, not a static mockup. What makes a
//! host executable and reproducible:
//!
//! - **versioned validation** — every ```emd fence validates against the
//!   component wire version (floor AND ceiling), with bounds and evidence
//!   rules preserved; version-first, so the REJECTED version is named;
//! - **typed component boundaries** — a page's behavior is declared in a
//!   `page.host.json` manifest (dependencies, fixture, masks); source-only
//!   pages are read-only prose hosts;
//! - **dependency resolution with actionable failures** — implementation
//!   dependencies resolve against the local component workspace; design
//!   dependencies resolve against the manifest's pinned token profiles;
//!   anything unresolved names the manifest key and the expected shape;
//! - **deterministic render evidence** — every component renders to a
//!   stable UI-neutral scene digest; identical input ⇒ identical digest;
//! - **fixture/readiness/reset** — a declared synthetic fixture with a
//!   typed mutable surface, a reset that reproduces EXACTLY the declared
//!   initial state, and a readiness predicate;
//! - **source/revision identity on every rendered artifact** — content
//!   digest, page revision label, workspace revision, per-component digests;
//! - **controlled lifecycle** — load → host → reset → unload; a reload
//!   draws NO residual state (re-executed tests assert digest equality);
//! - **error presentation with actionable locations** — fence index, byte
//!   range, line:col, and the component path inside the tree
//!   (`root.grid.children[2].plot.series`);
//! - **ux-falsify evidence bundles** — `check`/`render`/`run`/`selftest`
//!   emit the manifest-schema bundle of the ux-falsify skill.

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
pub use host::{Digests, FixtureState, HostError, Lifecycle, PageHost};
pub use manifest::{HostManifest, MANIFEST_CONTRACT_REV, MANIFEST_FILENAME};
pub use page::{Page, PageComponent, ParsedPage};
pub use resolve::{Resolution, Resolutions};

pub const HOST_VERSION: &str = concat!(
    "emd-page-host/0.1 (workspace ",
    env!("CARGO_PKG_VERSION"),
    ")"
);
