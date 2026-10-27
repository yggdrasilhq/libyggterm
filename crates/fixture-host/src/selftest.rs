// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The planted-defect selftest (the false-acceptance guard), ARMS-NEUTRAL.
//! The judge's common comparison contract is [`crate::bundle::compare`]; a
//! plant is DETECTED when that contract returns MISMATCH for the planted
//! candidate against the scenario's own reference render. A red selftest
//! suspends all scenario verdicts — exactly the zcode-tui wave-0
//! discipline, at scenario level.
//!
//! Plants mutate a SYNTHETIC candidate copy of the scenario SOURCE, never
//! the shipped artifact: the selftest runs on a candidate the ARM mounts
//! from mutated source through its normal path, and the manifest DECLARES
//! the plant vocabulary before any candidate is inspected. The arm supplies
//! its plant vocabulary as `(id, fn(source) -> Option<mutated_source>)` —
//! a plant that cannot apply is reported NOT PLANTED, never detected (a
//! no-op plant must not count as coverage).

use crate::bundle::{self};
use crate::host::{HostError, ScenarioHost};
use crate::mounted::Mounted;

/// The arm's declared plant vocabulary.
pub struct ArmPlants {
    /// (id, mutator): mutator returns the MUTATED SOURCE or None when the
    /// plant cannot apply to this scenario.
    pub plants: Vec<(&'static str, fn(&str) -> Option<String>)>,
    /// Optional semantic probe: the arm's count of missing-observation
    /// slots in a scenario source. Used for the null-slot delta finding;
    /// None = no probed surface, the probe is skipped, never faked.
    pub slot_probe: Option<fn(&str) -> usize>,
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
    /// Semantic probe enrichment: for data plants, the slot-count delta.
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

/// Run the full selftest for a scenario source + its manifest, through the
/// arm's mount closure and plant vocabulary.
pub fn run_selftest<M, F>(
    source: &str,
    manifest: &crate::manifest::HostManifest,
    mount: &F,
    arm: &ArmPlants,
) -> SelftestReport
where
    M: Mounted,
    F: Fn(&str, &str) -> Result<ScenarioHost<M>, HostError>,
{
    // Reference digests (the arm's own load path — never a side door).
    let base_host = match mount("", source) {
        Ok(host) => host,
        Err(error) => return all_unverified(format!("selftest UNVERIFIED: base mount failed: {error}")),
    };
    if !base_host.mounted.is_valid() {
        return all_unverified(
            "selftest UNVERIFIED: the scenario's own items must be valid before a selftest can mean anything".to_string(),
        );
    }
    let base_digests = match base_host.digests() {
        Ok(digests) => digests,
        Err(error) => return all_unverified(format!("selftest UNVERIFIED: base digests: {error}")),
    };

    // Ref-vs-ref determinism: the same scenario mounted again, digests
    // re-computed — CLEAN.
    let re_digests = match mount("", source) {
        Ok(host) => host.digests(),
        Err(error) => {
            return all_unverified(format!("selftest UNVERIFIED: second mount failed: {error}"));
        }
    }
    .map_err(|error| {
        all_unverified(format!("selftest UNVERIFIED: second digest pass: {error}"))
    });
    let re_digests = match re_digests {
        Ok(digests) => digests,
        Err(report) => return report,
    };
    let ref_vs_ref = bundle::compare(manifest, &base_digests, &re_digests);

    let base_slots = arm.slot_probe.map(|probe| probe(source));
    let mut plants = Vec::new();
    let mut detected = 0;
    for (id, apply) in &arm.plants {
        if !manifest.selftest.plants.iter().any(|declared| declared == id) {
            // Not declared — the selftest runs only DECLARED plants
            // (undeclared plants would widen the guarded surface silently).
            continue;
        }
        let Some(candidate_source) = apply(source) else {
            plants.push(PlantResult {
                id: id.to_string(),
                planted: false,
                detected: false,
                finding: Some(format!(
                    "plant {id:?} could not apply to this scenario — declare only plants this scenario actually exhibits; a selftest with silent no-op plants lies about its coverage"
                )),
                null_slot_delta: None,
            });
            continue;
        };
        // Candidate identity: a DISTINCT revision path, never the
        // scenario's own — provenance must never conflate candidate and
        // reference.
        let candidate = match mount(&format!("synthetic-candidate/plant-{id}/"), &candidate_source)
        {
            Ok(host) => host,
            Err(error) => {
                // An unmountable candidate — most "wipe the evidence"
                // plants land here. Detection = the contract JUDGE runs and
                // sees the validation failure; that IS a mismatch at the
                // validation layer, so record it as one.
                plants.push(PlantResult {
                    id: id.to_string(),
                    planted: true,
                    detected: true,
                    finding: Some(format!(
                        "plant {id:?} rejected at the validation/host layer: {error}"
                    )),
                    null_slot_delta: None,
                });
                detected += 1;
                continue;
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
                detected += 1;
                continue;
            }
        };
        let comparison = bundle::compare(manifest, &base_digests, &candidate_digests);
        let is_detected = comparison.verdict == "MISMATCH";
        let mut findings = comparison.findings.clone();
        // The ANALYTICAL reason (when the arm probes slots): a lost slot is
        // a gap converted to data — named beyond a bare digest mismatch.
        let candidate_slots = arm.slot_probe.map(|probe| probe(&candidate_source));
        let slots_delta = match (base_slots, candidate_slots) {
            (Some(base), Some(candidate)) => base.saturating_sub(candidate),
            _ => 0,
        };
        if is_detected {
            if slots_delta > 0 {
                findings.insert(
                    0,
                    format!(
                        "{id:?}: {slots_delta} missing-observation slot(s) coerced out — a null gap became data ({:?} → {:?})",
                        base_slots, candidate_slots
                    ),
                );
            }
            if findings.is_empty() {
                findings.push(format!(
                    "plant {id:?} altered the scenario digests through no declared mask"
                ));
            }
            detected += 1;
        }
        plants.push(PlantResult {
            id: id.to_string(),
            planted: true,
            detected: is_detected,
            finding: (!findings.is_empty()).then(|| findings.join("; ")),
            null_slot_delta: Some(slots_delta),
        });
    }
    let declared_plants = manifest.selftest.plants.len();
    SelftestReport {
        passed: ref_vs_ref.verdict == "CLEAN" && detected == declared_plants,
        ref_vs_ref,
        detected,
        declared_plants,
        plants,
    }
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
