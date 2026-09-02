// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The plain-text (ASCII) editing surface — the DEGENERATE emd document.
//!
//! Ownership moved here from yedit on 2026-09-02 (ymacs
//! `docs/spec-primitives.md` §1.1): the whole text is ONE block, and the
//! edit semantics every libyggterm text editor shares — byte-faithful
//! splices, line math, the round-trip invariant — are THIS engine's, so
//! yedit's text mode, ymacs' editor slot, and any later app render and edit
//! through one model instead of each re-deriving line/offset math over a
//! `String`.
//!
//! The Dioxus VIEW of this surface (the multiline `text-input` widget) stays
//! in the shell for now — the same deferral as the markdown render (spec
//! §4). What a host must not re-implement anymore is the model below.
//!
//! Doctrines carried over from the markdown side:
//! - **The source is the document.** `TextSurface` owns the exact bytes;
//!   splices are the only mutation, and anything outside an edited range is
//!   byte-faithful by construction (CRLF, trailing whitespace, BOM-ish
//!   oddities all survive — never re-serialize what you do not own).
//! - **Fail loud at char boundaries.** An offset that would split a UTF-8
//!   scalar is a caller bug that would corrupt the document silently if
//!   saturated — it panics with the offending offset instead.

/// The whole text as one block. Line convention matches Emacs: an empty
/// document has ONE (empty) line; `"a\n"` is one line; `"a\nb"` is two.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextSurface {
    source: String,
}

impl TextSurface {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn into_source(self) -> String {
        self.source
    }

    /// Byte offset of every line start (line 0 starts at 0). A trailing
    /// `\n` does not start a new line — the Emacs convention `len_lines`
    /// uses is the same one these starts encode.
    pub fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0usize];
        for (i, b) in self.source.bytes().enumerate() {
            if b == b'\n' && i + 1 < self.source.len() {
                starts.push(i + 1);
            }
        }
        starts
    }

    /// Number of lines under the Emacs convention (never zero).
    pub fn len_lines(&self) -> usize {
        self.source.split_inclusive('\n').count().max(1)
    }

    /// Byte range of a line's CHARACTERS, excluding the `\n` terminator.
    /// A `\r` before the terminator is content (byte-faithful doctrine).
    pub fn line_range(&self, line: usize) -> std::ops::Range<usize> {
        let starts = self.line_starts();
        assert!(
            line < starts.len(),
            "line {line} out of range ({} lines)",
            starts.len()
        );
        let start = starts[line];
        let hard_end = starts.get(line + 1).copied().unwrap_or(self.source.len());
        let end = if hard_end > start && self.source.as_bytes()[hard_end - 1] == b'\n' {
            hard_end - 1
        } else {
            hard_end
        };
        start..end
    }

    /// Byte offset of (0-based line, byte column). `line == len_lines()`
    /// with column 0 is the point-max position (one past the last
    /// character). Panics past the end or on a column that is not a char
    /// boundary of that line.
    pub fn offset_of(&self, line: usize, col: usize) -> usize {
        let starts = self.line_starts();
        if line == starts.len() {
            assert!(
                col == 0,
                "the phantom line after the last newline has only column 0"
            );
            return self.source.len();
        }
        let range = self.line_range(line);
        let at = range.start + col;
        assert!(
            at <= range.end,
            "col {col} past end of line {line} ({} chars)",
            range.len()
        );
        assert!(
            self.source.is_char_boundary(at),
            "col {col} splits a UTF-8 scalar on line {line}"
        );
        at
    }

    /// (0-based line, 0-based byte column) of a byte offset. `offset ==
    /// len` addresses the position after the last character.
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        assert!(
            offset <= self.source.len() && self.source.is_char_boundary(offset),
            "offset {offset} is not a char boundary at or before the end"
        );
        let mut line = 0usize;
        let mut last_start = 0usize;
        for (i, b) in self.source.bytes().enumerate() {
            if i >= offset {
                break;
            }
            if b == b'\n' {
                line += 1;
                last_start = i + 1;
            }
        }
        (line, offset - last_start)
    }

    /// Byte-faithful insert at a char boundary.
    pub fn insert(&mut self, offset: usize, text: &str) {
        assert!(
            self.source.is_char_boundary(offset),
            "insert at {offset} is not a char boundary"
        );
        self.source.insert_str(offset, text);
    }

    /// Byte-faithful delete; everything outside `range` is untouched.
    pub fn delete(&mut self, range: std::ops::Range<usize>) {
        assert!(range.start <= range.end, "inverted range {range:?}");
        assert!(
            self.source.is_char_boundary(range.start)
                && self.source.is_char_boundary(range.end)
                && range.end <= self.source.len(),
            "delete range {range:?} is not on char boundaries within the document"
        );
        self.source.replace_range(range, "");
    }

    /// Byte-faithful replace — the one primitive node-granular editing
    /// composes from (org's TODO cycle and checkbox toggle both land here).
    pub fn replace(&mut self, range: std::ops::Range<usize>, text: &str) {
        self.delete(range.clone());
        self.insert(range.start, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splices_are_byte_faithful_outside_the_edit() {
        let mut s = TextSurface::new("alpha\nbeta\r\ngamma");
        s.replace(6..10, "BETA");
        assert_eq!(s.source(), "alpha\nBETA\r\ngamma");
        // The CRLF terminator survived untouched; only the four edited
        // bytes moved.
        s.insert(0, "»");
        assert_eq!(s.source(), "»alpha\nBETA\r\ngamma");
        s.delete(0..2);
        assert_eq!(s.source(), "alpha\nBETA\r\ngamma");
    }

    #[test]
    fn line_math_matches_emacs_conventions() {
        let s = TextSurface::new("");
        assert_eq!(s.len_lines(), 1, "an empty document has one line");
        assert_eq!(s.line_range(0), 0..0);

        let s = TextSurface::new("a\n");
        assert_eq!(s.len_lines(), 1, "a trailing newline does not add a line");
        assert_eq!(s.line_range(0), 0..1);

        let s = TextSurface::new("a\nb");
        assert_eq!(s.len_lines(), 2);
        assert_eq!(s.line_range(1), 2..3);
    }

    #[test]
    fn line_math_survives_utf8_and_crlf() {
        let s = TextSurface::new("héllo\n中文行\r\nthree");
        let (l, c) = s.line_col(s.offset_of(1, 3));
        assert_eq!((l, c), (1, 3));
        // \r before a \n is content (byte-faithful doctrine).
        assert_eq!(&s.source()[s.line_range(1)], "中文行\r");
        // offset_of must refuse a column that splits 中.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.offset_of(1, 1)));
        assert!(result.is_err(), "mid-scalar column must fail loud");
    }

    #[test]
    fn line_col_round_trips_through_every_offset() {
        let s = TextSurface::new("one\ntwo\n");
        for at in 0..=s.source().len() {
            let (l, c) = s.line_col(at);
            assert_eq!(s.offset_of(l, c), at, "offset {at} did not round-trip");
        }
    }

    #[test]
    fn a_splice_persists_through_further_line_math() {
        // The composition the org animations rely on: splice on line 0,
        // then re-derive a LATER line's range — offsets shift by the edit.
        let mut s = TextSurface::new("* TODO one\nplain\n- [ ] box\n");
        // Same-length keyword swap keeps every offset.
        s.replace(2..6, "DONE");
        assert_eq!(s.source(), "* DONE one\nplain\n- [ ] box\n");
        // Removing the keyword and its space shifts later lines up by 5.
        let r = s.line_range(0);
        s.replace(2..7, "");
        assert_eq!(s.source(), "* one\nplain\n- [ ] box\n");
        assert_eq!(&s.source()[s.line_range(1)], "plain");
        assert_eq!(&s.source()[s.line_range(2)], "- [ ] box");
    }
}
