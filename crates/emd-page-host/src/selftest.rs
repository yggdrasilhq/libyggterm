// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The planted-defect selftest (the false-acceptance guard), EMD ARM: the
//! plant vocabulary and the null-slot analytical probe live here; the
//! selftest engine and the comparison judge live in the portable contract
//! (`fixture_host::selftest`). Same guarantee as before: ref-vs-ref CLEAN
//! first, then every DECLARED plant detected through the common contract.

use crate::page::{Page, ParsedPage};
use fixture_host::manifest::HostManifest;
use fixture_host::selftest::{ArmPlants, SelftestReport};
use fixture_host::ScenarioHost;

/// The emd arm's plants; ids in `manifest.selftest.plants`.
pub fn arm_plants() -> ArmPlants {
    ArmPlants {
        plants: vec![
            ("null-to-zero", plant_null_to_zero_str),
            ("tone-drift", plant_tone_drift),
            ("version-skip", plant_version_skip),
        ],
        slot_probe: Some(slot_probe),
    }
}

/// Running the emd selftest over a page (the ecosystem spelling).
pub fn run_selftest(page: &Page, manifest: &HostManifest) -> SelftestReport {
    for declared in &manifest.selftest.plants {
        if !arm_plants().plants.iter().any(|(id, _)| id == declared) {
            return SelftestReport {
                ref_vs_ref: crate::bundle::Comparison {
                    verdict: "UNVERIFIED".to_string(),
                    findings: vec![format!(
                        "selftest UNVERIFIED: manifest.selftest.plants declares unknown plant {declared:?}; the emd arm's vocabulary is {:?}",
                        supported_plants()
                    )],
                },
                plants: Vec::new(),
                detected: 0,
                declared_plants: manifest.selftest.plants.len(),
                passed: false,
            };
        }
    }
    let page_path = page.path.clone();
    let mount = |path: &str, source: &str| -> Result<ScenarioHost<ParsedPage>, fixture_host::HostError> {
        let subject_path = if path.is_empty() {
            page_path.clone()
        } else {
            format!("{path}{}", page_path.split('/').last().unwrap_or("page.emd"))
        };
        mount_scene(&subject_path, source, &manifest.clone())
    };
    fixture_host::selftest::run_selftest(&page.source, manifest, &mount, &arm_plants())
}

/// Mount a scenario scene directly through the arm (parse + portable load)
/// — the selftest's candidate path; the CLI's `PageHost::load` adds the
/// wrapper identity on top of exactly this function.
pub fn mount_scene(
    path: &str,
    source: &str,
    manifest: &HostManifest,
) -> Result<ScenarioHost<ParsedPage>, fixture_host::HostError> {
    if let Err(message) = manifest.validate() {
        return Err(fixture_host::HostError::Manifest(message));
    }
    let page = Page {
        path: path.to_string(),
        source: source.to_string(),
        revision: manifest.revision.clone(),
    };
    let parsed = crate::page::parse_page(&page)
        .map_err(|error| fixture_host::HostError::Mount(error.message.clone()))?;
    ScenarioHost::load(page.path, manifest.clone(), parsed)
}

/// A missing observation silently coerced to zero — THE dangerous
/// analytical defect: it converts evidence gaps into plotted data.
pub fn plant_null_to_zero_str(source: &str) -> Option<String> {
    // Three spellings the grammar admits: a plot `y` slot (pretty or
    // compact) and a bare sparkline `values` entry. The plant applies to
    // whichever the source uses — cosmetics must not decide whether the
    // dangerous coercion is testable.
    if let Some(planted) = replace_first(source, "\"y\": null", "\"y\": 0.0") {
        return Some(planted);
    }
    if let Some(planted) = replace_first(source, "\"y\":null", "\"y\":0.0") {
        return Some(planted);
    }
    replace_after(
        source,
        "\"values\":[",
        &|tail: &str| match tail.find("null") {
            Some(index) => Some(replace_at(tail, index, "null", "0.0")),
            None => None,
        },
    )
}

/// A metric tone drifting — the colorblind-safe state vocabulary violated
/// (warning shows as critical / good shows as neutral). Simplest textual
/// tone mutation.
pub fn plant_tone_drift(source: &str) -> Option<String> {
    replace_first(source, "\"tone\":\"good\"", "\"tone\":\"warning\"")
}

/// An unsupported wire version silently downgraded — the failure this
/// campaign exists to prevent in the OTHER direction (a host accepting a
/// foreign grammar). Detected at validation, not digest.
pub fn plant_version_skip(source: &str) -> Option<String> {
    replace_first(source, "\"version\":1", "\"version\":2")
}

/// The null-slot analytical probe: how many missing-observation slots a
/// source carries (Plots' y + Sparklines' values, recursively). Legacy
/// name kept: `plant_null_to_zero` of the first revision.
pub fn slot_probe(source: &str) -> usize {
    let page = Page {
        path: "probe".to_string(),
        source: source.to_string(),
        revision: None,
    };
    match crate::page::parse_page(&page) {
        Ok(parsed) => parsed.components.iter().map(|c| c.null_slots).sum(),
        Err(_) => 0,
    }
}

pub fn replace_first(source: &str, needle: &str, replacement: &str) -> Option<String> {
    let index = source.find(needle)?;
    let mut out = source.to_string();
    out.replace_range(index..index + needle.len(), replacement);
    Some(out)
}

/// Replace the first `null` that appears AFTER an opener — the
/// sparkline/series spelling of a missing observation.
fn replace_after(
    source: &str,
    opener: &str,
    edit: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    let opener_index = source.find(opener)?;
    let tail = &source[opener_index + opener.len()..];
    let edited_tail = edit(tail)?;
    let mut edited = source.to_string();
    edited.replace_range(opener_index + opener.len().., &edited_tail);
    Some(edited)
}

/// Replace one exact range by index (used with `find` above).
fn replace_at(source: &str, index: usize, needle: &str, replacement: &str) -> String {
    let mut out = source.to_string();
    out.replace_range(index..index + needle.len(), replacement);
    out
}

// (kept lean: mutation is where correctness lives; a fuzzer could join
// later under the same contract)
// (kept lean: mutation is where correctness lives; a fuzzer could join
// later under the same contract)
pub fn supported_plants() -> Vec<&'static str> {
    arm_plants()
        .plants
        .iter()
        .map(|(id, _)| *id)
        .collect()
}
