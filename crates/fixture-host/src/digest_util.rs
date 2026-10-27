// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Shared digest helpers (one sha256 spelling, every arm).

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    let hash = Sha256::digest(bytes);
    let mut out = String::with_capacity(hash.len() * 2);
    for byte in hash {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
