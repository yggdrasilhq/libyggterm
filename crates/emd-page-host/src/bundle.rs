// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The ux-falsify evidence bundle, EMD ARM spelling: the schema and the
//! judge live in `fixture_host::bundle`; this module is the arm identity
//! (driver string, page-shaped call signatures) so page call sites keep
//! one spelling.

use crate::host::PageHost;
pub use fixture_host::bundle::Comparison;
use fixture_host::bundle::{self as portable, ArmIdentity};
use serde_json::Value;
use std::path::PathBuf;

pub const DRIVER: &str = concat!("emd-page-host ", env!("CARGO_PKG_VERSION"));

fn arm() -> ArmIdentity {
    ArmIdentity {
        driver: DRIVER.to_string(),
        arm: "emd-page",
        pixel_note: "UNVERIFIED — pixels are a surface adapter's job (gallery-shot is the surface layer)",
    }
}

/// Bundle: `page` and `render` verbs over the portable schema (one schema,
/// both arms; the arm field names which mounting contract produced it).
pub fn render_bundle(host: &PageHost, verb: &str, pixel_note: &'static str) -> Value {
    let mut identity = portable::render_bundle(&host.scenario, arm(), verb);
    if !pixel_note.is_empty() {
        identity["capture"]["pixel_capture"] = serde_json::json!(pixel_note);
    }
    identity
}

/// Join repo revision identity into a bundle (CLI runs in a git context).
pub fn join_identity(bundle: &mut Value, git_revision: Option<String>) {
    portable::join_identity(bundle, git_revision);
}

/// Hashed retention (the TUI/web discipline): write under the sha256 of the
/// bundle's own canonical serialization; generated runs stay out of git.
pub fn retain(bundle_value: &Value) -> Result<PathBuf, String> {
    // Env rule preserved: UX_FALSIFY_EVIDENCE overrides the evidence root.
    let root = std::env::var("UX_FALSIFY_EVIDENCE").unwrap_or_else(|_| {
        home()
            .join(".local/share/ux-falsify-evidence/emd-page-host")
            .to_string_lossy()
            .into_owned()
    });
    portable::retain(bundle_value, &root)
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"))
}

/// The judge lives in the portable contract; call sites reach it as
/// `bundle::compare` — the single spelling.
pub fn compare(
    manifest: &crate::manifest::HostManifest,
    base: &fixture_host::Digests,
    candidate: &fixture_host::Digests,
) -> Comparison {
    portable::compare(manifest, base, candidate)
}
