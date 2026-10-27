// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The EMD ARM: how a parsed emd page mounts as a portable scenario
//! (`fixture_host::Mounted`). Everything arm-specific lives here — the
//! markdown/fence walk, the component digests, the fence-index evidence
//! regions, and the resolution table against the LOCAL component
//! workspace — so the portable contract never grows a markdown dependency.

use crate::error::PageError;
use crate::page::ParsedPage;
use fixture_host::digest_util::sha256_hex;
use fixture_host::manifest::HostManifest;
use fixture_host::mounted::{ItemDigest, Mounted, validation_item};
use fixture_host::resolution::{Resolution, Resolutions, Resolved};
use serde_json::Value;

/// The workspace version this arm's components build against — the pin
/// `workspace:<semver>` resolves against.
pub const HOST_WORKSPACE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The component wire version the resolved pages render through.
pub const WIRE_VERSION: u16 = emd_renderer::components::COMPONENT_VERSION;

/// Resolve every declared implementation dependency (against the local
/// component workspace — the arm IS the workspace that supplies the
/// components). Key/value pairs stay in "key" position: the REPORT names
/// the failing key.
pub fn resolve_implementation(manifest: &HostManifest, manifest_name: &str) -> Vec<(String, Resolution)> {
    manifest
        .dependencies
        .implementation
        .iter()
        .enumerate()
        .map(|(index, dep)| {
            let key = format!("{manifest_name}: dependencies.implementation[{index}]");
            if dep.crate_name.trim().is_empty() {
                return (
                    key.clone(),
                    Resolution::Failed(format!(
                        "{key}.\"crate\": must name the page's owning component crate (expected e.g. \"emd-renderer\"; got {:?}) — the executable host renders the SAME component the product imports",
                        dep.crate_name
                    )),
                );
            }
            if let Some(version) = dep.revision.strip_prefix("workspace:") {
                if version != HOST_WORKSPACE_VERSION {
                    return (
                        key.clone(),
                        Resolution::Failed(format!(
                            "{key}.revision: pinned workspace revision {:?} does not match this workspace's host revision {:?} — the page would execute in a workspace it did not pin; declare the new revision, never silently widen the pin",
                            version, HOST_WORKSPACE_VERSION
                        )),
                    );
                }
                return (
                    key.clone(),
                    Resolution::Ok(Resolved {
                        crate_name: dep.crate_name.clone(),
                        revision: dep.revision.clone(),
                        workspace_version: HOST_WORKSPACE_VERSION.to_string(),
                    }),
                );
            }
            if let Some(version) = dep.revision.strip_prefix("component:") {
                return match version.parse::<u16>() {
                    Ok(version) if version == WIRE_VERSION => (
                        key.clone(),
                        Resolution::Ok(Resolved {
                            crate_name: dep.crate_name.clone(),
                            revision: dep.revision.clone(),
                            workspace_version: HOST_WORKSPACE_VERSION.to_string(),
                        }),
                    ),
                    Ok(version) => (
                        key.clone(),
                        Resolution::Failed(format!(
                            "{key}.revision: component wire version {version} unsupported; this renderer's wire version is {WIRE_VERSION} — a version mismatch may need a migration decision, not a silent reinterpretation"
                        )),
                    ),
                    Err(error) => (
                        key.clone(),
                        Resolution::Failed(format!(
                            "{key}.revision: pin must be \"workspace:<semver>\" or \"component:<wire-version-number>\"; got {:?} ({error})",
                            dep.revision
                        )),
                    ),
                };
            }
            (
                key.clone(),
                Resolution::Failed(format!(
                    "{key}.revision: pin must be \"workspace:<semver>\" or \"component:<wire-version-number>\"; got {:?}",
                    dep.revision
                )),
            )
        })
        .collect()
}

/// Resolve the design (token) dependency against the scenario's content
/// content digest + pinned profile digest, one string ONE digest to bind.
pub fn resolve_design(
    manifest: &HostManifest,
    content_sha256: &str,
    manifest_name: &str,
) -> Option<(String, Resolution)> {
    let dep = manifest.dependencies.design.as_ref()?;
    let resolved_digest = design_token_digest(content_sha256, dep);
    Some(match manifest.design.profiles.get(&dep.profile) {
        None => (
            format!("{manifest_name}: dependencies.design.profile"),
            Resolution::Failed(format!(
                "\"{}\" is not a pinned profile — declare it under design.profiles or fix the key; the page renders with the CONSUMING app's semantic tokens (the inheritance chain), never the originating brand's",
                dep.profile
            )),
        ),
        Some(pinned) if pinned != &dep.digest => (
            format!("{manifest_name}: dependencies.design.digest"),
            Resolution::Failed(format!(
                "{:?} does not match pinned profile {:?} ({pinned:?}) — regenerate the dependency digest declaring a NEW page revision, never update the pin silently",
                dep.digest, dep.profile
            )),
        ),
        Some(_) => (
            format!("{manifest_name}: dependencies.design"),
            Resolution::Ok(Resolved {
                crate_name: "design".to_string(),
                revision: format!("profile:{}", dep.profile),
                workspace_version: resolved_digest,
            }),
        ),
    })
}

/// The resolved token digest joins the content identity and the pinned
/// profile — one string so an evidence bundle has ONE digest to bind.
pub fn design_token_digest(content_sha256: &str, design: &crate::manifest::DesignDep) -> String {
    sha256_hex(
        format!(
            "page-tokens:content={};profile={};pin={}",
            content_sha256, design.profile, design.digest
        )
        .as_bytes(),
    )
}

impl Mounted for ParsedPage {
    fn path(&self) -> String {
        self.path.clone()
    }

    fn content_sha256(&self) -> String {
        self.content_sha256.clone()
    }

    fn design_witness(&self) -> Option<String> {
        None
    }

    fn is_valid(&self) -> bool {
        self.components.iter().all(|c| c.error.is_none())
    }

    fn item_digests(&self) -> Result<Vec<ItemDigest>, String> {
        let mut digests = Vec::new();
        for component in &self.components {
            match &component.component {
                Some(document) => {
                    let digest = crate::digest::component_digest(&document.component)
                        .map_err(|error| error.message)?;
                    digests.push(ItemDigest {
                        index: component.location.fence_index,
                        kind: component.kind.clone().unwrap_or_default(),
                        digest,
                        error: None,
                    });
                }
                None => digests.push(ItemDigest {
                    index: component.location.fence_index,
                    kind: component.kind.clone().unwrap_or_default(),
                    digest: "invalid".to_string(),
                    error: component.error.clone(),
                }),
            }
        }
        Ok(digests)
    }

    fn validation_items(&self) -> Vec<Value> {
        self.components
            .iter()
            .filter(|c| c.error.is_some())
            .map(|c| {
                validation_item(
                    c.location.fence_index,
                    format!(
                        "fence {} at {}:{} path={}",
                        c.location.fence_index + 1,
                        c.location.line,
                        c.location.col,
                        c.location.block_path
                    ),
                    c.error.as_deref().unwrap_or("<unknown>"),
                )
            })
            .collect()
    }

    fn resolutions(&self, manifest: &HostManifest) -> Resolutions {
        Resolutions { implementation: resolve_implementation(manifest, "page.host.json"), design: resolve_design(manifest, &self.content_sha256, "page.host.json") }
    }

    fn null_slots(&self) -> Option<usize> {
        Some(self.components.iter().map(|c| c.null_slots).sum())
    }

    fn identity_component(&self) -> &'static str {
        "emd-component-page"
    }
}

fn _satisfy_type_alias(_error: &PageError) -> &'static str { "" }
