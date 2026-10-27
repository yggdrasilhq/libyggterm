// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL-2.0/.

//! Page parsing through the DOCUMENTED path: the page is markdown parsed by
//! `emd_renderer::parse_markdown_blocks` (the same fold every host uses),
//! fences enumerated in document order (including inside lists/quotes),
//! fence byte ranges recovered from the markdown parser's own offset-iter
//! pass (which yields every CodeBlock in the same order), and line:col
//! derived from the opener's byte offset.

use crate::error::{ComponentLocation, PageError, PageItemError};
use emd_renderer::{MdBlock, parse_markdown_blocks, top_level_block_ranges};
use pulldown_cmark::CodeBlockKind;
use sha2::{Digest as _, Sha256};
use std::ops::Range;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut out = String::with_capacity(hash.len() * 2);
    for byte in hash {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// A page glued to its identity. Provenance law: a rendered artifact always
/// carries its source identity.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// Display path relative to the repo root (file-relative in bundles).
    pub path: String,
    pub source: String,
    /// Page revision label (from the manifest when present).
    pub revision: Option<String>,
}

impl Page {
    /// Content digest — the shortest honest identity of the page.
    pub fn content_sha256(&self) -> String {
        sha256_hex(self.source.as_bytes())
    }
}

/// One ```emd fence on the page — parsed component or actionable failure.
#[derive(Debug, Clone, PartialEq)]
pub struct PageComponent {
    pub location: ComponentLocation,
    /// Fully validated component, when the fence parsed.
    pub component: Option<emd_renderer::ComponentDocument>,
    /// Actionable parse/validation failure (never silently swallowed —
    /// the SAME message the rendered host displays, spec: visible error card).
    pub error: Option<String>,
    pub kind: Option<String>,
    /// How many missing observations (`null` slots, each MUST draw a gap)
    /// this component carries — the null-coercion detection surface.
    pub null_slots: usize,
    /// `true` when the fence is a TOP-LEVEL markdown block (its range is
    /// one of `top_level_block_ranges`'): the click-to-edit substrate.
    pub is_top_level_block: bool,
}

/// The parsed page: what a host loads.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPage {
    pub path: String,
    pub revision: Option<String>,
    pub content_sha256: String,
    pub blocks_total: usize,
    pub top_level_blocks: usize,
    pub components: Vec<PageComponent>,
}

impl ParsedPage {
    pub fn errors(&self) -> Vec<PageItemError> {
        self.components
            .iter()
            .filter_map(|c| {
                c.error.as_ref().map(|message| PageItemError {
                    location: c.location.clone(),
                    message: message.clone(),
                })
            })
            .collect()
    }

    pub fn is_valid(&self) -> bool {
        self.components.iter().all(|c| c.error.is_none())
    }

    pub fn fence_summary(&self) -> String {
        let ok = self.components.iter().filter(|c| c.error.is_none()).count();
        let bad = self.components.len() - ok;
        format!(
            "page {}: {} markdown blocks ({} top-level), {} emd fences: {ok} valid, {bad} invalid",
            self.path,
            self.blocks_total,
            self.top_level_blocks,
            self.components.len()
        )
    }
}

/// Parse a page through the documented path, adding host validation.
pub fn parse_page(page: &Page) -> Result<ParsedPage, PageError> {
    let blocks = parse_markdown_blocks(&page.source);
    let blocks_total = blocks.len();

    // Every component fence, in document order, with its tree path.
    let mut found: Vec<PageComponentInternal> = Vec::new();
    walk(&blocks, "blocks", &mut found);

    // The markdown parser's own offset-iter yields EVERY CodeBlock (with
    // range) in the same document order; the "emd" ones are exactly the
    // fences the fold produced components for. Zip the two orders.
    let fence_ranges: Vec<Range<usize>> = emd_codeblock_ranges(&page.source);
    if fence_ranges.len() != found.len() {
        return Err(PageError {
            message: format!(
                "page range mapping misaligned: {} emd fences in the offset-iter pass but {} in the document fold — this is a HOST defect (renderer range/fold drift), not a page defect; the page gives no verdicts until corrected",
                fence_ranges.len(),
                found.len()
            ),
        });
    }
    let top_level = top_level_block_ranges(&page.source);
    let components = found
        .into_iter()
        .zip(fence_ranges)
        .map(|(internal, range)| {
            let (line, col) = line_col(&page.source, range.start);
            let location = ComponentLocation {
                fence_index: internal.fence_index,
                byte_range: Some(range.clone()),
                line,
                col: col + 3, // the fence opener runs ``` wide
                block_path: internal.block_path,
            };
            let is_top_level_block = top_level.contains(&range);
            let null_slots = internal
                .component
                .as_ref()
                .map(|document| count_null_slots(&document.component))
                .unwrap_or(0);
            PageComponent {
                location,
                component: internal.component,
                error: internal.error,
                kind: internal.kind,
                null_slots,
                is_top_level_block,
            }
        })
        .collect();

    Ok(ParsedPage {
        path: page.path.clone(),
        revision: page.revision.clone(),
        content_sha256: page.content_sha256(),
        blocks_total,
        top_level_blocks: top_level.len(),
        components,
    })
}

struct PageComponentInternal {
    fence_index: usize,
    block_path: String,
    component: Option<emd_renderer::ComponentDocument>,
    error: Option<String>,
    kind: Option<String>,
}

fn walk(blocks: &[MdBlock], path: &str, found: &mut Vec<PageComponentInternal>) {
    for (index, block) in blocks.iter().enumerate() {
        match block {
            MdBlock::Component(document) => found.push(PageComponentInternal {
                fence_index: found.len(),
                block_path: format!("{path}[{index}]"),
                component: Some(document.clone()),
                error: None,
                kind: Some(kind_name(&document.component)),
            }),
            MdBlock::ComponentError { message, .. } => found.push(PageComponentInternal {
                fence_index: found.len(),
                block_path: format!("{path}[{index}]"),
                component: None,
                error: Some(message.clone()),
                kind: None,
            }),
            MdBlock::BlockQuote(inner) => walk(inner, &format!("{path}[{index}].quote"), found),
            MdBlock::List { items, .. } => {
                for (i, item) in items.iter().enumerate() {
                    walk(item, &format!("{path}[{index}].item[{i}]"), found);
                }
            }
            _ => {}
        }
    }
}

fn kind_name(component: &emd_renderer::EmdComponent) -> String {
    use emd_renderer::EmdComponent as C;
    match component {
        C::Grid(_) => "grid",
        C::Panel(_) => "panel",
        C::Plot(_) => "plot",
        C::Sparkline(_) => "sparkline",
        C::Metric(_) => "metric",
        C::Query(_) => "query",
        C::DataGrid(_) => "datagrid",
        C::AgentFinding(_) => "agentfinding",
    }
    .to_string()
}

fn emd_codeblock_ranges(source: &str) -> Vec<Range<usize>> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    // Options must EXACTLY match parse_markdown_blocks' own pass or the two
    // document-order walks could disagree.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let mut open_code_blocks = 0usize;
    for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                if open_code_blocks == 0 && is_emd(&kind) {
                    ranges.push(range.clone());
                }
                open_code_blocks += 1;
            }
            Event::End(TagEnd::CodeBlock) => open_code_blocks = open_code_blocks.saturating_sub(1),
            _ => {}
        }
    }
    ranges
}

fn is_emd(kind: &CodeBlockKind) -> bool {
    match kind {
        CodeBlockKind::Fenced(info) => info.split_whitespace().next() == Some("emd"),
        _ => false,
    }
}

pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let head = &source[..offset.min(source.len())];
    let line = head.matches('\n').count() + 1;
    let col = offset - (head.rfind('\n').map(|i| i + 1).unwrap_or(0));
    (line, col + 1)
}

fn count_null_slots(component: &emd_renderer::EmdComponent) -> usize {
    use emd_renderer::EmdComponent as C;
    match component {
        C::Plot(spec) => spec
            .series
            .iter()
            .map(|series| {
                series
                    .values
                    .iter()
                    .filter(|point| point.y.is_none())
                    .count()
            })
            .sum(),
        C::Sparkline(spec) => spec.values.iter().filter(|value| value.is_none()).count(),
        C::Grid(spec) => spec.children.iter().map(count_null_slots).sum(),
        C::Panel(spec) => spec.children.iter().map(count_null_slots).sum(),
        _ => 0,
    }
}
