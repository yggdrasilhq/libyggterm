// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Dependency resolution with actionable failures. Portable: an arm
//! resolves its OWN dependencies (an emd page against the local component
//! workspace; a plain fixture app against its declared launcher shape) but
//! the resolution TYPE table — and therefore the bundle schema — is ONE.
//! Anything unresolved names the manifest key and the expected shape.

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

/// The full resolution table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Resolutions {
    pub implementation: Vec<(String, Resolution)>,
    pub design: Option<(String, Resolution)>,
}

impl Resolutions {
    pub fn failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        for (_key, resolution) in &self.implementation {
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

/// Key prefix the resolution REPORT names (kept arm-honest: the emd arm
/// says `page.host.json:`; the plain arm says `scenario.host.json:`).
pub type ResolutionKey = String;
