// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Error presentation with actionable locations. Every validation failure a
//! host reports carries WHERE it failed: the fence index (nth ```emd block,
//! 1-based in report text), the byte range inside the page source, the
//! line:col of the fence opener, and — when the failure is inside the
//! component tree — the typed component path (`root.grid.children[2]...`).

use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentLocation {
    /// 0-based index of the ```emd fence within the page, document order.
    pub fence_index: usize,
    /// Byte range of the whole fence (``` opener to the closing fence),
    /// when the offset-iter mapping aligned. `None` means the internal
    /// range-mapping invariant failed — itself a loud page-level defect.
    pub byte_range: Option<Range<usize>>,
    /// 1-based line and column of the fence opener.
    pub line: usize,
    pub col: usize,
    /// Path of the fenced block through the markdown tree
    /// (e.g. `blocks[1]`, `blocks[4].quote.blocks[0]`).
    pub block_path: String,
}

impl ComponentLocation {
    pub fn line_col(&self) -> String {
        format!("{}:{}", self.line, self.col)
    }
}

/// One actionable diagnostic for one fence.
#[derive(Debug, Clone, PartialEq)]
pub struct PageItemError {
    pub location: ComponentLocation,
    /// The parser/validator message (spec-grade: names the violated rule).
    pub message: String,
}

impl PageItemError {
    /// A one-line finding in the ux-falsify finding shape.
    pub fn finding(&self, page_display: &str) -> String {
        let region = self.location.line_col();
        let path = self.location.block_path.as_str();
        format!(
            "[{page_display}.emd-fence#{}] axis=validation region={region} path={path} detail: {message} fix: address the named component path in the fenced JSON",
            self.location.fence_index + 1,
            message = self.message,
        )
    }
}

/// A page-level failure (malformed manifest, unreadable file, misaligned
/// range mapping).
#[derive(Debug, Clone, PartialEq)]
pub struct PageError {
    pub message: String,
}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PageError {}
