// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The org grammar line — typed org-mode nodes with source ranges.
//!
//! Authorized by ymacs `docs/spec-primitives.md` §1.1 (owner directive,
//! 2026-09-02): org constructs arrive as TYPED NODES here — the same
//! discipline as the ```emd fenced components — never as post-hoc string
//! hacks in a renderer or an editor mode. ymacs' org mode animates these
//! nodes (TODO cycling, checkbox toggling, headline navigation); yedit's
//! later org support and any other libyggterm app consume the same tree.
//!
//! Doctrines, carried over unchanged from the markdown side:
//! - **The source is the document.** Every node carries its byte
//!   `Range`; the leaf ranges TILE the source exactly (`leaf_ranges` +
//!   the tiling test are the lock), so `render == source` holds by
//!   construction and node-granular splices are byte-exact.
//! - **Unknown constructs stay VISIBLE.** A construct this grammar does
//!   not type (a `#+begin_quote` block, a `#+RESULTS:` line, a stars-only
//!   line) parses as `Text` — verbatim, never dropped, never re-wrapped.
//!   Failing loud here means "nothing vanishes", the same contract as
//!   `ComponentError`.
//! - **The engine yields typed nodes; interpretation is the app's
//!   domain** (the wikilink rule, spec §5). The heading keyword slot
//!   records any ALL-CAPS word after the stars — whether that token is a
//!   legal TODO keyword is `org-todo-keywords` membership, the app's
//!   business. Line/byte geometry is the engine's.
//!
//! Line conventions: org headlines need `*` stars FOLLOWED BY A SPACE
//! (parity — a stars-only line is text); `#+begin_src` blocks and
//! `:DRAWER:` blocks are typed only when terminated, else they stay
//! visible as text; CRLF and multibyte titles survive (byte-exact
//! ranges, `\r` is content).

/// One parsed org document: the top-level forest of typed nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrgDoc {
    pub nodes: Vec<OrgNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrgNode {
    Heading(Heading),
    SrcBlock(SrcBlock),
    Drawer(Drawer),
    CheckboxItem(CheckboxItem),
    Table(Table),
    /// The inert remainder: any line(s) this grammar does not type. Always
    /// visible verbatim through its range.
    Text {
        range: std::ops::Range<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1-based outline level (one `*` per level).
    pub level: u8,
    /// The ALL-CAPS keyword slot after the stars, if present (e.g.
    /// `TODO`). Membership in a workflow is the app's decision.
    pub todo: Option<String>,
    /// `[#A]`-style priority cookie character, if present.
    pub priority: Option<char>,
    /// Title text after keyword/priority, with the trailing tag cookie
    /// removed. Raw (not entity-decoded); org has no entities.
    pub title: String,
    /// `:tag:` cookie contents, in order.
    pub tags: Vec<String>,
    /// Byte range of the keyword TOKEN itself — the splice target of a
    /// TODO cycle. `None` when the headline has no keyword.
    pub keyword_range: Option<std::ops::Range<usize>>,
    /// Byte range of the whole headline LINE (stars through terminator).
    pub range: std::ops::Range<usize>,
    /// The subtree under this headline: nested headings and content nodes.
    pub body: Vec<OrgNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrcBlock {
    /// First token after `#+begin_src`, if any.
    pub language: Option<String>,
    /// The exact inner bytes between the begin and end lines.
    pub body: String,
    /// Byte range begin line .. end line (terminators included).
    pub range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawer {
    pub name: String,
    /// The exact inner bytes between the open and `:END:` lines.
    pub body: String,
    pub range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxItem {
    /// `true` unless the state character is a space (org's `[X]`/`[x]`
    /// are both "checked"; the cycle writes `[X]`, Emacs parity).
    pub checked: bool,
    /// Item text after the checkbox.
    pub text: String,
    /// Byte range of the STATE CHARACTER — the toggle splice target.
    pub state_range: std::ops::Range<usize>,
    pub range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// Each `|`-line's content, in order (separators included).
    pub lines: Vec<String>,
    pub range: std::ops::Range<usize>,
}

struct Line {
    /// Byte offset of the line's first character.
    start: usize,
    /// Byte offset of the `\n` (or end of source).
    content_end: usize,
    /// Byte offset one past the `\n` when present, else end of source.
    end_incl: usize,
}

fn scan_lines(source: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            lines.push(Line {
                start,
                content_end: i,
                end_incl: i + 1,
            });
            start = i + 1;
        }
    }
    lines.push(Line {
        start,
        content_end: source.len(),
        end_incl: source.len(),
    });
    lines
}

/// Stars count of a headline line, parity rule (org's outline regex
/// `^\*+ `): one or more `*` FOLLOWED BY A SPACE. A stars-only line is
/// text, not an empty headline.
fn heading_stars(content: &str) -> Option<u8> {
    let stars = content.bytes().take_while(|b| *b == b'*').count();
    if stars == 0 || stars > u8::MAX as usize {
        return None;
    }
    if content[stars..].starts_with(' ') {
        Some(stars as u8)
    } else {
        None
    }
}

const KEYWORD_MARKER: &str = "#+begin_src";
const KEYWORD_END: &str = "#+end_src";
const DRAWER_END: &str = ":end:";

fn line_text<'a>(source: &'a str, line: &Line) -> &'a str {
    &source[line.start..line.content_end]
}

fn trimmed_lower(source: &str, line: &Line) -> String {
    line_text(source, line).trim().to_ascii_lowercase()
}

fn find_block_end(source: &str, lines: &[Line], from: usize, end_marker: &str) -> Option<usize> {
    (from..lines.len()).find(|j| trimmed_lower(source, &lines[*j]) == end_marker)
}

/// A `#+begin_src` line, and where its block ends. `None` when the line
/// is not a src begin or the block is unterminated (unterminated stays
/// visible as text — never a swallowed region).
fn src_block_at(source: &str, lines: &[Line], i: usize) -> Option<(SrcBlock, usize)> {
    let content = line_text(source, &lines[i]);
    if !content
        .trim_start()
        .to_ascii_lowercase()
        .starts_with(KEYWORD_MARKER)
    {
        return None;
    }
    let end_j = find_block_end(source, lines, i + 1, KEYWORD_END)?;
    let after_marker = content.trim_start()[KEYWORD_MARKER.len()..].trim_start();
    let language = after_marker.split_whitespace().next().map(str::to_string);
    let range = lines[i].start..lines[end_j].end_incl;
    let body = if end_j > i + 1 {
        source[lines[i + 1].start..lines[end_j].start].to_string()
    } else {
        String::new()
    };
    Some((
        SrcBlock {
            language,
            body,
            range,
        },
        end_j,
    ))
}

/// A `:NAME:` drawer line, and where it ends.
fn drawer_at(source: &str, lines: &[Line], i: usize) -> Option<(Drawer, usize)> {
    let content = line_text(source, &lines[i]);
    let inner = content.strip_prefix(':')?.strip_suffix(':')?;
    if inner.is_empty()
        || !inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let end_j = find_block_end(source, lines, i + 1, DRAWER_END)?;
    let range = lines[i].start..lines[end_j].end_incl;
    let body = if end_j > i + 1 {
        source[lines[i + 1].start..lines[end_j].start].to_string()
    } else {
        String::new()
    };
    Some((
        Drawer {
            name: inner.to_string(),
            body,
            range,
        },
        end_j,
    ))
}

fn checkbox_at(line: &Line, content: &str) -> Option<CheckboxItem> {
    let indent = content.len() - content.trim_start().len();
    let rest = &content[indent..];
    let bullet = rest.chars().next()?;
    if !matches!(bullet, '-' | '+' | '*') {
        return None;
    }
    let after_bullet = &rest[bullet.len_utf8()..];
    if !after_bullet.starts_with(" [") {
        return None;
    }
    let bytes = after_bullet.as_bytes();
    if bytes.len() < 4 || bytes[3] != b']' {
        return None;
    }
    let state = match bytes[2] {
        b' ' | b'x' | b'X' => bytes[2] as char,
        _ => return None,
    };
    let text = after_bullet[4..]
        .strip_prefix(' ')
        .unwrap_or(&after_bullet[4..]);
    let state_at = line.start + indent + 3;
    Some(CheckboxItem {
        checked: state != ' ',
        text: text.trim().to_string(),
        state_range: state_at..state_at + 1,
        range: line.start..line.end_incl,
    })
}

fn is_tag_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '%')
}

/// Split a trailing `:a:b:` tag cookie off a headline title.
fn split_tags(title_part: &str) -> (&str, Vec<String>) {
    if !title_part.ends_with(':') {
        return (title_part, Vec::new());
    }
    let run_len = title_part
        .chars()
        .rev()
        .take_while(|c| is_tag_char(*c) || *c == ':')
        .map(|c| c.len_utf8())
        .sum::<usize>();
    let run = &title_part[title_part.len() - run_len..];
    if !run.starts_with(':') || run.len() < 2 {
        return (title_part, Vec::new());
    }
    let tags: Vec<String> = run
        .split(':')
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    let title = title_part[..title_part.len() - run_len].trim_end();
    (title, tags)
}

fn parse_heading(line: &Line, content: &str, stars: u8) -> Heading {
    // heading_stars guaranteed the space after the stars. A trailing CR
    // (CRLF file) is content for byte-exact ranges but never part of the
    // parsed title/tags/keyword.
    let content = content.strip_suffix('\r').unwrap_or(content);
    // heading_stars guaranteed the space after the stars.
    let mut rest = content[stars as usize..].strip_prefix(' ').unwrap_or("");
    let mut todo = None;
    let mut keyword_range = None;
    let token_len = rest.split(' ').next().map(str::len).unwrap_or(0);
    let token = &rest[..token_len];
    // The keyword SLOT: an ALL-CAPS token of two or more characters
    // (org's stock keywords are all ≥2). Whether the token is a legal
    // TODO keyword is workflow membership — the app's business, not the
    // engine's. Single uppercase letters stay title text so plain prose
    // headlines ("* A note") parse as Emacs.
    if token_len >= 2
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        todo = Some(token.to_string());
        keyword_range =
            Some(line.start + stars as usize + 1..line.start + stars as usize + 1 + token_len);
        rest = &rest[token_len..];
        rest = rest.strip_prefix(' ').unwrap_or(rest);
    }
    let mut priority = None;
    if rest.starts_with("[#") && rest.len() >= 4 && rest.as_bytes()[3] == b']' {
        priority = rest[2..3].chars().next();
        rest = &rest[4..];
        rest = rest.strip_prefix(' ').unwrap_or(rest);
    }
    let (title, tags) = split_tags(rest);
    Heading {
        level: stars,
        todo,
        priority,
        title: title.trim().to_string(),
        tags,
        keyword_range,
        range: line.start..line.end_incl,
        body: Vec::new(),
    }
}

fn parse_block(source: &str, lines: &[Line], mut i: usize, ctx_stars: u8) -> (Vec<OrgNode>, usize) {
    let mut nodes: Vec<OrgNode> = Vec::new();
    while i < lines.len() {
        let line = &lines[i];
        let content = line_text(source, line);
        if let Some(stars) = heading_stars(content) {
            if stars <= ctx_stars {
                break; // a sibling (or ancestor's sibling) — caller resumes
            }
            let mut heading = parse_heading(line, content, stars);
            let (body, next) = parse_block(source, lines, i + 1, stars);
            heading.body = body;
            nodes.push(OrgNode::Heading(heading));
            i = next;
            continue;
        }
        if let Some((block, end_j)) = src_block_at(source, lines, i) {
            nodes.push(OrgNode::SrcBlock(block));
            i = end_j + 1;
            continue;
        }
        if let Some((drawer, end_j)) = drawer_at(source, lines, i) {
            nodes.push(OrgNode::Drawer(drawer));
            i = end_j + 1;
            continue;
        }
        if let Some(item) = checkbox_at(line, content) {
            nodes.push(OrgNode::CheckboxItem(item));
            i += 1;
            continue;
        }
        if content.starts_with('|') {
            let run_start = i;
            while i < lines.len() && line_text(source, &lines[i]).starts_with('|') {
                i += 1;
            }
            nodes.push(OrgNode::Table(Table {
                lines: lines[run_start..i]
                    .iter()
                    .map(|l| line_text(source, l).to_string())
                    .collect(),
                range: lines[run_start].start..lines[i - 1].end_incl,
            }));
            continue;
        }
        // Text run: coalesce everything the grammar does not type. The
        // first line always belongs (it fell through every branch above).
        let run_start = i;
        while i < lines.len() {
            let c = line_text(source, &lines[i]);
            if heading_stars(c).is_some()
                || checkbox_at(&lines[i], c).is_some()
                || c.starts_with('|')
                || src_block_at(source, lines, i).is_some()
                || drawer_at(source, lines, i).is_some()
            {
                break;
            }
            i += 1;
        }
        nodes.push(OrgNode::Text {
            range: lines[run_start].start..lines[i - 1].end_incl,
        });
    }
    (nodes, i)
}

/// Parse org source into the typed forest. Total: the leaf ranges tile
/// the source exactly, so nothing is dropped and splices stay byte-exact.
pub fn parse_org(source: &str) -> OrgDoc {
    let lines = scan_lines(source);
    let (nodes, _next) = parse_block(source, &lines, 0, 0);
    OrgDoc { nodes }
}

impl OrgNode {
    /// The node's full source range (headline line for a heading — the
    /// subtree lives in `body`).
    pub fn range(&self) -> std::ops::Range<usize> {
        match self {
            OrgNode::Heading(h) => h.range.clone(),
            OrgNode::SrcBlock(b) => b.range.clone(),
            OrgNode::Drawer(d) => d.range.clone(),
            OrgNode::CheckboxItem(c) => c.range.clone(),
            OrgNode::Table(t) => t.range.clone(),
            OrgNode::Text { range } => range.clone(),
        }
    }
}

impl OrgDoc {
    /// Leaf ranges depth-first in document order — the tiling lock.
    pub fn leaf_ranges(&self) -> Vec<std::ops::Range<usize>> {
        fn walk(node: &OrgNode, out: &mut Vec<std::ops::Range<usize>>) {
            if let OrgNode::Heading(h) = node {
                out.push(h.range.clone());
                for child in &h.body {
                    walk(child, out);
                }
            } else {
                out.push(node.range());
            }
        }
        let mut out = Vec::new();
        for node in &self.nodes {
            walk(node, &mut out);
        }
        out
    }

    /// Every heading, depth-first in document order.
    pub fn headings(&self) -> Vec<&Heading> {
        fn walk<'a>(node: &'a OrgNode, out: &mut Vec<&'a Heading>) {
            if let OrgNode::Heading(h) = node {
                out.push(h);
                for child in &h.body {
                    walk(child, out);
                }
            }
        }
        let mut out = Vec::new();
        for node in &self.nodes {
            walk(node, &mut out);
        }
        out
    }
}

/// Byte-exact splice — the one editing primitive; org animations compose
/// it over node ranges. The plain-text surface owns the semantics, so
/// this is `TextSurface::replace` with sugar.
pub fn splice(source: &str, range: std::ops::Range<usize>, replacement: &str) -> String {
    let mut surface = crate::TextSurface::new(source);
    surface.replace(range, replacement);
    surface.into_source()
}

/// The default TODO cycle the engine ships (org's stock workflow):
/// no keyword → `TODO` → `DONE` → none. A custom workflow order is the
/// app's configuration, not the engine's.
pub fn next_todo_keyword(current: Option<&str>) -> Option<String> {
    match current {
        None => Some("TODO".to_string()),
        Some("TODO") => Some("DONE".to_string()),
        Some(_) => None,
    }
}

/// Cycle the TODO keyword of one heading in `source`, byte-exactly:
/// the title, tags, and every other line are untouched. A keyword is
/// removed together with the single space that follows it.
pub fn cycle_todo(source: &str, heading: &Heading) -> String {
    let next = next_todo_keyword(heading.todo.as_deref());
    match (heading.keyword_range.clone(), next) {
        (Some(range), Some(keyword)) => splice(source, range, &keyword),
        (Some(range), None) => {
            let mut end = range.end;
            if source[end..].starts_with(' ') {
                end += 1;
            }
            splice(source, range.start..end, "")
        }
        (None, Some(keyword)) => {
            let at = heading.range.start + heading.level as usize + 1;
            splice(source, at..at, &format!("{keyword} "))
        }
        (None, None) => source.to_string(),
    }
}

/// Toggle one checkbox in `source`, byte-exactly: space → `X`, any
/// checked state → space (Emacs `C-c C-c` parity).
pub fn toggle_checkbox(source: &str, item: &CheckboxItem) -> String {
    let state = if item.checked { " " } else { "X" };
    splice(source, item.state_range.clone(), state)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE LOCK: leaf ranges tile the source exactly — contiguous,
    /// ordered, no gaps, no overlaps. Every grammar addition re-runs
    /// this over a fixture containing that addition.
    #[test]
    fn leaf_ranges_tile_the_source_exactly() {
        let source = "* Top :core:\nintro text\n** Nested\n- [ ] box one\n- [X] box two\n#+begin_src rust\nfn main() {}\n#+end_src\n:PROPERTIES:\n:prop: 1\n:END:\n| a | b |\n|---+---|\n| 1 | 2 |\ntrailing words\n";
        let doc = parse_org(source);
        let ranges = doc.leaf_ranges();
        let mut at = 0usize;
        for r in &ranges {
            assert_eq!(r.start, at, "gap or overlap at byte {at}: {ranges:?}");
            at = r.end;
        }
        assert_eq!(at, source.len(), "ranges must reach the end");
    }

    #[test]
    fn headings_carry_todo_priority_tags_and_the_keyword_range() {
        let source = "* TODO [#A] Write the engine :rust:core:\n";
        let doc = parse_org(source);
        let headings = doc.headings();
        assert_eq!(headings.len(), 1);
        let h = headings[0];
        assert_eq!(h.level, 1);
        assert_eq!(h.todo.as_deref(), Some("TODO"));
        assert_eq!(h.priority, Some('A'));
        assert_eq!(h.title, "Write the engine");
        assert_eq!(h.tags, vec!["rust", "core"]);
        let kr = h.keyword_range.clone().unwrap();
        assert_eq!(&source[kr], "TODO");
    }

    #[test]
    fn subtrees_nest_by_stars() {
        let source = "* A\n** B\n*** C\n* D\n";
        let doc = parse_org(source);
        let top: Vec<&str> = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                OrgNode::Heading(h) => Some(h.title.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(top, vec!["A", "D"]);
        let a = match &doc.nodes[0] {
            OrgNode::Heading(h) => h,
            _ => unreachable!(),
        };
        let b = match a.body[0] {
            OrgNode::Heading(ref h) => h,
            _ => unreachable!(),
        };
        assert_eq!(b.title, "B");
        assert!(matches!(b.body[0], OrgNode::Heading(ref h) if h.title == "C"));
    }

    #[test]
    fn todo_cycle_splices_are_byte_exact() {
        let source = "* TODO write it\nbody line\n* DONE other :x:\n";
        let doc = parse_org(source);
        let h0 = doc.headings()[0].clone();
        let step1 = cycle_todo(source, &h0);
        assert_eq!(step1, "* DONE write it\nbody line\n* DONE other :x:\n");
        let doc1 = parse_org(&step1);
        let step2 = cycle_todo(&step1, doc1.headings()[0]);
        assert_eq!(step2, "* write it\nbody line\n* DONE other :x:\n");
        // Cycling a keyword-less heading inserts the slot.
        let doc2 = parse_org(&step2);
        let step3 = cycle_todo(&step2, doc2.headings()[0]);
        assert_eq!(step3, "* TODO write it\nbody line\n* DONE other :x:\n");
    }

    #[test]
    fn checkbox_toggle_is_byte_exact_and_marks_uppercase() {
        let source = "- [ ] unchecked\n- [x] lowercase checked\n";
        let doc = parse_org(source);
        let items: Vec<CheckboxItem> = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                OrgNode::CheckboxItem(c) => Some(c.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(items.len(), 2);
        assert!(!items[0].checked);
        assert!(items[1].checked);
        assert_eq!(items[0].text, "unchecked");
        assert_eq!(
            toggle_checkbox(source, &items[0]),
            "- [X] unchecked\n- [x] lowercase checked\n"
        );
        assert_eq!(
            toggle_checkbox(source, &items[1]),
            "- [ ] unchecked\n- [ ] lowercase checked\n"
        );
    }

    #[test]
    fn src_blocks_and_drawers_are_typed_with_exact_bodies() {
        let source = "#+begin_src rust\nfn main() {}\nlet x = 1;\n#+end_src\n:PROPERTIES:\n:prop: 1\n:END:\n";
        let doc = parse_org(source);
        match (&doc.nodes[0], &doc.nodes[1]) {
            (OrgNode::SrcBlock(b), OrgNode::Drawer(d)) => {
                assert_eq!(b.language.as_deref(), Some("rust"));
                assert_eq!(b.body, "fn main() {}\nlet x = 1;\n");
                assert_eq!(
                    &source[b.range.clone()],
                    "#+begin_src rust\nfn main() {}\nlet x = 1;\n#+end_src\n"
                );
                assert_eq!(d.name, "PROPERTIES");
                assert_eq!(d.body, ":prop: 1\n");
            }
            other => panic!("expected src block + drawer, got {other:?}"),
        }
    }

    #[test]
    fn tables_are_typed_whole() {
        let source = "| a | b |\n|---+---|\n| 1 | 2 |\nafter\n";
        let doc = parse_org(source);
        match &doc.nodes[0] {
            OrgNode::Table(t) => {
                assert_eq!(t.lines, vec!["| a | b |", "|---+---|", "| 1 | 2 |"]);
                assert_eq!(t.lines.len(), 3);
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn unknown_constructs_stay_visible_as_text() {
        let source = "#+begin_quote\nquoted prose\n#+end_quote\n#+RESULTS: noise\n***\n";
        let doc = parse_org(source);
        let flat = format!("{doc:?}");
        // Nothing vanished — every line is present in the tree's ranges.
        let mut at = 0usize;
        for r in doc.leaf_ranges() {
            assert_eq!(r.start, at);
            at = r.end;
        }
        assert_eq!(at, source.len());
        // And none of them became anything other than Text.
        assert!(
            doc.headings().is_empty(),
            "stars-only line is not a headline"
        );
        assert!(flat.contains("Text"), "unknown constructs must be Text");
    }

    #[test]
    fn unterminated_blocks_degrade_to_visible_text_not_swallowed_regions() {
        let source = "#+begin_src rust\nfn main() {}\n";
        let doc = parse_org(source);
        assert!(
            doc.nodes.iter().all(|n| matches!(n, OrgNode::Text { .. })),
            "unterminated src stays visible text: {doc:?}"
        );
        let mut at = 0;
        for r in doc.leaf_ranges() {
            assert_eq!(r.start, at);
            at = r.end;
        }
        assert_eq!(at, source.len());
    }

    #[test]
    fn crlf_and_multibyte_titles_round_trip() {
        let source = "* TODO 写作 :中文:\r\n正文 héllo\r\n- [ ] 检查\r\n";
        let doc = parse_org(source);
        let mut at = 0usize;
        for r in doc.leaf_ranges() {
            assert_eq!(r.start, at);
            at = r.end;
        }
        assert_eq!(at, source.len());
        let h = doc.headings()[0];
        assert_eq!(h.todo.as_deref(), Some("TODO"));
        assert_eq!(h.tags, vec!["中文"]);
        let spliced = cycle_todo(source, h);
        assert!(spliced.starts_with("* DONE 写作 :中文:\r\n"), "{spliced:?}");
        let mut items: Vec<&CheckboxItem> = Vec::new();
        fn walk<'a>(nodes: &'a [OrgNode], out: &mut Vec<&'a CheckboxItem>) {
            for n in nodes {
                match n {
                    OrgNode::CheckboxItem(c) => out.push(c),
                    OrgNode::Heading(h) => walk(&h.body, out),
                    _ => {}
                }
            }
        }
        walk(&doc.nodes, &mut items);
        assert_eq!(items.len(), 1);
        assert_eq!(
            toggle_checkbox(source, items[0]),
            "* TODO 写作 :中文:\r\n正文 héllo\r\n- [X] 检查\r\n"
        );
    }
}
