// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The planted-defect selftest (the false-acceptance guard). The judge's
//! common comparison contract is [`crate::bundle::compare`]; a plant is
//! DETECTED when that contract returns MISMATCH for the planted candidate
//! against the page's own reference render. A red selftest suspends all
//! page verdicts — exactly the zcode-tui wave-0 discipline, at page level.
//!
//! Plants mutate a SYNTHETIC candidate copy of the page, never the shipped
//! source: the selftest runs on a copy, and the manifest DECLARES the plant
//! vocabulary it uses before any candidate is inspected.

use crate::Page;
use crate::bundle::{self};
use crate::host::Digests;

/// Semantic plants; ids in `manifest.selftest.plants`. Each returns the
/// MUTATED SOURCE or None when the plant cannot apply to this page (the
/// selftest reports that as NOT PLANTED, not as detected — a no-op would
/// otherwise be a false acceptance).
pub const PLANTS: &[(&str, fn(&Page) -> Option<String>)] = &[
    ("null-to-zero", plant_null_to_zero),
    ("tone-drift", plant_tone_drift),
    ("version-skip", plant_version_skip),
];

/// A missing observation silently coerced to zero — THE dangerous
/// analytical defect: it converts evidence gaps into plotted data.
pub fn plant_null_to_zero(page: &Page) -> Option<String> {
    // Three spellings the grammar admits: a plot `y` slot (pretty or
    // compact) and a bare sparkline `values` entry. The plant applies to
    // whichever the fixture uses — cosmetics must not decide whether the
    // dangerous coercion is testable.
    if let Some(planted) = replace_first(page, "\"y\": null", "\"y\": 0.0") {
        return Some(planted);
    }
    if let Some(planted) = replace_first(page, "\"y\":null", "\"y\":0.0") {
        return Some(planted);
    }
    replace_after(
        page,
        "\"values\":[",
        &|source: &Page| match source.source.find("null") {
            Some(index) => Some(replace_at(&source.source, index, "null", "0.0")),
            None => None,
        },
    )
}

/// A metric tone drifting — the colorblind-safe state vocabulary violated
/// (warning shows as critical / good shows as neutral). Simplest textual
/// tone mutation.
pub fn plant_tone_drift(page: &Page) -> Option<String> {
    replace_first(page, "\"tone\":\"good\"", "\"tone\":\"warning\"")
}

/// An unsupported wire version silently downgraded — the failure this
/// campaign exists to prevent in the OTHER direction (a host accepting a
/// foreign grammar). Detected at validation, not digest.
pub fn plant_version_skip(page: &Page) -> Option<String> {
    replace_first(page, "\"version\":1", "\"version\":2")
}

fn replace_first(page: &Page, needle: &str, replacement: &str) -> Option<String> {
    let index = page.source.find(needle)?;
    let mut source = page.source.clone();
    source.replace_range(index..index + needle.len(), replacement);
    Some(source)
}

/// Replace the first `null` that appears AFTER a `values:[` opener — the
/// sparkline/series spelling of a missing observation.
fn replace_after(
    page: &Page,
    opener: &str,
    edit: &dyn Fn(&Page) -> Option<String>,
) -> Option<String> {
    let opener_index = page.source.find(opener)?;
    let tail_source: String = page.source[opener_index + opener.len()..].to_string();
    let tail_page: Page = Page {
        path: page.path.clone(),
        source: tail_source,
        revision: None,
    };
    let edited_tail = edit(&tail_page)?;
    let mut edited = page.source.clone();
    edited.replace_range(opener_index + opener.len().., &edited_tail);
    Some(edited)
}

/// Replace one exact range by index (used with `find` above).
fn replace_at(source: &str, index: usize, needle: &str, replacement: &str) -> String {
    let mut out = source.to_string();
    out.replace_range(index..index + needle.len(), replacement);
    out
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PlantResult {
    pub id: String,
    /// Could the plant actually apply (a no-op plant would be a false
    /// pass — it must NOT count as detected).
    pub planted: bool,
    pub detected: bool,
    /// The finding the common contract produced (spec-grade or None).
    pub finding: Option<String>,
    /// Semantic probe enrichment: for data plants, the null-slot delta.
    pub null_slot_delta: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SelftestReport {
    /// Ref-vs-ref determinism FIRST — a dirty determinism gate voids the
    /// plant results (same discipline as the web adapter: CLEAN before
    /// any ref-vs-candidate comparison means anything).
    pub ref_vs_ref: bundle::Comparison,
    pub plants: Vec<PlantResult>,
    pub detected: usize,
    pub declared_plants: usize,
    /// True only when determinism is clean AND every declared plant is
    /// both planted and detected.
    pub passed: bool,
}

/// Run the full selftest for a page + its manifest (already validated).
pub fn run_selftest(page: &Page, manifest: &crate::HostManifest) -> SelftestReport {
    // Reference parse + digests.
    let base_parsed = match crate::page::parse_page(page) {
        Ok(parsed) => parsed,
        Err(error) => {
            return all_unverified(format!("selftest UNVERIFIED: page does not parse: {error}"));
        }
    };
    if !base_parsed.is_valid() {
        return all_unverified("selftest UNVERIFIED: the page's own fences must be valid before a selftest can mean anything".to_string());
    }
    let host = match crate::PageHost::load(page.clone(), manifest.clone()) {
        Ok(host) => host,
        Err(error) => {
            return all_unverified(format!("selftest UNVERIFIED: host load failed: {error}"));
        }
    };
    let base_digests = match host.digests() {
        Ok(digests) => digests,
        Err(error) => return all_unverified(format!("selftest UNVERIFIED: base digests: {error}")),
    };
    // Ref-vs-ref: the same page, re-parsed, digests re-computed — CLEAN.
    let reparsed = match crate::page::parse_page(page) {
        Ok(parsed) => parsed,
        Err(error) => {
            return all_unverified(format!("selftest UNVERIFIED: reparse failed: {error}"));
        }
    };
    let re_digests = match host_digests_for(manifest, page) {
        Ok(digests) => digests,
        Err(error) => {
            return all_unverified(format!("selftest UNVERIFIED: second digest pass: {error}"));
        }
    };
    let _ = reparsed;
    let ref_vs_ref = bundle::compare(manifest, &base_digests, &re_digests);

    // Plants: candidate = mutated SOURCE, judged through the same contract.
    let mut plants = Vec::new();
    let mut detected = 0;
    for (id, apply) in PLANTS {
        if !manifest
            .selftest
            .plants
            .iter()
            .any(|declared| declared == id)
        {
            // Not declared — the selftest runs only DECLARED plants
            // (undeclared plants would widen the guarded surface silently).
            continue;
        }
        mutated(
            page,
            &id,
            *apply,
            &base_parsed,
            manifest,
            &base_digests,
            &mut plants,
            &mut detected,
        );
    }
    let declared_plants = manifest.selftest.plants.len();
    if std::env::var("EMD_PAGE_HOST_SELFTEST_DEBUG").is_ok() {
        eprintln!(
            "selftest: detected={detected} declared={declared_plants} ref={}",
            ref_vs_ref.verdict
        );
    }
    SelftestReport {
        passed: ref_vs_ref.verdict == "CLEAN" && detected == declared_plants,
        ref_vs_ref,
        detected,
        declared_plants,
        plants,
    }
}

/// Mismatch semantics shared by every plant: the contract judge, PLUS the
/// semantic probes (an honest digest change is one signal; a null-slot
/// regression is the ANALYTICAL reason the finding names).
fn mutated(
    page: &Page,
    id: &str,
    apply: fn(&Page) -> Option<String>,
    base_parsed: &crate::ParsedPage,
    manifest: &crate::HostManifest,
    base_digests: &Digests,
    plants: &mut Vec<PlantResult>,
    detected: &mut usize,
) {
    let Some(source) = apply(page) else {
        plants.push(PlantResult {
            id: id.to_string(),
            planted: false,
            detected: false,
            finding: Some(format!(
                "plant {id:?} could not apply to this page — declare only plants this page actually exhibits; a selftest with silent no-op plants lies about its coverage"
            )),
            null_slot_delta: None,
        });
        return;
    };
    // Candidate identity: a DISTINCT revision, never the page's own —
    // provenance must never conflate candidate and reference.
    let candidate_page = Page {
        path: format!("synthetic-candidate/plant-{id}/{}", page.path),
        source,
        revision: Some(format!("planted-{id}")),
    };
    let candidate = match crate::PageHost::load(candidate_page.clone(), manifest.clone()) {
        Ok(host) => host,
        Err(error) => {
            // An unloadable candidate — most “wipe the evidence” plants land
            // here. Detection = the contract JUDGE runs and sees the page
            // error; that IS a mismatch at the validation layer, so record
            // it as one.
            plants.push(PlantResult {
                id: id.to_string(),
                planted: true,
                detected: true,
                finding: Some(format!(
                    "plant {id:?} rejected at the validation/host layer: {error}"
                )),
                null_slot_delta: None,
            });
            *detected += 1;
            return;
        }
    };
    let candidate_digests = match candidate.digests() {
        Ok(digests) => digests,
        Err(error) => {
            plants.push(PlantResult {
                id: id.to_string(),
                planted: true,
                detected: true,
                finding: Some(format!(
                    "plant {id:?} undigible: {error} — treated as detected MISMATCH"
                )),
                null_slot_delta: None,
            });
            *detected += 1;
            return;
        }
    };
    let comparison = bundle::compare(manifest, base_digests, &candidate_digests);
    let is_detected = comparison.verdict == "MISMATCH";
    let mut findings = comparison.findings.clone();
    // The ANALYTICAL reason: a lost null slot is a gap converted to data —
    // always named when detected, beyond a bare digest mismatch.
    let base_slots: usize = base_parsed.components.iter().map(|c| c.null_slots).sum();
    let candidate_slots: usize = candidate
        .parsed
        .components
        .iter()
        .map(|c| c.null_slots)
        .sum();
    // The null-regression probe is meaningful ONLY for a data plant: an
    // invalid-fence plant zeroes the parse the same way a validation error
    // zeroes any count — do not bleed one signal into the other.
    let data_plant = *id == *"null-to-zero";
    if is_detected {
        if data_plant && candidate_slots < base_slots {
            findings.insert(
                0,
                format!(
                    "{id:?}: {} missing-observation slot(s) coerced out — a null gap became data ({base_slots} → {candidate_slots})",
                    base_slots - candidate_slots
                ),
            );
        }
        if findings.is_empty() {
            findings.push(format!(
                "plant {id:?} altered the page digests through no declared mask"
            ));
        }
        *detected += 1;
    }
    plants.push(PlantResult {
        id: id.to_string(),
        planted: true,
        detected: is_detected,
        finding: (!findings.is_empty()).then(|| findings.join("; ")),
        null_slot_delta: Some(base_slots.saturating_sub(candidate_slots)),
    });
}

fn all_unverified(reason: String) -> SelftestReport {
    SelftestReport {
        ref_vs_ref: bundle::Comparison {
            verdict: "UNVERIFIED".to_string(),
            findings: vec![reason],
        },
        plants: Vec::new(),
        detected: 0,
        declared_plants: 0,
        passed: false,
    }
}

fn host_digests_for(manifest: &crate::HostManifest, page: &Page) -> Result<Digests, String> {
    crate::PageHost::load(page.clone(), manifest.clone())
        .map_err(|error| error.to_string())?
        .digests()
}

// (kept lean: mutation is where correctness lives; a fuzzer could join
// later under the same contract)
pub fn supported_plants() -> Vec<&'static str> {
    PLANTS.iter().map(|(id, _)| *id).collect()
}
