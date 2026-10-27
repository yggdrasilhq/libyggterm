// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Migration behavior: the MANIFEST FILENAME. `page.host.json` beside an
//! emd page remains the documented emd spelling (contract rev 1, legacy);
//! the portable `scenario.host.json` spelling parses IDENTICALLY and is
//! what the plain arm requires. Until the v2 contract rev, both spellings
//! are valid under the SAME schema; a schema change ships as a new
//! contract rev with its own loader, never as a silent extension.

use emd_page_host::{HostManifest, MANIFEST_FILENAME, SCENARIO_MANIFEST_FILENAME};

#[test]
fn the_legacy_spelling_and_the_portable_spelling_parse_identically() {
    let legacy = r#"{
        "contract_rev": 1,
        "revision": "migration-note",
        "fixture": { "values": { "k": 1 }, "mutable_keys": ["k"] }
    }"#;
    let portable = r#"{
        "contract_rev": 1,
        "kind": "emd-page",
        "revision": "migration-note",
        "fixture": { "values": { "k": 1 }, "mutable_keys": ["k"] }
    }"#;
    let a: HostManifest = serde_json::from_str(legacy).unwrap();
    let b: HostManifest = serde_json::from_str(portable).unwrap();
    assert_eq!(a.fixture, b.fixture);
    assert_eq!(b.declared_kind(), emd_page_host::manifest::Kind::EmdPage);
}

#[test]
fn the_emd_beside_loader_reads_both_spellings_scenario_first() {
    // Directory A (legacy), directory B (portable), workspace root for the
    // beside-lookups — no fixture repository state is touched.
    let tmp = std::env::temp_dir().join("emd-page-host-migration-test");
    std::fs::create_dir_all(&tmp).unwrap();
    let a = tmp.join("legacy");
    let b = tmp.join("portable");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    let manifest = r#"{"contract_rev":1,"revision":"x"}"#;
    std::fs::write(a.join(MANIFEST_FILENAME), manifest).unwrap();
    std::fs::write(b.join(SCENARIO_MANIFEST_FILENAME), manifest).unwrap();
    let (_, a_manifest) =
        emd_page_host::PageHost::load_manifest_beside(&tmp.join("legacy").join("page.emd"))
            .expect("legacy spelling beside a page LOADS");
    assert_eq!(a_manifest.contract_rev, 1);
    let (path, b_manifest) =
        emd_page_host::PageHost::load_manifest_beside(&tmp.join("portable").join("page.emd"))
            .expect("portable spelling beside a page LOADS");
    assert!(path.ends_with("scenario.host.json"));
    assert_eq!(a_manifest.revision, b_manifest.revision);
    let _ = std::fs::remove_dir_all(&tmp);
}
