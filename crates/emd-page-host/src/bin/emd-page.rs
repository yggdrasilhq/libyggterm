// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `emd-page` — the page-host CLI. One entry command per task:
//!
//!   emd-page check    <page.emd> [--manifest P]    validation report
//!   emd-page render   <page.emd> [--manifest P]    hashed evidence bundle
//!   emd-page run      <page.emd> [--rounds N]      N fresh-reset runs; all
//!                                     digests must agree (determinism)
//!   emd-page selftest <page.emd> [--manifest P]    planted defects judged
//!
//! Exit codes: 0 pass · 1 failure (MISMATCH/UNVERIFIED) · 2 usage · 3 I/O.

use emd_page_host::{
    Digests, HostError, HostManifest, Page, PageHost,
    bundle::{self, Comparison},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(match cli(&args) {
        Ok(code) => code,
        Err(Fail(message)) => {
            eprintln!("emd-page: {message}");
            3
        }
    });
}

struct Fail(String);

/// Merge the usage channel into `Fail` — same exit path, different arg shape.
fn cli(args: &[String]) -> Result<i32, Fail> {
    let Some(verb) = args.get(1).cloned() else {
        eprintln!(
            "usage: emd-page <check|render|run|selftest> <page.emd> [--manifest P] [--rounds N]"
        );
        return Ok(2);
    };
    let mut page_path: Option<String> = None;
    let mut manifest_path: Option<String> = None;
    let mut rounds: usize = 2;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" | "-m" => {
                i += 1;
                match args.get(i) {
                    Some(path) => manifest_path = Some(path.clone()),
                    None => return Err(Fail("--manifest needs a path".to_string())),
                }
            }
            "--rounds" | "-n" => {
                i += 1;
                match args.get(i).and_then(|value| value.parse().ok()) {
                    Some(parsed) => rounds = parsed,
                    None => return Err(Fail("--rounds needs a number".to_string())),
                }
            }
            arg if !arg.starts_with("--") => {
                if page_path.is_none() {
                    page_path = Some(arg.to_string());
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
    let Some(page_path) = page_path else {
        eprintln!("a page path is required");
        return Ok(2);
    };
    let source = match std::fs::read_to_string(&page_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("unreadable page {page_path}: {error}");
            return Ok(3);
        }
    };
    let manifest = load_manifest(manifest_path.as_deref(), &page_path)?;
    let page = Page {
        path: page_path,
        source,
        revision: manifest.revision.clone(),
    };
    match verb.as_str() {
        "check" => check(page, manifest),
        "render" => render(page, manifest),
        "run" => run(page, manifest, rounds),
        "selftest" => selftest(page, manifest),
        other => {
            eprintln!("emd-page: unknown verb {other:?}; only check|render|run|selftest");
            Ok(2)
        }
    }
}

fn load_manifest(explicit: Option<&str>, page_path: &str) -> Result<HostManifest, Fail> {
    let path = explicit.map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::path::Path::new(page_path)
            .parent()
            .map(|p| p.join(emd_page_host::MANIFEST_FILENAME))
            .unwrap_or_else(|| std::path::PathBuf::from(emd_page_host::MANIFEST_FILENAME))
    });
    if path.exists() {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| Fail(format!("unreadable manifest {}: {error}", path.display())))?;
        return serde_json::from_str(&text).map_err(|error| {
            Fail(format!(
                "manifest at {} is not a valid HostManifest: {error}",
                path.display()
            ))
        });
    }
    if explicit.is_some() {
        return Err(Fail(format!("manifest not found at {}", path.display())));
    }
    // No manifest beside the page: a prose page IS a legal host
    // (read-only, no declared behavior) — the DEFAULT manifest is used.
    Ok(HostManifest::default())
}

fn git_revision() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    Some(stdout.trim().to_string()).filter(|rev| !rev.is_empty())
}

fn load_host(page: Page, manifest: &HostManifest) -> Result<PageHost, Fail> {
    PageHost::load(page, manifest.clone()).map_err(|error| match &error {
        HostError::Manifest(message)
        | HostError::Mount(message)
        | HostError::Fixture(message)
        | HostError::Dependency(message) => Fail(message.clone()),
        other => Fail(other.to_string()),
    })
}

fn load_errors(host: &PageHost) -> Vec<String> {
    let mut findings = host
        .parsed
        .errors()
        .iter()
        .map(|error| error.finding(&host.parsed.path))
        .collect::<Vec<_>>();
    findings.extend(host.scenario.resolutions.failures());
    findings
}

fn check(page: Page, manifest: HostManifest) -> Result<i32, Fail> {
    let host = load_host(page.clone(), &manifest)?;
    let findings = load_errors(&host);
    let value = serde_json::json!({
        "verb": "check",
        "page": page.path,
        "page_content_sha256": page.content_sha256(),
        "git_revision": git_revision(),
        "valid": findings.is_empty(),
        "valid_fence_count": host.parsed.components.iter().filter(|c| c.error.is_none()).count(),
        "fence_count": host.parsed.components.len(),
        "findings": findings,
    });
    let _unloaded = host.unload();
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_default()
    );
    Ok(i32::from(
        !value
            .get("valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    ))
}

fn render(page: Page, manifest: HostManifest) -> Result<i32, Fail> {
    let host = load_host(page, &manifest)?;
    let bundle_value = bundle::render_bundle(
        &host,
        "render",
        "UNVERIFIED — scene-truth is the structural capture at this layer; pixel proof is a host-surface adapter's job",
    );
    let mut bundle_value = bundle_value;
    bundle::join_identity(&mut bundle_value, git_revision());
    let fence_errors_empty = bundle_value
        .pointer("/evaluation/item_errors")
        .and_then(|v| v.as_array())
        .map(|v| v.is_empty())
        .unwrap_or(true);
    let dependencies_clean = bundle_value
        .pointer("/evaluation/resolutions_clean")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    match bundle::retain(&bundle_value) {
        Ok(path) => {
            println!("bundle retained: {}", path.display());
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle_value).unwrap_or_default()
            );
            if fence_errors_empty && dependencies_clean {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(error) => Err(Fail(format!("bundle retention failed: {error}"))),
    }
}

/// N fresh-reset rounds: each round is a FRESH host (an independently reset
/// state — the acceptance rule at the runtime layer). Round 1 digsests are
/// the page reference; every later round must compare CLEAN against them.
fn run(page: Page, manifest: HostManifest, rounds: usize) -> Result<i32, Fail> {
    if rounds < 2 {
        return usage_result("run needs at least TWO fresh-reset rounds (the acceptance contract)");
    }
    let mut digests: Vec<Digests> = Vec::new();
    for round in 0..rounds {
        let host = load_host(page.clone(), &manifest).map_err(|error| error)?;
        if let Ok(true) = host.ready() {
            digests.push(host.digests().err_or(round)?);
        } else {
            return Err(Fail(format!(
                "round {}: host NEVER READY — page or dependency invalid; not a silent UNVERIFIED pass",
                round + 1
            )));
        }
        let _unloaded = host.unload();
    }
    let mut verdict = "CLEAN".to_string();
    let mut findings: Vec<String> = Vec::new();
    for digest_set in digests.iter().skip(1) {
        let Comparison {
            verdict: compared,
            findings: compared_findings,
        } = bundle::compare(&manifest, &digests[0], digest_set);
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
            "page": page.path,
            "page_content_sha256": page.content_sha256(),
            "git_revision": git_revision(),
            "verdict": verdict,
            "findings": findings,
        })
    );
    Ok(i32::from(verdict != "CLEAN"))
}

fn usage_result(message: &str) -> Result<i32, Fail> {
    eprintln!("emd-page: {message}");
    Ok(2)
}

fn selftest(page: Page, manifest: HostManifest) -> Result<i32, Fail> {
    let report = emd_page_host::selftest::run_selftest(&page, &manifest);
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

/// Turns a Possible digest error into an I/O-level failure with context.
trait ErrOr {
    fn err_or(self, round: usize) -> Result<Digests, Fail>;
}

impl ErrOr for Result<Digests, String> {
    fn err_or(self, round: usize) -> Result<Digests, Fail> {
        self.map_err(|error| Fail(format!("round {round}: digest computation failed: {error}")))
    }
}
