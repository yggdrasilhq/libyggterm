// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! fixture-host — the PORTABLE scenario contract (chain 0029 rescoping of
//! the emd component-runtime brief):
//!
//! The falsifiable unit is the SCENARIO: component implementation +
//! fixture + interaction sequence + expected observations. This crate is
//! the arm-neutral half of it — one manifest model, one mount abstraction
//! ([`Mounted`]), one controlled lifecycle ([`ScenarioHost`]), one common
//! comparison judge and evidence bundle. Two arms implement it:
//!
//! - **emd arm** (`emd-page-host`): an emd component page mounted from a
//!   parsed markdown/fences scene;
//! - **plain arm** (`crate::plain`): an ordinary fixture app (the
//!   miniapp pattern) mounted through the runner command contract —
//!   `capabilities | reset_fixture | observe`.
//!
//! Which arm WINS (plain vs emd, on time/reset/detection/maintenance) is
//! the 0029 deciding experiment — NOT decided here; this crate exists so
//! the experiment runs against ONE contract instead of two.
//!
//! - **version-scoped manifests** — contract rev validation before
//!   anything else; the manifest declares kind/launch/fixture/masks/
//!   selftest plants;
//! - **typed fixture boundaries** — mutation only through declared keys;
//!   reload draws nothing from the previous host;
//! - **deterministic evidence** — per-item digests + fixture-joined
//!   scenario digest; identical input ⇒ identical digest;
//! - **actionable failures** — unresolved deps and invalid items carry
//!   keys/regions in the findings, never silent prose;
//! - **ux-falsify bundles** — the common evidence schema, one judge.

pub mod bundle;
pub mod digest_util;
pub mod host;
pub mod manifest;
pub mod mounted;
pub mod plain;
pub mod resolution;
pub mod selftest;

pub use bundle::{ArmIdentity, Comparison};
pub use host::{Digests, FixtureState, HostError, Lifecycle, ScenarioHost};
pub use manifest::{
    HostManifest, Kind, LEGACY_PAGE_MANIFEST_FILENAME, MANIFEST_CONTRACT_REV,
    SCENARIO_MANIFEST_FILENAME,
};
pub use mounted::{ItemDigest, Mounted};
pub use resolution::{Resolution, Resolutions, Resolved};
pub use selftest::{ArmPlants, PlantResult, SelftestReport};

pub const HOST_VERSION: &str = concat!(
    "fixture-host/0.1 (workspace ",
    env!("CARGO_PKG_VERSION"),
    ")"
);
