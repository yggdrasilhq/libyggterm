// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The COMMON evidence contract: hashed bundles in the ux-falsify manifest
//! schema, and the comparison verdicts (`CLEAN | MISMATCH`, `UNVERIFIED`
//! when anything could not be captured) shared by render runs, deterministic
//! run rounds, and the planted-defect selftest. One judge, everywhere —
//! the same reason the TUI and web adapters share one differ law.

use crate::manifest::HostManifest;
use crate::mounted::Mounted;
use crate::host::{Digests, ScenarioHost};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

/// Arm-supplied identity for bundle building.
pub struct ArmIdentity {
    pub driver: String,
    /// Which arm's mounting contract produced this scene ("emd-page" or
    /// "plain" — the manifest's kind value).
    pub arm: &'static str,
    /// The pixel-capture statement (scene-truth is the structural capture;
    /// pixel proof is a SURFACE adapter's job and stays UNVERIFIED here).
    pub pixel_note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    pub verdict: String,
    pub findings: Vec<String>,
}

/// Mask-aware, judge-shaped comparison of two digest sets. Verdicts are the
/// comparison verdicts — never MATCH (that belongs to the full acceptance
/// loop contract): this layer says CLEAN, the loop says MATCH.
pub fn compare(manifest: &HostManifest, base: &Digests, candidate: &Digests) -> Comparison {
    // Item count divergence is a mismatch even when every digest equalled
    // (a candidate that LOSES an item is an altered scenario).
    if base.components.len() != candidate.components.len() {
        return Comparison {
            verdict: "MISMATCH".to_string(),
            findings: vec![format!(
                "axis=bounds: item count differs (ref {}, candidate {}) — an added or removed item is an altered scenario",
                base.components.len(),
                candidate.components.len()
            )],
        };
    }
    let mut findings = Vec::new();
    for (base_item, candidate_item) in base.components.iter().zip(&candidate.components) {
        if let Some(reason) = manifest.item_masked(base_item.index) {
            // Declared mask — recorded, never silently skipped.
            findings.push(format!(
                "mask recorded: item {} masked ({reason})",
                base_item.index + 1
            ));
            continue;
        }
        match (&base_item.error, &candidate_item.error) {
            (Some(base_message), Some(candidate_message)) if base_message == candidate_message => {}
            (None, None) => {}
            (base_message, candidate_message) => {
                findings.push(format!(
                    "axis=validation: item {} ({}) renders a different actionable error (ref: {:?}; candidate: {:?})",
                    base_item.index + 1,
                    base_item.kind,
                    base_message.as_deref().unwrap_or("<valid>"),
                    candidate_message.as_deref().unwrap_or("<valid>")
                ));
            }
        }
        // Digest inequality covers value, delta, tone, geometry — everything
        // the scene owns; the finding's evidence region is the item index.
        if base_item.digest != candidate_item.digest {
            findings.push(format!(
                "axis=scene: item {} ({}) digest diverges (ref {}… candidate {}…)",
                base_item.index + 1,
                base_item.kind,
                &base_item.digest[..16.min(base_item.digest.len())],
                &candidate_item.digest[..16.min(candidate_item.digest.len())]
            ));
        }
    }
    if base.fixture_hash != candidate.fixture_hash {
        findings.push(
            "axis=fixture: fixture state digest diverges — the declared fixture is part of the scenario digest, so an altered fixture state can never look like an unaltered scenario".to_string(),
        );
    }
    let mismatch = findings.iter().any(|f| f.starts_with("axis="));
    Comparison {
        verdict: if mismatch { "MISMATCH" } else { "CLEAN" }.to_string(),
        findings,
    }
}

/// The scenario-render bundle in the ux-falsify manifest schema. Pixel
/// capture is NOT this layer's job: scene-truth (the UI-neutral digests) is
/// the structural capture; pixel proving happens through a host-SURFACE
/// adapter and is recorded UNVERIFIED at this layer, never faked.
pub fn render_bundle<M: Mounted>(host: &ScenarioHost<M>, arm: ArmIdentity, verb: &str) -> Value {
    let (digests, fallback) = match host
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
                "driver": arm.driver, "verb": verb, "verdict": "UNVERIFIED",
                "findings": [message],
                "subject": host.mounted.path(),
            })),
        ),
    };
    if let Some(bundle) = fallback {
        return bundle;
    }
    let items: Vec<Value> = digests
        .components
        .iter()
        .map(|c| json!({ "item": c.index + 1, "kind": c.kind, "digest": c.digest, "error": c.error }))
        .collect();
    let resolved_token_digest = host
        .resolutions
        .design
        .as_ref()
        .and_then(|(_, resolution)| match resolution {
            crate::resolution::Resolution::Ok(resolved) => Some(resolved.workspace_version.clone()),
            _ => None,
        });
    let validation_ok = host.mounted.validation_items().is_empty();
    json!({
        "driver": arm.driver,
        "arm": arm.arm,
        "verb": verb,
        "identity": {
            "component": host.mounted.identity_component(),
            "state": "rendered",
            "claim": "the scenario is a reproducible executable unit: identical input renders identical scene evidence",
            "contract_rev": host.manifest.contract_rev,
        },
        "subjects": {
            "path": host.mounted.path(),
            "source_sha256": host.mounted.content_sha256(),
            "scenario_revision": host.manifest.revision,
            "launch_command": host.manifest.launch,
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
            "items": items,
            "scenario_digest": digests.page,
            "pixel_capture": arm.pixel_note,
            "item_count": digests.components.len(),
        },
        "evaluation": {
            "scenario_digest": digests.page,
            "validation_ok": validation_ok,
            "resolutions_clean": host.resolutions.ok(),
            "unresolved_dependencies": host.resolutions.failures(),
            "item_errors": host.mounted.validation_items(),
        },
        "provenance": {
            "driver_version": arm.driver,
            "source_sha256": host.mounted.content_sha256(),
            "scenario_revision": host.manifest.revision,
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
pub fn retain(bundle_value: &Value, root: &str) -> Result<PathBuf, String> {
    let canonical = serde_json::to_vec_pretty(bundle_value).map_err(|e| e.to_string())?;
    let hash = crate::digest_util::sha256_hex(&canonical);
    let dir = PathBuf::from(root).join(&hash[..12]);
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let file = dir.join("bundle.json");
    std::fs::write(&file, canonical).map_err(|error| error.to_string())?;
    Ok(file)
}
