// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The PROSE type system: ONE owner for how rendered markdown reads.
//!
//! `emd-renderer` answers *what* a document is — `MdBlock` and `MdInline`, a
//! platform-neutral tree with no opinion about faces. Every host then had to
//! answer *how it reads*, and three of them answered separately: a terminal's
//! Web View, a document reader and a chat app each spelled their own heading
//! scale, their own code face, their own paragraph rhythm. Three answers to one
//! question is how a "shared design language" quietly stops being shared — the
//! Web View was drawing code in `ui-monospace` while the token set beside it
//! named JetBrains Mono, and neither knew about the other.
//!
//! So the decisions live here, once, and a host adapter spells no literal. What
//! a host still owns is its BRAND: [`ProseInk`] carries the colours, because an
//! app's ink and accent are the one part of this that should differ.
//!
//! ## The three surfaces, and why they are not one
//!
//! - [`ProseTokens::document`] — a rendered markdown file. It owns its whole
//!   body type because nothing above it has decided: a legible sans at 16px,
//!   line-height 1.7 (user-directed 2026-07-18 "readability like The New York
//!   Times", refined 2026-07-23 to sans).
//! - [`ProseTokens::conversation`] — an agent transcript. Its body type is
//!   INHERITED, because the turn above it already decided and the two sides of
//!   a conversation are deliberately unequal: the person's ask is sans at 15px,
//!   the machine's answer is serif at 16px, and ONE markdown renderer serves
//!   both. Those two decisions are [`ProseBody::CONVERSATION_ASK`] and
//!   [`ProseBody::CONVERSATION_ANSWER`], here rather than in the components, so
//!   this file is the only place a type decision is made.
//! - [`ProseTokens::rail`] — markdown inside a 300px contributed pane. It keeps
//!   the interface face at the rail's own size; reading typography belongs to
//!   document-scale surfaces only.
//!
//! Everything below the body — headings, code, lists, tables, quotes, rules —
//! is shared by all three. A heading is a heading.
//!
//! ⚠ Every `*_style` helper emits a FIXED property-key set and varies only
//! values, including `inherit` where a surface declines to decide. Dioxus
//! applies `style` property-by-property and never clears a key a later render
//! omits, so a branch that drops one leaves the previous branch's value
//! painted.

/// The face an answer is set in. Serif is the machine's voice in a transcript:
/// one face, one meaning, never changed for decoration.
pub const PROSE_SERIF_STACK: &str =
    "\"Source Serif 4\", \"Noto Serif\", \"Iowan Old Style\", Georgia, serif";

/// The interface face — labels, controls, and the person's own words.
pub const UI_SANS_STACK: &str = "\"Inter Variable\", \"Inter\", system-ui, sans-serif";

/// The document reader's face. Deliberately NOT [`UI_SANS_STACK`]: a rendered
/// article falls back through the platform's reading sans (SF Pro Text, Segoe
/// UI, Noto Sans) rather than through the interface stack, so a machine without
/// Inter still reads like a page instead of like chrome. User-directed
/// 2026-07-23, after a serif pass was tried and rejected.
pub const READING_SANS_STACK: &str = "'Inter', 'SF Pro Text', 'Segoe UI', 'Noto Sans', \
     'Liberation Sans', 'Helvetica Neue', Arial, sans-serif";

/// The machine's face, everywhere. JetBrains Mono is the project-wide preferred
/// monospace; a host that spells `ui-monospace` instead is not overriding a
/// decision, it is missing one.
pub const MONO_STACK: &str = "\"JetBrains Mono\", \"Iosevka Term\", ui-monospace, monospace";

/// The reading column's width, in CSS pixels.
///
/// 720 is a measure of roughly 78 characters at the prose size, which is the
/// upper end of comfortable. It is a token rather than a literal because the
/// user card, the work card and the divider all have to agree with it — three
/// call sites spelling 720 is how a column starts drifting.
pub const PROSE_COLUMN_PX: u32 = 720;

/// Body copy: the four type properties that decide how a paragraph reads.
///
/// Every field is optional and `None` means **inherit** — not "unset". A
/// surface that sits inside something which has already chosen (a conversation
/// turn, a contributed rail pane) must not re-choose, or the two disagree and
/// the inner one silently wins. That was the live defect this type exists to
/// prevent: a transcript whose turns were set at line-height 1.72 rendered its
/// actual paragraphs at 1.55, because the markdown root re-decided.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProseBody {
    pub font: Option<&'static str>,
    pub size_px: Option<f32>,
    pub line_height: Option<f32>,
    pub tracking: Option<&'static str>,
}

impl ProseBody {
    /// Inherit every decision from the surface above.
    pub const INHERIT: Self = Self {
        font: None,
        size_px: None,
        line_height: None,
        tracking: None,
    };

    /// The machine's answer in a transcript. A hair of negative tracking: a
    /// serif at 16px sets a touch loose on a screen, and the answer is the one
    /// block on the page long enough for that to accumulate.
    pub const CONVERSATION_ANSWER: Self = Self {
        font: Some(PROSE_SERIF_STACK),
        size_px: Some(16.0),
        line_height: Some(1.72),
        tracking: Some("-0.003em"),
    };

    /// The person's ask. 15px rather than a full step below the answer — an ask
    /// is quieter than what it gets back, not smaller print.
    pub const CONVERSATION_ASK: Self = Self {
        font: Some(UI_SANS_STACK),
        size_px: Some(15.0),
        line_height: Some(1.6),
        tracking: None,
    };

    /// A rendered markdown document.
    pub const DOCUMENT: Self = Self {
        font: Some(READING_SANS_STACK),
        size_px: Some(16.0),
        line_height: Some(1.7),
        tracking: Some("0.002em"),
    };

    /// Markdown in a contributed rail pane: the caller's face and size, with
    /// the rail's tighter leading.
    pub const RAIL: Self = Self {
        font: None,
        size_px: None,
        line_height: Some(1.55),
        tracking: None,
    };

    /// The four properties as CSS. Always the same four keys.
    pub fn style(&self) -> String {
        format!(
            "font-family:{}; font-size:{}; line-height:{}; letter-spacing:{};",
            self.font.unwrap_or("inherit"),
            self.size_px.map(css_px).unwrap_or_else(inherit),
            self.line_height
                .map(|value| value.to_string())
                .unwrap_or_else(inherit),
            self.tracking.unwrap_or("inherit"),
        )
    }
}

/// One heading level.
///
/// Sizes are `em` so a heading tracks whatever body size the surface settled
/// on — the same scale reads correctly at a document's 16px and at a rail
/// pane's 11px, which is the whole reason a rail can share this table.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProseHeading {
    pub size_em: f32,
    pub weight: u16,
    pub tracking: &'static str,
    /// More air above than below: a heading belongs to what follows it, and
    /// the gap above is what separates it from what came before.
    pub space_above_px: u32,
    pub space_below_px: u32,
}

/// The host's brand, and the only part of prose a host decides.
///
/// Owned `String`s on purpose: a host's palette is usually runtime data (a
/// theme the user picked), not a `&'static str`, and forcing it to leak one
/// would push every consumer into a cache of its own.
#[derive(Clone, PartialEq, Debug)]
pub struct ProseInk {
    /// Body ink.
    pub ink: String,
    /// De-emphasised prose — a blockquote's text.
    pub muted: String,
    /// Links and the blockquote bar.
    pub accent: String,
    /// Rules, table separators, code borders.
    pub hairline: String,
    /// The fill behind code, inline and block.
    pub code_surface: String,
}

impl ProseInk {
    pub fn new(
        ink: impl Into<String>,
        muted: impl Into<String>,
        accent: impl Into<String>,
        hairline: impl Into<String>,
        code_surface: impl Into<String>,
    ) -> Self {
        Self {
            ink: ink.into(),
            muted: muted.into(),
            accent: accent.into(),
            hairline: hairline.into(),
            code_surface: code_surface.into(),
        }
    }
}

/// Every face, size and rhythm a rendered markdown surface draws with.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProseTokens {
    pub prose_font: &'static str,
    pub ui_font: &'static str,
    pub mono_font: &'static str,
    /// This surface's body copy — see [`ProseBody`] on why `None` means
    /// inherit.
    pub body: ProseBody,
    /// h1 through h4. Deeper levels reuse h4: past four, a document is
    /// outlining, not titling, and a fifth size is a size nobody can identify.
    pub headings: [ProseHeading; 4],
    pub paragraph_gap_px: u32,
    /// The gap around a block that is not a paragraph — code, table, quote.
    pub block_gap_px: u32,
    pub rule_gap_px: u32,
    pub list_indent_px: u32,
    pub list_item_gap_px: u32,
    pub quote_bar_px: u32,
    pub quote_inset_px: u32,
    pub inline_code_em: f32,
    pub inline_code_radius_px: u32,
    pub code_block_em: f32,
    pub code_block_line_height: f32,
    pub code_block_radius_px: u32,
    pub code_block_pad: &'static str,
    pub table_em: f32,
    pub table_cell_pad: &'static str,
    pub table_line_height: f32,
    pub image_max_width_px: u32,
    pub image_max_height_px: u32,
    pub image_radius_px: u32,
    /// The measure, for a surface that centres its own column.
    pub column_px: u32,
}

/// The rhythm every surface shares. Only [`ProseTokens::body`] and the faces
/// differ between presets; a heading is a heading.
const SHARED: ProseTokens = ProseTokens {
    prose_font: PROSE_SERIF_STACK,
    ui_font: UI_SANS_STACK,
    mono_font: MONO_STACK,
    body: ProseBody::INHERIT,
    // Heavy weights, negative tracking on the top two, NO rule underneath —
    // decoration the markdown did not ask for (user spec 2026-07-18).
    headings: [
        ProseHeading {
            size_em: 1.8,
            weight: 800,
            tracking: "-0.015em",
            space_above_px: 34,
            space_below_px: 16,
        },
        ProseHeading {
            size_em: 1.42,
            weight: 780,
            tracking: "-0.01em",
            space_above_px: 30,
            space_below_px: 14,
        },
        ProseHeading {
            size_em: 1.18,
            weight: 740,
            tracking: "0",
            space_above_px: 26,
            space_below_px: 12,
        },
        ProseHeading {
            size_em: 1.04,
            weight: 720,
            tracking: "0",
            space_above_px: 20,
            space_below_px: 10,
        },
    ],
    paragraph_gap_px: 14,
    block_gap_px: 14,
    rule_gap_px: 20,
    list_indent_px: 28,
    list_item_gap_px: 6,
    quote_bar_px: 3,
    quote_inset_px: 16,
    inline_code_em: 0.82,
    inline_code_radius_px: 4,
    code_block_em: 0.8,
    code_block_line_height: 1.55,
    code_block_radius_px: 8,
    code_block_pad: "12px 16px",
    table_em: 0.88,
    table_cell_pad: "8px 14px 8px 0",
    table_line_height: 1.55,
    image_max_width_px: 560,
    image_max_height_px: 320,
    image_radius_px: 14,
    column_px: PROSE_COLUMN_PX,
};

impl ProseTokens {
    /// A rendered markdown document — a file, an article, a reader pane.
    pub const fn document() -> Self {
        Self {
            body: ProseBody::DOCUMENT,
            ..SHARED
        }
    }

    /// An agent transcript. Inherits its body from the turn that wraps it, so
    /// the same renderer draws the person's sans ask and the machine's serif
    /// answer without either re-deciding.
    pub const fn conversation() -> Self {
        Self {
            body: ProseBody::INHERIT,
            ..SHARED
        }
    }

    /// Markdown inside a contributed rail pane. Interface face, the caller's
    /// size, tighter leading — reading typography applies to document-scale
    /// surfaces only.
    pub const fn rail() -> Self {
        Self {
            body: ProseBody::RAIL,
            ..SHARED
        }
    }

    /// The wrapper a rendered document sits in.
    pub fn root_style(&self) -> String {
        format!(
            "{} text-rendering:optimizeLegibility; font-feature-settings:'kern' 1, 'liga' 1;",
            self.body.style(),
        )
    }

    /// A heading at `level` (1-based; anything past four reuses h4).
    pub fn heading_style(&self, level: u8, ink: &ProseInk) -> String {
        let index = (level.clamp(1, 4) as usize) - 1;
        let heading = self.headings[index];
        format!(
            "font-size:{}; font-weight:{}; margin:{}px 0 {}px 0; letter-spacing:{}; \
             line-height:1.25; color:{};",
            css_em(heading.size_em),
            heading.weight,
            heading.space_above_px,
            heading.space_below_px,
            heading.tracking,
            ink.ink,
        )
    }

    pub fn paragraph_style(&self, ink: &ProseInk) -> String {
        format!(
            "margin:0 0 {}px 0; color:{};",
            self.paragraph_gap_px, ink.ink
        )
    }

    /// A link is distinguished by the accent alone — markdown has no underline
    /// syntax, so an underline is never ours to add (user spec).
    pub fn link_style(&self, ink: &ProseInk) -> String {
        format!("color:{}; text-decoration:none;", ink.accent)
    }

    pub fn inline_code_style(&self, ink: &ProseInk) -> String {
        format!(
            "background:{}; border:1px solid {}; border-radius:{}px; padding:1px 5px; \
             font-family:{}; font-size:{}; color:{};",
            ink.code_surface,
            ink.hairline,
            self.inline_code_radius_px,
            self.mono_font,
            css_em(self.inline_code_em),
            ink.ink,
        )
    }

    pub fn code_block_style(&self, ink: &ProseInk) -> String {
        format!(
            "background:{}; border:1px solid {}; border-radius:{}px; padding:{}; \
             overflow-x:auto; margin:{}px 0; font-family:{}; font-size:{}; \
             line-height:{}; color:{};",
            ink.code_surface,
            ink.hairline,
            self.code_block_radius_px,
            self.code_block_pad,
            self.block_gap_px,
            self.mono_font,
            css_em(self.code_block_em),
            self.code_block_line_height,
            ink.ink,
        )
    }

    /// The accent bar carries the quote; the text stays upright. Italicising a
    /// whole quote reads as decoration, and the bar already said it.
    ///
    /// ⚠ An accent LEFT BAR is the blockquote's vocabulary and nothing else's —
    /// never reuse it for editability or selection (user, 2026-07-23).
    pub fn blockquote_style(&self, ink: &ProseInk) -> String {
        format!(
            "border-left:{}px solid {}; margin:{}px 0; padding:2px 0 2px {}px; color:{};",
            self.quote_bar_px, ink.accent, self.block_gap_px, self.quote_inset_px, ink.muted,
        )
    }

    pub fn list_style(&self, ink: &ProseInk) -> String {
        format!(
            "margin:0 0 {}px 0; padding-left:{}px; color:{};",
            self.paragraph_gap_px, self.list_indent_px, ink.ink
        )
    }

    pub fn list_item_style(&self) -> String {
        format!("margin:{}px 0;", self.list_item_gap_px)
    }

    /// Wide tables scroll inside their own container; the document itself never
    /// scrolls horizontally.
    pub fn table_wrap_style(&self) -> String {
        format!("overflow-x:auto; margin:{}px 0;", self.block_gap_px)
    }

    pub fn table_style(&self) -> String {
        format!("border-collapse:collapse; font-size:{};", css_em(self.table_em))
    }

    /// Horizontal separators ONLY — no vertical grid, no header fill. A full
    /// cell grid reads as a spreadsheet; an article's table is rows of text
    /// with quiet rules, and the header's rule is the heavier of the two.
    pub fn table_head_cell_style(&self, ink: &ProseInk) -> String {
        self.table_cell(ink, 2, 700)
    }

    pub fn table_cell_style(&self, ink: &ProseInk) -> String {
        self.table_cell(ink, 1, 400)
    }

    fn table_cell(&self, ink: &ProseInk, rule_px: u32, weight: u16) -> String {
        format!(
            "border:0; border-bottom:{}px solid {}; padding:{}; text-align:left; \
             vertical-align:top; line-height:{}; font-weight:{}; color:{};",
            rule_px, ink.hairline, self.table_cell_pad, self.table_line_height, weight, ink.ink,
        )
    }

    pub fn rule_style(&self, ink: &ProseInk) -> String {
        format!(
            "border-top:1px solid {}; margin:{}px 0;",
            ink.hairline, self.rule_gap_px
        )
    }

    /// An image is DISPLAYED, not linked — the one place a transcript full of
    /// pasted screenshots differs from a document, and why `MdInline::Image` is
    /// a typed node rather than a glyph plus a link.
    pub fn image_style(&self) -> String {
        format!(
            "display:block; width:auto; max-width:min(100%, {}px); max-height:{}px; \
             border-radius:{}px; object-fit:contain;",
            self.image_max_width_px, self.image_max_height_px, self.image_radius_px,
        )
    }

    pub fn image_frame_style(&self) -> String {
        "display:block; margin:8px 0 4px 0;".to_string()
    }
}

fn inherit() -> String {
    "inherit".to_string()
}

/// `16.0` is `16px`, not `16px` spelled `16.0px` — a CSS length with a trailing
/// `.0` is valid and still looks like a bug in a DOM inspector.
fn css_px(value: f32) -> String {
    format!("{}px", trim_float(value))
}

fn css_em(value: f32) -> String {
    format!("{}em", trim_float(value))
}

fn trim_float(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink() -> ProseInk {
        ProseInk::new("#111", "#666", "#07c", "#ddd", "#f4f4f4")
    }

    /// The defect this module was written for: a surface that inherits must
    /// emit `inherit`, not a number of its own, or it silently overrides the
    /// turn that already decided.
    #[test]
    fn a_conversation_surface_decides_nothing_about_body_copy() {
        let style = ProseTokens::conversation().root_style();
        assert!(style.contains("font-size:inherit"), "{style}");
        assert!(style.contains("line-height:inherit"), "{style}");
        assert!(style.contains("font-family:inherit"), "{style}");
        assert!(style.contains("letter-spacing:inherit"), "{style}");
    }

    #[test]
    fn a_document_surface_decides_all_of_it() {
        let style = ProseTokens::document().root_style();
        assert!(style.contains("font-size:16px"), "{style}");
        assert!(style.contains("line-height:1.7"), "{style}");
        assert!(style.contains("letter-spacing:0.002em"), "{style}");
        assert!(style.contains("'Inter'"), "{style}");
    }

    /// A rail keeps the caller's face and size and changes only the leading.
    #[test]
    fn a_rail_surface_decides_only_its_leading() {
        let style = ProseTokens::rail().root_style();
        assert!(style.contains("font-size:inherit"), "{style}");
        assert!(style.contains("font-family:inherit"), "{style}");
        assert!(style.contains("line-height:1.55"), "{style}");
    }

    /// Every branch of `ProseBody::style` emits the SAME four keys, because
    /// Dioxus never clears a key a later render omits.
    #[test]
    fn every_body_emits_the_same_property_keys() {
        let keys = |body: ProseBody| {
            let mut names: Vec<String> = body
                .style()
                .split(';')
                .filter_map(|part| part.split_once(':').map(|(key, _)| key.trim().to_string()))
                .collect();
            names.sort();
            names
        };
        let expected = keys(ProseBody::DOCUMENT);
        assert_eq!(expected.len(), 4);
        for body in [
            ProseBody::INHERIT,
            ProseBody::RAIL,
            ProseBody::CONVERSATION_ASK,
            ProseBody::CONVERSATION_ANSWER,
        ] {
            assert_eq!(keys(body), expected);
        }
    }

    /// Code wears the project's monospace on every surface. The Web View drew
    /// `ui-monospace` for months while the token set beside it named JetBrains
    /// Mono; that is the class of drift this module exists to end.
    #[test]
    fn code_wears_the_project_monospace_on_every_surface() {
        for tokens in [
            ProseTokens::document(),
            ProseTokens::conversation(),
            ProseTokens::rail(),
        ] {
            assert!(tokens.inline_code_style(&ink()).contains("JetBrains Mono"));
            assert!(tokens.code_block_style(&ink()).contains("JetBrains Mono"));
        }
    }

    /// The rhythm is shared even though the body is not — a heading is a
    /// heading, whichever surface it lands on.
    #[test]
    fn the_scale_below_body_copy_is_identical_across_surfaces() {
        let document = ProseTokens::document();
        let conversation = ProseTokens::conversation();
        assert_eq!(document.headings, conversation.headings);
        assert_eq!(
            document.heading_style(2, &ink()),
            conversation.heading_style(2, &ink())
        );
        assert_eq!(
            document.paragraph_style(&ink()),
            conversation.paragraph_style(&ink())
        );
    }

    /// A heading past h4 reuses h4 rather than falling off the table.
    #[test]
    fn a_deep_heading_reuses_the_last_level() {
        let tokens = ProseTokens::document();
        assert_eq!(
            tokens.heading_style(6, &ink()),
            tokens.heading_style(4, &ink())
        );
        assert_eq!(
            tokens.heading_style(0, &ink()),
            tokens.heading_style(1, &ink())
        );
    }

    /// Headings carry more air above than below, at every level.
    #[test]
    fn a_heading_belongs_to_what_follows_it() {
        for heading in ProseTokens::document().headings {
            assert!(
                heading.space_above_px > heading.space_below_px,
                "{heading:?}"
            );
        }
    }

    #[test]
    fn a_css_length_never_carries_a_trailing_zero() {
        assert_eq!(css_px(16.0), "16px");
        assert_eq!(css_px(12.5), "12.5px");
        assert_eq!(css_em(1.8), "1.8em");
    }
}
