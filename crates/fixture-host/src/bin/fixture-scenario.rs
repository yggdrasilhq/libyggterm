// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! `fixture-scenario` — the plain arm's CLI. One entry command per task,
//! the SAME verdict/exit-code discipline as `emd-page`:
//!
//!   fixture-scenario check    <scenario.dir>   manifest + mount validation
//!   fixture-scenario run      <scenario.dir> [--rounds N]
//!                             N fresh-reset rounds; every round mounts the
//!                             launcher fresh, resets, observes; digests must
//!                             agree (determinism).
//!   fixture-scenario selftest <scenario.dir>   planted defects judged
//!
//! Exit codes: 0 pass · 1 failure (MISMATCH/UNVERIFIED) · 2 usage · 3 I/O.

use fixture_host::bundle::Comparison;
use fixture_host::plain::PlainScene;
use fixture_host::{Digests, HostError, HostManifest, ScenarioHost};
use std::path::PathBuf;

const DRIVER: &str = concat!("fixture-scenario ", env!("CARGO_PKG_VERSION"));

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(match cli(&args) {
        Ok(code) => code,
        Err(Fail(message)) => {
            eprintln!("fixture-scenario: {message}");
            3
        }
    });
}

struct Fail(String);

impl From<HostError> for Fail {
    fn from(error: HostError) -> Self {
        Fail(error.to_string())
    }
}

fn cli(args: &[String]) -> Result<i32, Fail> {
    let Some(verb) = args.get(1).cloned() else {
        eprintln!("usage: fixture-scenario <check|run|selftest> <scenario.dir> [--rounds N]");
        return Ok(2);
    };
    let mut dir: Option<String> = None;
    let mut rounds: usize = 2;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--rounds" | "-n" => {
                i += 1;
                match args.get(i).and_then(|value| value.parse().ok()) {
                    Some(parsed) => rounds = parsed,
                    None => return Err(Fail("--rounds needs a number".to_string())),
                }
            }
            arg if !arg.starts_with("--") => {
                if dir.is_none() {
                    dir = Some(arg.to_string());
                } else {
                    eprintln!("unexpected positional argument: {arg:?}");
                    return Ok(2);
                }
            }
            unknown => {
                eprintln!("unknown flag: {unknown}");
                return Ok(2);
            }
        }
        i += 1;
    }
    let Some(dir_arg) = dir else {
        eprintln!("a scenario directory is required");
        return Ok(2);
    };
    let dir_path = resolve_dir(&dir_arg);
    let manifest_path = std::path::Path::new(&dir_path).join(fixture_host::SCENARIO_MANIFEST_FILENAME);
    let text = std::fs::read_to_string(&manifest_path).map_err(|error| {
        Fail(format!(
            "unreadable manifest {}: {error} (a plain-arm scenario carries {} beside the launcher)",
            manifest_path.display(),
            fixture_host::SCENARIO_MANIFEST_FILENAME
        ))
    })?;
    let manifest: HostManifest = serde_json::from_str(&text).map_err(|error| {
        Fail(format!(
            "manifest at {} is not a valid HostManifest: {error}",
            manifest_path.display()
        ))
    })?;
    if manifest.declared_kind() != fixture_host::manifest::Kind::Plain {
        return Err(Fail(format!(
            "manifest kind must be \"plain\" for the fixture-scenario arm (declared: {:?}) — a plain launcher driven through the wrong arm is a wrong-mount, not a failure to guess away",
            manifest.kind
        )));
    }
    let declared_launch = manifest.launch.clone().ok_or_else(|| {
        Fail("manifest: plain scenarios must declare \"launch\" — the runner contract needs an entry command".to_string())
    })?;
    // The manifest declares the launcher relative to the scenario dir (a
    // repo-relative path stays honest in a clean checkout); resolve tokens
    // against the scenario dir when the shell cwd cannot find them.
    let launch = resolve_launch(&dir_path, &declared_launch);
    match verb.as_str() {
        "check" => check(&launch, manifest),
        "run" => run(&launch, manifest, rounds),
        "selftest" => selftest(&launch, manifest),
        other => {
            eprintln!("fixture-scenario: unknown verb {other:?}; only check|run|selftest");
            Ok(2)
        }
    }
}

fn resolve_dir(dir: &str) -> PathBuf {
    std::path::PathBuf::from(dir).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(dir))
}

/// Resolve a launcher path relative to the scenario dir (never silently:
/// an unresolved launcher keeps its declared shape and fails at
/// `capabilities` with the named file).
fn resolve_launch(dir: &std::path::Path, launch: &str) -> String {
    launch
        .split_whitespace()
        .map(|token| {
            let direct = PathBuf::from(token);
            let as_scenariodir = dir.join(token);
            // A launch may be declared crate-relative ("corpus/…") while
            // the scenario dir IS the corpus dir — try the dir's parent.
            let as_sibling = dir.parent().unwrap_or(dir).join(token);
            if direct.exists() || token.starts_with('/') {
                token.to_string()
            } else if as_scenariodir.exists() {
                as_scenariodir.canonicalize().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| token.to_string())
            } else if as_sibling.exists() {
                as_sibling.canonicalize().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| token.to_string())
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Mount the launcher fresh and observe its scene (the plain arm's
/// mounting contract).
fn mount(launch: &str, manifest: &HostManifest) -> Result<ScenarioHost<PlainScene>, HostError> {
    // Capabilities gate FIRST — a launcher that cannot speak the contract
    // is a wrong-mount error, never an empty observation.
    PlainScene::check_capabilities(launch).map_err(|error| HostError::Mount(error.to_string()))?;
    PlainScene::reset_fixture(launch).map_err(|error| HostError::Mount(error.to_string()))?;
    let scene = PlainScene::observe(launch).map_err(|error| HostError::Mount(error.to_string()))?;
    // The manifest-word cross-check: the app's OWN reported fixture must
    // reproduce the DECLARED initial — the manifest is the boundary, not
    // the app's word.
    let findings = fixture_host::plain::fixture_cross_check(manifest, &scene.fixture);
    if !findings.is_empty() {
        return Err(HostError::Fixture(findings.join("; ")));
    }
    ScenarioHost::load(format!("launch:{launch}"), manifest.clone(), scene)
}

fn check(launch: &str, manifest: HostManifest) -> Result<i32, Fail> {
    let value = match mount(launch, &manifest) {
        Ok(host) => {
            let findings = host.resolutions.failures();
            serde_json::json!({
                "verb": "check",
                "launch": launch,
                "valid": findings.is_empty() && host.ready().unwrap_or(false),
                "item_count": host.digests().map(|d| d.components.len()).unwrap_or(0),
                "findings": findings,
            })
        }
        Err(error) => serde_json::json!({
            "verb": "check",
            "launch": launch,
            "valid": false,
            "findings": [error.to_string()],
        }),
    };
    println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
    Ok(i32::from(
        !value.get("valid").and_then(|v| v.as_bool()).unwrap_or(false),
    ))
}

/// N fresh-reset rounds: each round is a FRESH mount of the launcher, an
/// explicit reset, and an observation. Round 1 digests are the reference;
/// every later round must compare CLEAN (determinism + reset reproduction).
fn run(launch: &str, manifest: HostManifest, rounds: usize) -> Result<i32, Fail> {
    if rounds < 2 {
        return Err(Fail(
            "run needs at least TWO fresh-reset rounds (the acceptance contract)".to_string(),
        ));
    }
    let mut digests: Vec<Digests> = Vec::new();
    for round in 0..rounds {
        let host = mount(launch, &manifest)?;
        match host.ready() {
            Ok(true) => {
                digests.push(
                    host.digests()
                        .map_err(|error| Fail(format!("round {round}: digest computation failed: {error}")))?,
                );
            }
            _ => {
                return Err(Fail(format!(
                    "round {}: host NEVER READY — mount, items, or resolution invalid; not a silent UNVERIFIED pass",
                    round + 1
                )));
            }
        }
        let _unloaded = host.unload();
    }
    let mut verdict = "CLEAN".to_string();
    let mut findings: Vec<String> = Vec::new();
    for digest_set in digests.iter().skip(1) {
        let Comparison { verdict: compared, findings: compared_findings } =
            fixture_host::bundle::compare(&manifest, &digests[0], digest_set);
        if compared != "CLEAN" {
            verdict = compared;
            findings.extend(compared_findings);
        }
    }
    println!(
        "{}",
        serde_json::json!({
            "verb": "run",
            "rounds": rounds,
            "launch": launch,
            "verdict": verdict,
            "findings": findings,
        })
    );
    Ok(i32::from(verdict != "CLEAN"))
}

/// The plain arm's declared plant vocabulary: plants mutate the LAUNCHER
/// (a candidate launcher script — never the deployed app), and the selftest
/// mounts the candidate through the same contract.
fn plain_arm_plants() -> fixture_host::ArmPlants {
    fn launcher_path(launch: &str) -> Option<PathBuf> {
        // The launcher token (skip a bare interpreter word like "sh /path"):
        // the first token that exists as an executable path.
        for token in launch.split_whitespace() {
            let path = PathBuf::from(token);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
    fn candidate_of(source: &str, needle: &str, replacement: &str, stem: &str) -> Option<String> {
        if !source.contains(needle) {
            return None;
        }
        let planted = source.replacen(needle, replacement, 1);
        let candidate = std::env::temp_dir().join(stem);
        std::fs::write(&candidate, &planted).ok()?;
        std::fs::set_permissions(&candidate, std::os::unix::fs::PermissionsExt::from_mode(0o755)).ok()?;
        Some(format!("sh {}", candidate.display()))
    }
    fn plant_item_drop(launch: &str) -> Option<String> {
        // A candidate launcher that UNDER-COUNTS the observed items — the
        // observation-losing defect class ("the data is fine, says the
        // app"). Detected through axis=bounds.
        let path = launcher_path(launch)?;
        let source = std::fs::read_to_string(&path).ok()?;
        candidate_of(&source, "include_badges=1", "include_badges=0", "fixture-item-drop-candidate.sh")
    }
    fn plant_initial_drift(launch: &str) -> Option<String> {
        // A candidate launcher whose reset reproduces DIFFERENT initial
        // fixture values — the reset-reproduction defect class; the
        // observed seed drift changes every derived node (axis=scene) and
        // the fixture cross-check names the reset lie.
        let path = launcher_path(launch)?;
        let source = std::fs::read_to_string(&path).ok()?;
        candidate_of(&source, "SEED_INITIAL=7", "SEED_INITIAL=9", "fixture-initial-drift-candidate.sh")
    }
    fixture_host::ArmPlants {
        plants: vec![("item-drop", plant_item_drop), ("initial-drift", plant_initial_drift)],
        slot_probe: None,
    }
}

fn validate_declared_plants(
    manifest: &HostManifest,
    arm_plants: &fixture_host::ArmPlants,
) -> Result<(), String> {
    for declared in &manifest.selftest.plants {
        if !arm_plants.plants.iter().any(|(id, _)| id == declared) {
            return Err(format!(
                "manifest: selftest.plants declares unknown plant {declared:?}; the plain arm's vocabulary is {:?}",
                arm_plants.plants.iter().map(|(id, _)| *id).collect::<Vec<_>>()
            ));
        }
    }
    Ok(())
}

fn selftest(launch: &str, manifest: HostManifest) -> Result<i32, Fail> {
    let arm_plants = plain_arm_plants();
    if let Err(findings) = validate_declared_plants(&manifest, &arm_plants) {
        return Err(Fail(findings));
    }
    // The selftest SOURCE is the launcher command itself — plants mutate
    // the LAUNCH string (a candidate launcher script), so candidates mount
    // through the same contract with their own identity.
    let mount = |_path: &str, launch: &str| mount(launch, &manifest);
    let report =
        fixture_host::selftest::run_selftest(launch, &manifest, &mount, &arm_plants);
    println!(
        "selftest: ref-vs-ref {} · plants detected {}/{} declared",
        report.ref_vs_ref.verdict, report.detected, report.declared_plants
    );
    for plant in &report.plants {
        println!(
            "  plant {:?}: planted={} detected={} finding={}",
            plant.id,
            plant.planted,
            plant.detected,
            plant.finding.as_deref().unwrap_or("<none>")
        );
    }
    for finding in &report.ref_vs_ref.findings {
        println!("  ref-vs-ref: {finding}");
    }
    Ok(i32::from(!report.passed))
}

