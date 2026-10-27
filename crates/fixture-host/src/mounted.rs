// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The MOUNTED SCENE abstraction. The portable contract knows nothing
//! about markdown, fences, or Dioxus — it knows only that a mounted
//! scenario reduces to a stable list of item digests plus identity. The
//! emd arm implements this over a parsed emd page; the plain arm over the
//! observation of a fixture app run through the runner command contract.

use crate::resolution::Resolutions;
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq)]
pub struct ItemDigest {
    /// Item index (0-based, document order for the emd arm; observation
    /// order for the plain arm). Evidence regions render +1 (fence N).
    pub index: usize,
    pub kind: String,
    pub digest: String,
    /// Actionable failure for this item, when it failed validation/render
    /// (never silently swallowed — the SAME message the host displays).
    pub error: Option<String>,
}

/// What an arm's validation errors look like INSIDE a bundle: one JSON
/// object per invalid item, in the shared evidence schema
/// (`{axis: "validation", region: …, message: …}`).
pub fn validation_item(_index: usize, region: String, message: &str) -> Value {
    json!({
        "axis": "validation",
        "region": region,
        "message": message,
    })
}

/// The arm-supplied identity of a mounted scene.
pub trait Mounted: std::fmt::Debug {
    /// Subject path: the page path (emd) or the fixture-app identity.
    fn path(&self) -> String;
    /// Source identity digest — content sha256 of the scenario source as
    /// the arm recorded it (the emd page source; the plain arm's full
    /// observation capture). One sha256 the bundle can bind.
    fn content_sha256(&self) -> String;
    /// Resolved dependency digest witnessed by the scenario's own source
    /// (for the emd arm: content+profile+pin joined; others may reuse it
    /// or return None to mark "no witness from source").
    fn design_witness(&self) -> Option<String>;
    /// Validity: does every item pass the arm's own validation?
    fn is_valid(&self) -> bool;
    /// The item digests (deterministic; computed THROUGH the arm's render
    /// path, never from the raw source alone).
    fn item_digests(&self) -> Result<Vec<ItemDigest>, String>;
    /// Validation findings in the bundle shape (one per invalid item).
    fn validation_items(&self) -> Vec<Value>;
    /// Resolve the manifest's dependencies IN THIS MOUNT (arm-specific).
    fn resolutions(&self, manifest: &crate::manifest::HostManifest) -> Resolutions;
    /// The bundle `identity.component` value (e.g. "emd-component-page").
    fn identity_component(&self) -> &'static str;
    /// The semantic null-slot count (missing observations), when the arm
    /// tracks it: the selftest's analytical probe enrichment. None = the
    /// arm reports no slot surface; the probe is skipped, never faked.
    fn null_slots(&self) -> Option<usize> {
        None
    }
}
