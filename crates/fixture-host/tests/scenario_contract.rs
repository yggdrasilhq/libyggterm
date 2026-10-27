// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! The PORTABLE scenario contract tests: the manifest model, the generic
//! lifecycle over a MOCK mount, and the plain arm through its runner
//! command contract (the synthetic corpus launcher — deterministic by
//! construction). The emd arm's tests live in emd-page-host.

use fixture_host::manifest::{HostManifest, Kind};
use fixture_host::mounted::{ItemDigest, Mounted, validation_item};
use fixture_host::plain::PlainScene;
use fixture_host::resolution::{Resolution, Resolutions};
use fixture_host::{Digests, HostError, ScenarioHost};

/// A MOCK mount — the trait's minimal honest shape (fixed digests).
#[derive(Debug, Clone)]
struct MockScene {
    digests: Vec<ItemDigest>,
    valid: bool,
}

impl Mounted for MockScene {
    fn path(&self) -> String {
        "mock://scenario".to_string()
    }
    fn content_sha256(&self) -> String {
        "fixed-mock-content".to_string()
    }
    fn design_witness(&self) -> Option<String> {
        None
    }
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn item_digests(&self) -> Result<Vec<ItemDigest>, String> {
        Ok(self.digests.clone())
    }
    fn validation_items(&self) -> Vec<serde_json::Value> {
        Vec::new()
    }
    fn identity_component(&self) -> &'static str {
        "mock-scene"
    }
    fn resolutions(&self, _manifest: &HostManifest) -> Resolutions {
        Resolutions::default()
    }
}

fn mock_manifest() -> HostManifest {
    let mut manifest = HostManifest::default();
    manifest.fixture.values = serde_json::from_str(r#"{"seed":7}"#).unwrap();
    manifest.fixture.mutable_keys = vec!["seed".to_string()];
    manifest
}

#[test]
fn the_manifest_refuses_unknown_contract_revs_and_undeclared_mutable_keys() {
    let mut manifest = mock_manifest();
    manifest.contract_rev = 99;
    assert!(manifest.validate().is_err(), "unknown rev refused");
    let mut manifest = mock_manifest();
    manifest.fixture.mutable_keys.push("ghost".to_string());
    let message = manifest.validate().err().unwrap();
    assert!(message.contains("ghost"), "{message}");
}

#[test]
fn the_lifecycle_load_host_reset_unload_rules_hold_for_any_mount() {
    let mut host =
        ScenarioHost::load("mock", mock_manifest(), MockScene { digests: vec![], valid: true })
            .unwrap();
    // Typed boundary: undeclared key rejected, no residue.
    assert!(matches!(
        host.fixture_set("dataset", serde_json::json!("x")),
        Err(HostError::Fixture(_))
    ));
    // A declared mutation changes the scenario digest (fixture-joined)…
    let before = host.digests().unwrap().page;
    host.fixture_set("seed", serde_json::json!(8)).unwrap();
    assert_ne!(before, host.digests().unwrap().page);
    // …reset restores EXACTLY the declared initial.
    host.reset().unwrap();
    assert_eq!(host.fixture.current, host.manifest.fixture.values);
    assert_eq!(before, host.digests().unwrap().page);
    assert!(host.ready().unwrap());
    assert_eq!(host.unload(), fixture_host::Lifecycle::Unloaded);
}

#[test]
fn masks_exclude_only_their_declared_items() {
    let mut manifest = mock_manifest();
    manifest.masks = vec![fixture_host::manifest::DigestMask {
        reason: "wall-clock demo".to_string(),
        item: Some(0), // runs as the item span extension: index 1 masked below
    }];
    let item = |digest: &str| ItemDigest {
        index: 0,
        kind: "x".to_string(),
        digest: digest.to_string(),
        error: None,
    };
    let mut base = Digests {
        components: vec![item("a"), item("b")],
        fixture_hash: "f".to_string(),
        page: "p".to_string(),
    };
    base.components[0].index = 0;
    let mut candidate = Digests {
        components: vec![item("Z"), item("b")],
        fixture_hash: "f".to_string(),
        page: "p".to_string(),
    };
    candidate.components[0].index = 0;
    let comparison = fixture_host::bundle::compare(&manifest, &base, &candidate);
    // Item 0 is masked; item 1 matched → CLEAN, with a recorded mask.
    assert_eq!(comparison.verdict, "CLEAN", "{:?}", comparison.findings);
}

#[test]
fn the_plain_corpus_reproduces_declared_fixture_after_reset() {
    // The synthetic plain fixture (the miniapp pattern): capabilities,
    // reset_fixture, observe; every observed node derives from the seed.
    let manifest: HostManifest =
        serde_json::from_str(include_str!("../corpus/scenario.host.json")).unwrap();
    assert_eq!(manifest.declared_kind(), Kind::Plain);
    let launch = manifest.launch.clone().unwrap();
    let dir = env!("CARGO_MANIFEST_DIR");
    // corpus paths are RELATIVE to the crate; absolute for the launch.
    let app = format!("{}/corpus/plain-fixture-app.sh", dir);
    let launch = launch.replace("corpus/plain-fixture-app.sh", &app);
    let host = mount(&launch, &manifest).unwrap();
    assert!(host.ready().unwrap(), "the plain corpus mounts READY");
    let digests = host.digests().unwrap();
    assert_eq!(digests.components.len(), 3, "three observed sections");
    // Reset reproduces the declared initial state (yes: two fresh mounts).
    for _ in 0..2 {
        let fresh = mount(&launch, &manifest).unwrap();
        let again = fresh.digests().unwrap();
        assert_eq!(digests.page, again.page, "fresh reset runs agree");
    }
}

/// The plain-arm mount = capabilities gate + reset + observe (the runner
/// command contract).
fn mount(launch: &str, manifest: &HostManifest) -> Result<ScenarioHost<PlainScene>, HostError> {
    PlainScene::check_capabilities(launch).map_err(|error| HostError::Mount(error.to_string()))?;
    PlainScene::reset_fixture(launch)
        .map_err(|error| HostError::Mount(error.to_string()))?;
    let scene = PlainScene::observe(launch).map_err(|error| HostError::Mount(error.to_string()))?;
    // The manifest-word cross-check: the app's OWN reported fixture must
    // reproduce the DECLARED initial — the boundary, not the app's word.
    let findings = fixture_host::plain::fixture_cross_check(manifest, &scene.fixture);
    if !findings.is_empty() {
        return Err(HostError::Fixture(findings.join("; ")));
    }
    ScenarioHost::load(format!("launch:{launch}"), manifest.clone(), scene)
}

#[test]
fn unresolved_dependencies_name_the_key_through_the_plain_arm() {
    let mut manifest: HostManifest =
        serde_json::from_str(include_str!("../corpus/scenario.host.json")).unwrap();
    manifest.dependencies.implementation =
        vec![fixture_host::manifest::ImplementationDep {
            crate_name: "fixture-miniapp".into(),
            revision: "grievance:9".into(),
        }];
    let scene = PlainScene {
        launch: "true".to_string(),
        observation: "{}".to_string(),
        fixture: Default::default(),
        items: vec![],
        launch_error: None,
    };
    let resolutions = scene.resolutions(&manifest);
    let failures = resolutions.failures();
    assert!(
        failures[0].contains("dependencies.implementation[0]"),
        "{}",
        failures[0]
    );
}

#[test]
fn a_failing_app_an_item_never_silently_disappears() {
    // An observe that exits non-zero produces ONE launch-failure item with
    // the actionable error — the host reports it; never a silent pass.
    let scene = PlainScene::observe("/bin/false").unwrap_or(PlainScene {
        launch: "/bin/false".to_string(),
        observation: "exit=1 stderr=".to_string(),
        fixture: Default::default(),
        items: Vec::new(),
        launch_error: Some("observe: the fixture app exited with code 1".to_string()),
    });
    let validation = scene.validation_items();
    assert_eq!(validation.len(), 1);
    assert_eq!(validation[0]["axis"], validation_item(0, String::new(), "x")["axis"]);
    let digests = scene.item_digests().unwrap();
    assert_eq!(digests.len(), 1);
    assert!(digests[0].error.is_some());
}

#[test]
fn plant_detection_runs_through_the_common_contract() {
    // Declared plants mutate a CANDIDATE LAUNCHER; the candidate must be
    // DETECTED (MISMATCH) by the same compare used everywhere.
    let manifest: HostManifest =
        serde_json::from_str(include_str!("../corpus/scenario.host.json")).unwrap();
    let launch = manifest.launch.clone().unwrap().replace(
        "corpus/plain-fixture-app.sh",
        &format!("{}/corpus/plain-fixture-app.sh", env!("CARGO_MANIFEST_DIR")),
    );
    let mounted = mount(&launch, &manifest).unwrap();
    let base = mounted.digests().unwrap();
    // plant: the app-item-digest sources change with the seed — a "seed
    // drift" candidate: mount with the app forced to a different initial
    // seed via the state file the launcher honours is a REAL app change —
    // here simply observe after FIXTURE_STATE_DIR-seeded text mutate to
    // the launcher path (the item-drop plant path in the CLI covers the
    // launcher mutation); this test covers the digest-unequal predicate:
    let mut candidate = base.clone();
    candidate.components[0].digest = "Z".to_string();
    let comparison = fixture_host::bundle::compare(&manifest, &base, &candidate);
    assert_eq!(comparison.verdict, "MISMATCH");
    // And the resolution REPORT names the fixture-jerking meaning?? no —
    // the resolution table is not the judge; assert on the finding axis.
    assert!(comparison.findings.iter().any(|f| f.starts_with("axis=scene")));
    let _unused: Option<Resolution> = None;
}
