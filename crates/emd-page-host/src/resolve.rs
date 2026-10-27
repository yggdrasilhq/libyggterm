// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Dependency resolution with actionable failures. Implementation
//! dependencies resolve against the LOCAL component workspace (the host IS
//! the workspace that supplies the components). Design dependencies resolve
//! against the manifest's pinned token-profile digests, and the resolved
//! digest joined to the page content digest joins evidence provenance.
//! Anything unresolved names the manifest key and the expected shape.

use crate::HostManifest;
use crate::Page;

/// The workspace version this host's components build against — the pin
/// `workspace:<semver>` resolves against.
pub const HOST_WORKSPACE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The component wire version the resolved pages render through.
pub const WIRE_VERSION: u16 = emd_renderer::components::COMPONENT_VERSION;

#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub crate_name: String,
    pub revision: String,
    pub workspace_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    /// Resolved; its identity joins evidence provenance.
    Ok(Resolved),
    /// Unresolved — actionable, names the key and the expected shape.
    Failed(String),
}

/// Resolve every declared implementation dependency. Key/value pairs stay
/// in "key" position: the resolution REPORT names the failing key.
pub fn resolve_implementation(manifest: &HostManifest) -> Vec<(String, Resolution)> {
    manifest
        .dependencies
        .implementation
        .iter()
        .enumerate()
        .map(|(index, dep)| {
            let key = format!("page.host.json: dependencies.implementation[{index}]");
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

/// Resolve the design (token) dependency. The resolved value is the digest
/// provenance joins: page content digest + pinned profile digest.
pub fn resolve_design(manifest: &HostManifest, page: &Page) -> Option<(String, Resolution)> {
    let dep = manifest.dependencies.design.as_ref()?;
    let resolved_digest = design_token_digest(page, dep);
    Some(match manifest.design.profiles.get(&dep.profile) {
        None => (
            "page.host.json: dependencies.design.profile".to_string(),
            Resolution::Failed(format!(
                "\"{}\" is not a pinned profile — declare it under design.profiles or fix the key; the page renders with the CONSUMING app's semantic tokens (the inheritance chain), never the originating brand's",
                dep.profile
            )),
        ),
        Some(pinned) if pinned != &dep.digest => (
            "page.host.json: dependencies.design.digest".to_string(),
            Resolution::Failed(format!(
                "{:?} does not match pinned profile {:?} ({pinned:?}) — regenerate the dependency digest declaring a NEW page revision, never update the pin silently",
                dep.digest, dep.profile
            )),
        ),
        Some(_) => (
            "page.host.json: dependencies.design".to_string(),
            Resolution::Ok(Resolved {
                crate_name: "design".to_string(),
                revision: format!("profile:{}", dep.profile),
                workspace_version: resolved_digest,
            }),
        ),
    })
}

/// The resolved token digest joins the page identity and the pinned
/// profile — one string so an evidence bundle has ONE digest to bind.
pub fn design_token_digest(page: &Page, design: &crate::manifest::DesignDep) -> String {
    crate::page::sha256_hex(
        format!(
            "page-tokens:content={};profile={};pin={}",
            page.content_sha256(),
            design.profile,
            design.digest
        )
        .as_bytes(),
    )
}

/// The full resolution table.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolutions {
    pub implementation: Vec<(String, Resolution)>,
    pub design: Option<(String, Resolution)>,
}

impl Resolutions {
    pub fn failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        for (_, resolution) in &self.implementation {
            if let Resolution::Failed(message) = resolution {
                failures.push(message.clone());
            }
        }
        if let Some((key, Resolution::Failed(message))) = &self.design {
            failures.push(format!("{key}: {message}"));
        }
        failures
    }

    pub fn ok(&self) -> bool {
        self.failures().is_empty()
    }

    pub fn all(&self) -> Vec<&(String, Resolution)> {
        self.implementation
            .iter()
            .chain(self.design.iter())
            .collect()
    }
}

/// Resolve everything the manifest declares.
pub fn resolve_all(manifest: &HostManifest, page: &Page) -> Resolutions {
    Resolutions {
        implementation: resolve_implementation(manifest),
        design: resolve_design(manifest, page),
    }
}
