// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The COMMON evidence contract: hashed bundles in the ux-falsify manifest
//! schema, and the comparison verdicts (`CLEAN | MISMATCH`, `UNVERIFIED`
//! when anything could not be captured) shared by render runs, deterministic
//! `run` rounds, and the planted-defect selftest. One judge, everywhere —
//! the same reason the TUI and web adapters share one differ law.

use crate::HostManifest;
use crate::host::{Digests, PageHost};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub const DRIVER: &str = concat!("emd-page-host ", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Comparison {
    pub verdict: String,
    pub findings: Vec<String>,
}

/// Mask-aware, judge-shaped comparison of two digest sets. Verdicts are the
/// comparison verdicts — never MATCH (that belongs to the full acceptance
/// loop contract): this layer says CLEAN, the loop says MATCH.
pub fn compare(manifest: &HostManifest, base: &Digests, candidate: &Digests) -> Comparison {
    // Fence count divergence is a mismatch even when every digest equalled
    // (a candidate that LOSES a fence is an altered page).
    if base.components.len() != candidate.components.len() {
        return Comparison {
            verdict: "MISMATCH".to_string(),
            findings: vec![format!(
                "axis=bounds: fence count differs (ref {}, candidate {}) — an added or removed component is an altered page",
                base.components.len(),
                candidate.components.len()
            )],
        };
    }
    let mut findings = Vec::new();
    for (base_component, candidate_component) in base.components.iter().zip(&candidate.components) {
        if let Some(reason) = manifest.fence_masked(base_component.fence_index) {
            // Declared mask — recorded, never silently skipped.
            findings.push(format!(
                "mask recorded: fence {} masked ({reason})",
                base_component.fence_index + 1
            ));
            continue;
        }
        match (&base_component.error, &candidate_component.error) {
            (Some(base_message), Some(candidate_message)) if base_message == candidate_message => {}
            (None, None) => {}
            (base_message, candidate_message) => {
                findings.push(format!(
                    "axis=validation: fence {} renders a different actionable error (ref: {:?}; candidate: {:?})",
                    base_component.fence_index + 1,
                    base_message.as_deref().unwrap_or("<valid>"),
                    candidate_message.as_deref().unwrap_or("<valid>")
                ));
            }
        }
        // Digest inequality covers value, delta, tone, geometry — everything
        // the scene owns; the finding's evidence region is the fence index.
        if base_component.digest != candidate_component.digest {
            findings.push(format!(
                "axis=scene: fence {} ({}) digest diverges (ref {}… candidate {}…)",
                base_component.fence_index + 1,
                base_component.kind,
                &base_component.digest[..16.min(base_component.digest.len())],
                &candidate_component.digest[..16.min(candidate_component.digest.len())]
            ));
        }
    }
    if base.fixture_hash != candidate.fixture_hash {
        findings.push(
            "axis=fixture: fixture state digest diverges — the declared fixture is part of the page digest, so an altered fixture state can never look like an unaltered page".to_string(),
        );
    }
    let mismatch = findings.iter().any(|f| f.starts_with("axis="));
    Comparison {
        verdict: if mismatch { "MISMATCH" } else { "CLEAN" }.to_string(),
        findings,
    }
}

/// The page-render bundle in the ux-falsify manifest schema. Pixel capture
/// is NOT this layer's job: scene-truth (the UI-neutral scene digests) is
/// the structural capture; pixel proving happens through a host-SURFACE
/// adapter later and is recorded UNVERIFIED at this layer, never faked.
pub fn render_bundle(host: &PageHost, verb: &str, pixel_note: &'static str) -> Value {
    let (digests, bundle) = match host
        .digests()
        .map_err(|error| format!("digest error: {error}"))
    {
        Ok(digests) => (digests, None),
        Err(message) => (
            Digests {
                components: Vec::new(),
                fixture_hash: host.fixture.initial_digest(),
                page: "unavailable".to_string(),
            },
            Some(json!({
                "driver": DRIVER, "verb": verb, "verdict": "UNVERIFIED",
                "findings": [message],
                "page": host.page.path,
            })),
        ),
    };
    if let Some(bundle) = bundle {
        return bundle;
    }
    let components: Vec<Value> = digests
        .components
        .iter()
        .map(|c| json!({ "fence": c.fence_index + 1, "kind": c.kind, "digest": c.digest, "error": c.error }))
        .collect();
    let fence_errors: Vec<Value> = host
        .parsed
        .components
        .iter()
        .filter(|c| c.error.is_some())
        .map(|c| {
            json!({
                "axis": "validation",
                "region": format!("fence {} at {}:{} path={}",
                    c.location.fence_index + 1, c.location.line, c.location.col, c.location.block_path),
                "message": c.error,
            })
        })
        .collect();
    let resolved_token_digest =
        host.resolution
            .design
            .as_ref()
            .and_then(|(_, resolution)| match resolution {
                crate::resolve::Resolution::Ok(resolved) => {
                    Some(resolved.workspace_version.clone())
                }
                _ => None,
            });
    json!({
        "driver": DRIVER,
        "verb": verb,
        "identity": {
            "component": "emd-component-page",
            "state": "rendered",
            "claim": "the page is a reproducible executable host: identical input renders identical scene evidence",
            "contract_rev": host.manifest.contract_rev,
        },
        "subjects": {
            "page": host.page.path,
            "page_content_sha256": host.parsed.content_sha256,
            "page_revision": host.manifest.revision,
            "workspace_host_revision": crate::resolve::HOST_WORKSPACE_VERSION,
            "component_wire_version": crate::resolve::WIRE_VERSION,
        },
        "environment": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
        "fixture": {
            "declared_initial": host.manifest.fixture.values,
            "current": host.fixture.current,
            "mutable_keys": host.manifest.fixture.mutable_keys,
            "fixture_sha256": digests.fixture_hash,
            "corpus_rule": "synthetic only — no private data in the repo",
        },
        "replay": {
            "reset_recipe": "fixture current := manifest-declared initial values",
            "lifecycle": format!("{:?}", host.lifecycle),
            "load_generation": host.load_generation,
        },
        "capture": {
            "components": components,
            "page_digest": digests.page,
            "pixel_capture": pixel_note,
            "fence_count": digests.components.len(),
        },
        "evaluation": {
            "page_digest": digests.page,
            "resolutions_clean": host.resolution.ok(),
            "unresolved_dependencies": host.resolution.failures(),
            "fence_errors": fence_errors,
        },
        "provenance": {
            "driver_version": DRIVER,
            "page_content_sha256": host.page.content_sha256(),
            "page_revision": host.page.revision,
            "resolved_token_digest": resolved_token_digest,
            "git_revision_note": "the CLI joins `git rev-parse` (see join_identity); a tarball checkout records EXPLICIT emptiness — never silently missing",
        }
    })
}

/// Join repo revision identity into a bundle (CLI runs in a git context).
pub fn join_identity(bundle: &mut Value, git_revision: Option<String>) {
    bundle["provenance"]["git_revision"] = match git_revision {
        Some(rev) => json!(rev),
        None => json!(null),
    };
}

/// Hashed retention (the TUI/web discipline): write under the sha256 of the
/// bundle's own canonical serialization; generated runs stay out of git.
pub fn retain(bundle_value: &Value) -> Result<PathBuf, String> {
    let root = std::env::var("UX_FALSIFY_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(".local/share/ux-falsify-evidence/emd-page-host"));
    let canonical = serde_json::to_vec_pretty(bundle_value).map_err(|e| e.to_string())?;
    let hash = hex(&Sha256::digest(&canonical));
    let dir = root.join(&hash[..12]);
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let file = dir.join("bundle.json");
    std::fs::write(&file, canonical).map_err(|error| error.to_string())?;
    Ok(file)
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
