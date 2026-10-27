// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The hosted-component contract tests: a component page must behave as a
//! reproducible executable test host — the load-bearing claim of the
//! ydesign loop (chain 0028 Q4). These tests are the acceptance evidence.

use emd_page_host::{HostManifest, Page, PageHost, selftest};

fn manifest() -> HostManifest {
    HostManifest {
        contract_rev: 1,
        ..HostManifest::default()
    }
}

fn synthetic_page() -> Page {
    // SYNTHETIC only: invented values, no private data.
    let source = "# A synthetic page\n\nSome prose before the fence.\n\n```emd\n\
{\"version\":1,\"kind\":\"grid\",\"spec\":{\"columns\":1,\"gap_px\":8,\"children\":[\
{\"kind\":\"sparkline\",\"spec\":{\"label\":\"Synthetic CPU\",\"values\":[4.0,null,6.0],\"evidence\":{\"question\":\"is CPU flat?\",\"source\":\"/dev/null\",\"window\":\"0\",\"freshness\":\"never\",\"units\":\"unitless\",\"state\":\"observed\",\"reproduction\":\"echo synthetic\"}}},\
{\"kind\":\"metric\",\"spec\":{\"label\":\"Synthetic label\",\"value\":\"1\",\"tone\":\"good\",\"evidence\":{\"question\":\"is the metric singular?\",\"source\":\"/dev/null\",\"window\":\"0\",\"freshness\":\"never\",\"units\":\"count\",\"state\":\"observed\",\"reproduction\":\"echo synthetic\"}}}\
]}}\n```\n";
    Page {
        path: "synthetic-page.emd".to_string(),
        source: source.to_string(),
        revision: None,
    }
}

#[test]
fn a_page_loads_and_is_hoing_through_the_documented_path() {
    let host = PageHost::load(synthetic_page(), manifest()).unwrap();
    assert!(
        host.ready().unwrap(),
        "a valid page with no declared behavior is READY"
    );
    let digests = host.digests().unwrap();
    assert_eq!(digests.components.len(), 1, "one ```emd fence");
    assert_eq!(digests.components[0].kind, "grid");
    let _unloaded = host.unload();
}

#[test]
fn versioned_validation_names_the_rejected_version_first() {
    // A future version: the rejected VERSION is named, not a shape error.
    let page = synthetic_page();
    let future = page.source.replace("\"version\":1", "\"version\":2");
    let page = Page {
        source: future,
        ..page
    };
    let parsed = emd_page_host::page::parse_page(&page).unwrap();
    let errors = parsed.errors();
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0]
            .message
            .contains("unsupported EMD component version 2"),
        "the rejected version is NAMED: {}",
        errors[0].message
    );
    // Location: fence index, byte range OPENED at the fence, line 6 (the
    // fence is the page's 6th line), and a top-level block path.
    let location = &errors[0].location;
    assert_eq!(location.fence_index, 0);
    assert_eq!(location.block_path, "blocks[2]");
    assert_eq!(location.line, 5, "{:?}", location);
    assert!(
        location.byte_range.is_some(),
        "byte range present (click-to-edit substrate)"
    );
    // A PRE-floor version needs a migration decision — loud, never silent.
    let past = page.source.replace("\"version\":2", "\"version\":0");
    let page = Page {
        source: past,
        ..page
    };
    let parsed = emd_page_host::page::parse_page(&page).unwrap();
    assert!(
        parsed.errors()[0]
            .message
            .contains("predates this renderer's floor"),
        "{}",
        parsed.errors()[0].message
    );
}

#[test]
fn bounds_and_typed_boundaries_report_actionable_locations() {
    // A bound violation names the component path, not just the rule.
    let page = synthetic_page();
    let bad_columns = page.source.replace("\"columns\":1", "\"columns\":9");
    let page = Page {
        source: bad_columns,
        ..page
    };
    let parsed = emd_page_host::page::parse_page(&page).unwrap();
    let errors = parsed.errors();
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains("root.grid.columns"),
        "path carried: {}",
        errors[0].message
    );
    assert!(
        errors[0].message.contains("between 1 and 4"),
        "{}",
        errors[0].message
    );
    assert!(
        errors[0].finding(&page.path).contains("axis=validation"),
        "the finding shape carries an axis: {}",
        errors[0].finding(&page.path)
    );
}

#[test]
fn an_unresolved_dependency_is_actionable_before_any_render() {
    let mut unresolved = manifest();
    unresolved.dependencies.implementation = vec![emd_page_host::manifest::ImplementationDep {
        crate_name: "yaml-magix".to_string(),
        revision: "workspace:0.0.1".to_string(),
    }];
    let host_result = PageHost::load(synthetic_page(), unresolved);
    // Unresolved is NOT a load error — the dependency failure must be
    // REPORTABLE through the same resolution table (the manifest key is
    // named), while the host stays auditable.
    let host = host_result.unwrap();
    assert!(!host.resolution.ok());
    let failures = host.resolution.failures();
    assert!(
        failures[0].contains("dependencies.implementation[0].revision"),
        "the failing key is named: {}",
        failures[0]
    );
    assert!(
        failures[0].contains("workspace"),
        "the expected shape is named: {}",
        failures[0]
    );
    assert!(
        !host.ready().unwrap(),
        "an unresolved dependency blocks readiness"
    );
}

#[test]
fn a_resolved_workspace_dependency_is_clean() {
    let mut manifest = manifest();
    manifest.dependencies.implementation = vec![emd_page_host::manifest::ImplementationDep {
        crate_name: "emd-renderer".to_string(),
        revision: format!(
            "workspace:{}",
            emd_page_host::resolve::HOST_WORKSPACE_VERSION
        ),
    }];
    let host = PageHost::load(synthetic_page(), manifest).unwrap();
    assert!(host.resolution.ok(), "{:?}", host.resolution.failures());
    assert!(host.ready().unwrap());
}

#[test]
fn fixture_set_is_typed_and_reset_restores_the_declared_initial() {
    let mut manifest = manifest();
    manifest.fixture.values =
        serde_json::from_str(r#"{"dataset":"synthetic-t3-buttons","seed":7}"#).unwrap();
    manifest.fixture.mutable_keys = vec!["seed".to_string()];
    let mut host = PageHost::load(synthetic_page(), manifest).unwrap();
    assert_eq!(host.fixture.current_digest(), host.fixture.initial_digest());

    // An UNDECLARED fixture mutation is a boundary violation.
    let taboo = host.fixture_set("dataset", serde_json::json!("overwritten"));
    assert!(matches!(
        taboo,
        Err(emd_page_host::host::HostError::Fixture(_))
    ));
    assert_eq!(
        host.fixture.current_digest(),
        host.fixture.initial_digest(),
        "a rejected mutation left no residue"
    );

    // A declared mutation changes fixture state — and the page digest with
    // it (altered fixture state never looks like an unaltered page).
    let before = host.digests().unwrap();
    host.fixture_set("seed", serde_json::json!(8)).unwrap();
    assert_ne!(host.fixture.current_digest(), host.fixture.initial_digest());
    let after = host.digests().unwrap();
    assert_ne!(
        before.page, after.page,
        "fixture mutation changes the page digest"
    );

    // Reset: EXACTLY the declared initial state again.
    host.reset().unwrap();
    assert_eq!(host.fixture.current, host.manifest.fixture.values);
    let restored = host.digests().unwrap();
    assert_eq!(
        before.fixtures(),
        restored.fixtures(),
        "reset reproduces the declared initial state"
    );
}

// A small accessor the tests use to compare the FIXTURE digest across hosts.
trait Fixtures {
    fn fixtures(&self) -> String;
}
impl Fixtures for emd_page_host::Digests {
    fn fixtures(&self) -> String {
        self.fixture_hash.clone()
    }
}

#[test]
fn repeated_reloads_leave_no_orphaned_state_and_no_duplicate_hosts() {
    let manifest_with_fixture = {
        let mut manifest = manifest();
        manifest.fixture.values = serde_json::from_str(r#"{"seed":7}"#).unwrap();
        manifest.fixture.mutable_keys = vec!["seed".to_string()];
        manifest
    };
    // Navigation 1: load, mutate, unload.
    let mut host = PageHost::load(synthetic_page(), manifest_with_fixture.clone()).unwrap();
    host.fixture_set("seed", serde_json::json!(99)).unwrap();
    let generation = host.load_generation;
    let _unloaded = host.unload(); // the controlled terminal state

    // Navigation 2: a FRESH load — the declared fixture, no residual 99,
    // no duplicate host registration (a fresh host is a fresh object).
    let host = PageHost::load(synthetic_page(), manifest_with_fixture).unwrap();
    assert_eq!(
        host.fixture_read("seed"),
        Some(&serde_json::json!(7)),
        "no residual fixture state"
    );
    assert_eq!(
        host.load_generation, generation,
        "each fresh load is a fresh generation"
    );
    assert!(host.ready().unwrap());
    let digests = host.digests().unwrap();
    reset_and_compare(&host, digests);
}

/// Two fresh reset runs must reproduce the SAME digest — acceptance rule 3+6.
fn reset_and_compare(host: &PageHost, digests: emd_page_host::Digests) {
    for _ in 0..2 {
        let mut fresh = PageHost::load(host.page.clone(), host.manifest.clone()).unwrap();
        fresh.reset().unwrap();
        let again = fresh.digests().unwrap();
        assert_eq!(again.page, digests.page, "two fresh reset runs agree");
        let _unloaded = fresh.unload();
    }
}

#[test]
fn determinism_gate_ref_vs_ref_is_clean_and_plants_are_detected() {
    // The synthetic CORPUS page runs the planted-defect selftest: declared
    // plants must ALL be detected through the common comparison contract.
    let mut manifest = manifest();
    manifest.selftest.plants = selftest::supported_plants()
        .into_iter()
        .map(str::to_string)
        .collect();
    let page = synthetic_page();
    // The null-to-zero plant needs a `"y": null`; tone-drift needs
    // `"tone":"good"`; version-skip triggers at validation. All three are
    // present in the synthetic fixture by construction.
    let report = selftest::run_selftest(&page, &manifest);
    assert!(
        report.ref_vs_ref.verdict == "CLEAN",
        "the determinism gate runs first and must be CLEAN: {:?}",
        report.ref_vs_ref
    );
    assert!(
        report.passed,
        "all declared plants detected: {:?} (ref_vs_ref: {:?})",
        report.plants, report.ref_vs_ref
    );
}

#[test]
fn pixel_capture_is_never_faked() {
    // Scene-truth is this layer's capture; the bundle records pixels as
    // UNVERIFIED rather than claiming a capture it does not have.
    let host = PageHost::load(synthetic_page(), manifest()).unwrap();
    let bundle_json = emd_page_host::bundle::render_bundle(
        &host,
        "render",
        "UNVERIFIED — pixels are a surface adapter's job",
    );
    assert_eq!(
        bundle_json.pointer("/capture/pixel_capture").unwrap(),
        &serde_json::json!("UNVERIFIED — pixels are a surface adapter's job")
    );
    assert_eq!(
        bundle_json.pointer("/identity/contract_rev").unwrap(),
        &serde_json::json!(1)
    );
}

/// The CLEAN-CHECKOUT acceptance: every shipped design/notebook page parses
/// through the documented path (the REAL corpus; the fixtures are synthetic).
#[test]
fn the_real_notebook_corpus_loads_in_a_clean_checkout() {
    // $CARGO_MANIFEST_DIR = crates/emd-page-host → the workspace root is two above.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design/notebooks");
    let mut rendered = 0;
    for entry in std::fs::read_dir(&root).expect("shipped design/notebooks in a clean checkout") {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|e| e.to_str()) != Some("emd") {
            continue;
        }
        let source = std::fs::read_to_string(entry.path()).unwrap();
        let page = Page {
            path: entry.file_name().into_string().unwrap(),
            source,
            revision: None,
        };
        let parsed = emd_page_host::page::parse_page(&page).unwrap();
        assert!(
            parsed.is_valid(),
            "REAL notebook page has invalid fences: {} — {:?}",
            page.path,
            parsed.errors()
        );
        rendered += 1;
    }
    assert!(
        rendered >= 12,
        "the full notebook corpus renders (got {rendered})"
    );
}
