// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The PLAIN arm: a fixture APP (the miniapp pattern — chain 0029 Q3:
//! "falsify against ordinary fixture apps") driven through the RUNNER
//! COMMAND CONTRACT. The scenario manifest declares the launch command;
//! the app is invoked as
//!
//! ```text
//! <launch> capabilities        → {"protocol":1,"verbs":[…]}
//! <launch> reset_fixture       → restore the app's declared initial state
//! <launch> observe             → {"fixture":{…},"items":[{"kind":…,"node":…}…]}
//! ```
//!
//! The scene truth of a plain fixture is its OBSERVATION: canonical JSON
//! nodes the host digests deterministically (the same sha256 discipline
//! the emd arm applies to renders). Reset = `reset_fixture` + a fresh
//! observe reproducing the declared initial fixture. Declared masks and
//! mutable keys come from the SAME manifest model — nothing about the
//! plain arm is a second contract.

use crate::digest_util::sha256_hex;
use crate::mounted::{ItemDigest, Mounted, validation_item};
use crate::resolution::{Resolution, Resolutions, Resolved};
use serde_json::Value;

pub const PROTOCOL_REV: u64 = 1;
pub const PLAIN_COMPONENT_IDENTITY: &str = "plain-fixture-app";

#[derive(Debug, Clone, PartialEq)]
pub enum PlainError {
    /// The launcher could not be run or its output was not the contract.
    Launch(String),
    /// Capabilities mismatch — the app speaks a different contract rev or
    /// is missing the required verbs.
    Capabilities(String),
}

/// Mount a plain fixture app and observe its scene.
#[derive(Debug, Clone, PartialEq)]
pub struct PlainScene {
    /// The launch command (the subject identity of a plain arm run).
    pub launch: String,
    /// The raw observation capture (its sha256 is the scenario source id).
    pub observation: String,
    pub fixture: serde_json::Map<String, serde_json::Value>,
    pub items: Vec<(String, String)>, // (kind, canonical node JSON)
    pub launch_error: Option<String>,
}

impl PlainScene {
    /// Run the app's observation pass through the launch command. Blocking;
    /// the runner contract is synchronous on purpose (a scenario's
    /// interaction sequence is replayable — observe is ONE step).
    pub fn observe(launch: &str) -> Result<Self, PlainError> {
        let (code, stdout, stderr) = run_verb(launch, "observe").map_err(PlainError::Launch)?;
        if code != 0 {
            return Ok(PlainScene {
                launch: launch.to_string(),
                observation: format!("exit={code} stderr={}", stderr.trim_end()),
                fixture: Default::default(),
                items: Vec::new(),
                launch_error: Some(format!(
                    "observe: the fixture app exited with code {code} — an app-level failure the host reports through item 1; it is never silently swallowed"
                )),
            });
        }
        let parsed: Value = serde_json::from_str(&stdout).map_err(|error| {
            PlainError::Launch(format!("observe output is not contract JSON: {error}"))
        })?;
        let fixture = parsed
            .get("fixture")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut items = Vec::new();
        for item in parsed
            .get("items")
            .and_then(Value::as_array)
            .unwrap_or(&vec![])
        {
            let kind = item
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("untyped")
                .to_string();
            let node = item.get("node").cloned().unwrap_or(Value::Null);
            items.push((kind, serde_json::to_string(&node).unwrap_or_default()));
        }
        Ok(PlainScene {
            launch: launch.to_string(),
            observation: stdout,
            fixture,
            items,
            launch_error: None,
        })
    }

    /// Capabilities check: the app must speak contract rev 1 with the
    /// required verbs. A mismatch is actionable, never interpreted.
    pub fn check_capabilities(launch: &str) -> Result<(), PlainError> {
        let (code, stdout, _) = run_verb(launch, "capabilities").map_err(|error| {
            PlainError::Capabilities(format!("capabilities launch failed: {error}"))
        })?;
        if code != 0 {
            return Err(PlainError::Capabilities(format!(
                "capabilities exited with code {code} — the fixture app is not speakable through the runner contract"
            )));
        }
        let caps: Value = serde_json::from_str(&stdout).map_err(|error| {
            PlainError::Capabilities(format!("capabilities output is not contract JSON: {error}"))
        })?;
        let speakers = caps.get("protocol").and_then(Value::as_u64);
        if speakers != Some(PROTOCOL_REV) {
            return Err(PlainError::Capabilities(format!(
                "capabilities.protocol={:?} unsupported; this host implements protocol rev {PROTOCOL_REV}",
                speakers
            )));
        }
        let verbs = caps
            .get("verbs")
            .and_then(Value::as_array)
            .map(|verbs| verbs.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        for required in ["reset_fixture", "observe"] {
            if !verbs.iter().any(|v| v == &required) {
                return Err(PlainError::Capabilities(format!(
                    "capabilities.verbs does not include {required:?} — declared, never assumed"
                )));
            }
        }
        Ok(())
    }

    pub fn reset_fixture(launch: &str) -> Result<(), PlainError> {
        let (code, _, stderr) = run_verb(launch, "reset_fixture").map_err(PlainError::Launch)?;
        if code != 0 {
            return Err(PlainError::Launch(format!(
                "reset_fixture exited with code {code} (stderr: {})",
                stderr.trim_end()
            )));
        }
        Ok(())
    }
}

impl std::fmt::Display for PlainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlainError::Launch(message) => write!(f, "launch: {message}"),
            PlainError::Capabilities(message) => write!(f, "capabilities: {message}"),
        }
    }
}

impl std::error::Error for PlainError {}

impl Mounted for PlainScene {
    fn path(&self) -> String {
        format!("launch:{}", self.launch)
    }

    fn content_sha256(&self) -> String {
        sha256_hex(self.observation.as_bytes())
    }

    fn design_witness(&self) -> Option<String> {
        None
    }

    fn identity_component(&self) -> &'static str {
        PLAIN_COMPONENT_IDENTITY
    }

    fn is_valid(&self) -> bool {
        self.launch_error.is_none() && !self.items.is_empty() && !self.items.is_empty()
    }

    fn item_digests(&self) -> Result<Vec<ItemDigest>, String> {
        let mut digests = Vec::new();
        if let Some(message) = &self.launch_error {
            digests.push(ItemDigest {
                index: 0,
                kind: "launch-failure".to_string(),
                digest: sha256_hex(self.observation.as_bytes()),
                error: Some(message.clone()),
            });
            return Ok(digests);
        }
        for (index, (kind, node)) in self.items.iter().enumerate() {
            digests.push(ItemDigest {
                index,
                kind: kind.clone(),
                digest: sha256_hex(node.as_bytes()),
                error: None,
            });
        }
        Ok(digests)
    }

    fn validation_items(&self) -> Vec<Value> {
        self.launch_error
            .as_ref()
            .map(|message| vec![validation_item(0, self.path().clone(), message)])
            .unwrap_or_default()
    }

    fn resolutions(&self, manifest: &crate::manifest::HostManifest) -> Resolutions {
        // The plain arm resolves the launch command itself: a missing
        // launcher is an unresolved dependency, not an app-level failure.
        let key = "scenario.host.json: launch".to_string();
        let implementation: Vec<(String, Resolution)> = manifest
            .dependencies
            .implementation
            .iter()
            .enumerate()
            .map(|(index, dep)| {
                let dep_key = format!("scenario.host.json: dependencies.implementation[{index}]");
                match dep.revision.strip_prefix("script:") {
                    Some(_) => (
                        dep_key,
                        Resolution::Ok(Resolved {
                            crate_name: dep.crate_name.clone(),
                            revision: dep.revision.clone(),
                            workspace_version: PROTOCOL_REV.to_string(),
                        }),
                    ),
                    None => (
                        dep_key.clone(),
                        Resolution::Failed(format!(
                            "{dep_key}.revision: plain-fixture apps pin \"script:<n>\" (the launcher contract revision); got {:?}",
                            dep.revision
                        )),
                    ),
                }
            })
            .collect();
        let launch_ok = Some(PlainScene::observe(&self.launch).is_ok());
        Resolutions {
            implementation: implementation.into_iter().chain(
                vec![(
                    key,
                    if launch_ok == Some(true) {
                        Resolution::Ok(Resolved {
                            crate_name: "launch".to_string(),
                            revision: self.launch.clone(),
                            workspace_version: PROTOCOL_REV.to_string(),
                        })
                    } else {
                        Resolution::Failed(
                            "launch: the declared launcher does not speak the runner contract (capabilities/reset_fixture/observe)".to_string(),
                        )
                    },
                )],
            ).collect(),
            design: resolve_design_portable(manifest),
        }
    }
}

/// Design-dep resolution for anything arm-neutral: pinned profile digest
/// match (the token witness from SOURCE is the arm's option).
fn resolve_design_portable(
    manifest: &crate::manifest::HostManifest,
) -> Option<(String, Resolution)> {
    let dep = manifest.dependencies.design.as_ref()?;
    Some(match manifest.design.profiles.get(&dep.profile) {
        None => (
            "scenario.host.json: dependencies.design.profile".to_string(),
            Resolution::Failed(format!(
                "\"{}\" is not a pinned profile — declare it under design.profiles or fix the key; the scenario renders with the CONSUMING app's semantic tokens (the inheritance chain), never the originating brand's",
                dep.profile
            )),
        ),
        Some(pinned) if pinned != &dep.digest => (
            "scenario.host.json: dependencies.design.digest".to_string(),
            Resolution::Failed(format!(
                "{:?} does not match pinned profile {:?} ({pinned:?}) — regenerate the dependency digest declaring a NEW scenario revision, never update the pin silently",
                dep.digest, dep.profile
            )),
        ),
        Some(_) => (
            "scenario.host.json: dependencies.design".to_string(),
            Resolution::Ok(Resolved {
                crate_name: "design".to_string(),
                revision: format!("profile:{}", dep.profile),
                workspace_version: dep.digest.clone(),
            }),
        ),
    })
}

/// Fixture-state cross-check: the app's OWN reported fixture against the
/// manifest's declared initial — a mismatch is a finding (the manifest is
/// the boundary, not the app's word).
pub fn fixture_cross_check(
    manifest: &crate::manifest::HostManifest,
    observed: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let mut findings = Vec::new();
    for (key, declared) in &manifest.fixture.values {
        match observed.get(key) {
            Some(observed) if observed == declared => {}
            Some(observed) => findings.push(format!(
                "axis=fixture: observed fixture {key:?} = {observed} does not reproduce the declared initial {declared} — reset did not restore the manifest-declared state",
            )),
            None => findings.push(format!(
                "axis=fixture: observed fixture does not report declared key {key:?} — the app's observation drops declared state",
            )),
        }
    }
    findings
}

fn run_verb(launch: &str, verb: &str) -> Result<(i32, String, String), String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{launch} {verb}"))
        .output()
        .map_err(|error| format!("launching {launch:?}: {error}"))?;
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}
