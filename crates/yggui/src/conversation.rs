// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The CONVERSATION surface: an agent transcript rendered as a document.
//!
//! This module owns the *shape* of a conversation — the reading column, the
//! asymmetry between what a person asked and what the machine answered, the
//! quiet seam where tool work collects, the metadata type scale, and the tokens
//! every one of those reads its colour from. It deliberately does **not** own
//! how a message BODY is rendered: each host keeps its own markdown/content
//! pipeline and hands it in as an `Element`. That is the seam that actually
//! needs sharing, and drawing it here is what lets one design language cover a
//! terminal's JSONL web view and a chat app's agent pipeline without either
//! importing the other's content model.
//!
//! **Platform-neutral on purpose.** Nothing here touches `dioxus::desktop`,
//! `tao`, or the filesystem, so the same components compile for web, desktop
//! and mobile targets. Keep it that way — a single desktop import here would
//! quietly make the design language desktop-only.
//!
//! ## The rules the components encode
//!
//! - **One reading column.** Prose that runs the width of a maximised window is
//!   not readable; the column is capped and centred and everything shares it.
//! - **A person asks, the document answers.** The user's turn is a bounded card
//!   set against the page; the assistant's turn IS the page. Two facing bubbles
//!   is a messenger idiom, and it makes a 900-line answer look like a text.
//! - **Work is context, not content.** Consecutive tool calls and thinking
//!   collect into ONE quiet card below the prose weight, folded, each row
//!   identifying its own call so folding does not cost recognition.
//! - **Three faces, one meaning each.** Mono is the machine, sans is the
//!   person, serif is the answer. A face never changes for decoration.
//! - **No component invents a colour.** Everything comes from
//!   [`ConversationTokens`], so a host retheming its shell retheme this too.
//!
//! ⚠ Every `style` helper here emits a FIXED property-key set and varies only
//! values. Dioxus applies `style` property-by-property and never clears a key a
//! later render omits, so a branch that drops one leaves the previous branch's
//! value painted.

use dioxus::prelude::*;

/// Hover-reveal and row-hover behaviour, which inline styles cannot express.
///
/// Emitted once by [`ConversationColumn`]; a host that composes the pieces
/// itself can inline this constant instead.
pub const CONVERSATION_CSS: &str = r#"
.yggui-conv-actions {
  opacity: 0;
  transition: opacity 160ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-conv-turn:hover .yggui-conv-actions,
.yggui-conv-turn:focus-within .yggui-conv-actions {
  opacity: 1;
}
@media (hover: none) {
  .yggui-conv-actions { opacity: 1; }
}
.yggui-conv-work-row {
  transition: background-color 140ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-conv-work-row:hover,
.yggui-conv-work-row:focus-visible {
  background-color: var(--yggui-conv-row-hover);
}
.yggui-conv-quiet-button {
  transition: color 140ms cubic-bezier(0.2, 0, 0, 1),
    background-color 140ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-conv-quiet-button:hover {
  color: var(--yggui-conv-ink);
  background-color: var(--yggui-conv-row-hover);
}
@keyframes yggui-conv-pulse {
  0%, 100% { opacity: 0.25; transform: scale(0.82); }
  50% { opacity: 0.9; transform: scale(1); }
}
.yggui-conv-pulse-dot {
  animation: yggui-conv-pulse 1200ms ease-in-out infinite;
}
"#;

/// The reading column's width, in CSS pixels.
///
/// 720 is a measure of roughly 78 characters at the prose size, which is the
/// upper end of comfortable. It is a token rather than a literal because the
/// user card, the work card and the divider all have to agree with it — three
/// call sites spelling 720 is how a column starts drifting.
pub const CONVERSATION_COLUMN_PX: u32 = 720;

/// The assistant run's left inset. Shared by the answer and its work so the
/// machine's side of the page has exactly one left edge; see
/// [`ConversationTokens::run_inset_px`].
pub const CONVERSATION_RUN_INSET_PX: u32 = 14;

/// How many changed-file chips a work row draws before it counts the rest.
pub const CHANGED_FILE_CHIP_LIMIT: usize = 4;

/// How many rows a work group shows before it offers "show more".
pub const WORK_GROUP_COLLAPSED_ROWS: usize = 6;

/// Every colour, face and measure the conversation surface draws with.
///
/// Built from the host's own palette so the transcript is unmistakably part of
/// the app around it: `ink`, `muted` and `accent` come straight from the host,
/// and everything else is derived per theme. A host with a different accent
/// gets a different conversation for free, and no component anywhere below is
/// allowed to spell a colour of its own.
#[derive(Clone, Copy, PartialEq)]
pub struct ConversationTokens {
    pub is_dark: bool,
    /// Body ink — the host's own text colour.
    pub ink: &'static str,
    /// The host's secondary ink, used for de-emphasised prose.
    pub muted: &'static str,
    /// The host's accent, used for the live/streaming signal and links.
    pub accent: &'static str,
    /// Metadata ink. Deliberately NOT the host's `muted`: a shell's muted has
    /// to stay legible as body copy, and timestamps at that weight shout.
    pub meta: &'static str,
    /// The universal hairline.
    pub hairline: &'static str,
    /// The person's card.
    pub ask_surface: &'static str,
    pub ask_hairline: &'static str,
    pub ask_shadow: &'static str,
    /// The work seam.
    pub work_surface: &'static str,
    pub work_hairline: &'static str,
    /// A work row's own name — one step below prose ink, because a run of
    /// forty tool names at full body contrast competes with the answer it sits
    /// under, which is the exact thing the seam exists to prevent.
    pub work_ink: &'static str,
    /// The well an expanded work row prints its output into.
    pub well_surface: &'static str,
    /// A changed-file chip.
    pub chip_surface: &'static str,
    pub chip_hairline: &'static str,
    /// The tint a hoverable row takes.
    pub row_hover: &'static str,
    /// The composer's fill. A step ABOVE the ask card rather than equal to it:
    /// the box you write into has to read as the live object on a page whose
    /// other cards are a record of what was already said.
    pub composer_surface: &'static str,
    /// Ink on the filled send button, which is `accent` — so this is the one
    /// colour that must contrast with the accent rather than with the page.
    pub send_glyph: &'static str,
    /// The diff stat's pair. `removed` doubles as the failed-call ink: both
    /// mean "this went away", and a third red in one row reads as a third
    /// meaning. Neither is the status vocabulary's `RED`, which is reserved for
    /// a dead runtime.
    pub added: &'static str,
    pub removed: &'static str,
    /// The answer's face.
    pub prose_font: &'static str,
    /// The person's face, and every label and control.
    pub ui_font: &'static str,
    /// The machine's face.
    pub mono_font: &'static str,
    /// The reading column, in px.
    pub column_px: u32,
    /// The assistant run's left inset, in px — ONE left edge for the machine's
    /// side of the conversation.
    ///
    /// The answer needs it to hold the live rule without the text sliding
    /// sideways when streaming ends, and its WORK has to sit on the same line,
    /// or the column's left margin jogs on every prose↔work alternation. A real
    /// transcript alternates on the order of a hundred times, so a 14px
    /// disagreement between two components reads as a ragged edge down the
    /// whole page — measured on the live host at prose x=643 against work
    /// x=630 before this was a shared number.
    pub run_inset_px: u32,
}

/// The light/dark values as CSS, for a host whose theme is resolved by the
/// STYLESHEET rather than by Rust.
///
/// Not every app can answer `is_dark` in Rust, and the ones that cannot are not
/// doing anything wrong: an app offering a **System** theme setting has
/// delegated the answer to `prefers-color-scheme`, which only CSS can see. Such
/// a host takes [`ConversationTokens::from_css_variables`] and includes this
/// sheet, and gets light, dark AND system for free — the same values, chosen a
/// layer down.
///
/// The dark arm is written twice on purpose: once under `prefers-color-scheme`
/// for System, and once under an explicit `.dark` / `[data-theme="dark"]` root
/// so an app-level override still wins over the OS.
pub const CONVERSATION_THEME_CSS: &str = r#"
:root {
  --yggui-conv-meta: #8b96a2;
  --yggui-conv-hairline: rgba(20,32,44,0.10);
  --yggui-conv-ask-surface: #eef2f7;
  --yggui-conv-ask-hairline: rgba(20,32,44,0.09);
  --yggui-conv-ask-shadow: 0 8px 20px rgba(90,116,140,0.10);
  --yggui-conv-work-surface: rgba(20,32,44,0.028);
  --yggui-conv-work-hairline: rgba(20,32,44,0.08);
  --yggui-conv-work-ink: #43525f;
  --yggui-conv-well-surface: rgba(20,32,44,0.045);
  --yggui-conv-chip-surface: rgba(255,255,255,0.86);
  --yggui-conv-chip-hairline: rgba(20,32,44,0.10);
  --yggui-conv-row-hover: rgba(20,32,44,0.045);
  --yggui-conv-composer-surface: #ffffff;
  --yggui-conv-send-glyph: #ffffff;
  --yggui-conv-added: #2f7d55;
  --yggui-conv-removed: #b4525f;
}
@media (prefers-color-scheme: dark) {
  :root:not(.light):not([data-theme="light"]) {
    --yggui-conv-meta: #8595a5;
    --yggui-conv-hairline: rgba(190,214,238,0.12);
    --yggui-conv-ask-surface: rgba(255,255,255,0.082);
    --yggui-conv-ask-hairline: rgba(190,214,238,0.19);
    --yggui-conv-ask-shadow: 0 10px 26px rgba(0,0,0,0.28);
    --yggui-conv-work-surface: rgba(255,255,255,0.032);
    --yggui-conv-work-hairline: rgba(190,214,238,0.10);
    --yggui-conv-work-ink: #b7c5d4;
    --yggui-conv-well-surface: rgba(0,0,0,0.24);
    --yggui-conv-chip-surface: rgba(255,255,255,0.07);
    --yggui-conv-chip-hairline: rgba(190,214,238,0.12);
    --yggui-conv-row-hover: rgba(255,255,255,0.055);
    --yggui-conv-composer-surface: #1d242c;
    --yggui-conv-send-glyph: #0e1418;
    --yggui-conv-added: #5fbf88;
    --yggui-conv-removed: #e08594;
  }
}
:root.dark, :root[data-theme="dark"] {
  --yggui-conv-meta: #8595a5;
  --yggui-conv-hairline: rgba(190,214,238,0.12);
  --yggui-conv-ask-surface: rgba(255,255,255,0.082);
  --yggui-conv-ask-hairline: rgba(190,214,238,0.19);
  --yggui-conv-ask-shadow: 0 10px 26px rgba(0,0,0,0.28);
  --yggui-conv-work-surface: rgba(255,255,255,0.032);
  --yggui-conv-work-hairline: rgba(190,214,238,0.10);
  --yggui-conv-work-ink: #b7c5d4;
  --yggui-conv-well-surface: rgba(0,0,0,0.24);
  --yggui-conv-chip-surface: rgba(255,255,255,0.07);
  --yggui-conv-chip-hairline: rgba(190,214,238,0.12);
  --yggui-conv-row-hover: rgba(255,255,255,0.055);
  --yggui-conv-composer-surface: #1d242c;
  --yggui-conv-send-glyph: #0e1418;
  --yggui-conv-added: #5fbf88;
  --yggui-conv-removed: #e08594;
}
"#;

impl ConversationTokens {
    /// The same design language, addressed through CSS custom properties.
    ///
    /// For a host whose theme is decided in the stylesheet — anything offering
    /// a **System** setting, because `prefers-color-scheme` is not visible from
    /// Rust. Pair it with [`CONVERSATION_THEME_CSS`] (or supply the same
    /// variable names yourself, which is how an app keeps its own brand tint
    /// while inheriting every other decision here).
    ///
    /// `ink`, `muted` and `accent` are still the host's, and may themselves be
    /// `var(--…)` — they are only ever interpolated into a style string.
    ///
    /// ⚠ `is_dark` reads `false` on these tokens and MUST NOT be used to decide
    /// anything: the answer genuinely is not known in Rust here. Nothing in this
    /// module branches on it — it is carried for the host's own `data-` stamps.
    pub fn from_css_variables(
        ink: &'static str,
        muted: &'static str,
        accent: &'static str,
    ) -> Self {
        Self {
            is_dark: false,
            ink,
            muted,
            accent,
            meta: "var(--yggui-conv-meta)",
            hairline: "var(--yggui-conv-hairline)",
            ask_surface: "var(--yggui-conv-ask-surface)",
            ask_hairline: "var(--yggui-conv-ask-hairline)",
            ask_shadow: "var(--yggui-conv-ask-shadow)",
            work_surface: "var(--yggui-conv-work-surface)",
            work_hairline: "var(--yggui-conv-work-hairline)",
            work_ink: "var(--yggui-conv-work-ink)",
            well_surface: "var(--yggui-conv-well-surface)",
            chip_surface: "var(--yggui-conv-chip-surface)",
            chip_hairline: "var(--yggui-conv-chip-hairline)",
            row_hover: "var(--yggui-conv-row-hover)",
            composer_surface: "var(--yggui-conv-composer-surface)",
            send_glyph: "var(--yggui-conv-send-glyph)",
            added: "var(--yggui-conv-added)",
            removed: "var(--yggui-conv-removed)",
            prose_font: PROSE_FONT,
            ui_font: UI_FONT,
            mono_font: MONO_FONT,
            column_px: CONVERSATION_COLUMN_PX,
            run_inset_px: CONVERSATION_RUN_INSET_PX,
        }
    }

    /// Derive the surface from a host palette.
    ///
    /// `ink`, `muted` and `accent` are the host's; every surface, hairline and
    /// tint below is derived from the theme so the two never disagree about
    /// what "a step above the page" means.
    pub fn from_palette(
        is_dark: bool,
        ink: &'static str,
        muted: &'static str,
        accent: &'static str,
    ) -> Self {
        if is_dark {
            Self {
                is_dark,
                ink,
                muted,
                accent,
                meta: "#8595a5",
                hairline: "rgba(190,214,238,0.12)",
                ask_surface: "rgba(255,255,255,0.082)",
                ask_hairline: "rgba(190,214,238,0.19)",
                ask_shadow: "0 10px 26px rgba(0,0,0,0.28)",
                work_surface: "rgba(255,255,255,0.032)",
                work_hairline: "rgba(190,214,238,0.10)",
                work_ink: "#b7c5d4",
                well_surface: "rgba(0,0,0,0.24)",
                chip_surface: "rgba(255,255,255,0.07)",
                chip_hairline: "rgba(190,214,238,0.12)",
                row_hover: "rgba(255,255,255,0.055)",
                composer_surface: "#1d242c",
                send_glyph: "#0e1418",
                added: "#5fbf88",
                removed: "#e08594",
                prose_font: PROSE_FONT,
                ui_font: UI_FONT,
                mono_font: MONO_FONT,
                column_px: CONVERSATION_COLUMN_PX,
            run_inset_px: CONVERSATION_RUN_INSET_PX,
            }
        } else {
            Self {
                is_dark,
                ink,
                muted,
                accent,
                meta: "#8b96a2",
                hairline: "rgba(20,32,44,0.10)",
                ask_surface: "#eef2f7",
                ask_hairline: "rgba(20,32,44,0.09)",
                ask_shadow: "0 8px 20px rgba(90,116,140,0.10)",
                work_surface: "rgba(20,32,44,0.028)",
                work_hairline: "rgba(20,32,44,0.08)",
                work_ink: "#43525f",
                well_surface: "rgba(20,32,44,0.045)",
                chip_surface: "rgba(255,255,255,0.86)",
                chip_hairline: "rgba(20,32,44,0.10)",
                row_hover: "rgba(20,32,44,0.045)",
                composer_surface: "#ffffff",
                send_glyph: "#ffffff",
                added: "#2f7d55",
                removed: "#b4525f",
                prose_font: PROSE_FONT,
                ui_font: UI_FONT,
                mono_font: MONO_FONT,
                column_px: CONVERSATION_COLUMN_PX,
            run_inset_px: CONVERSATION_RUN_INSET_PX,
            }
        }
    }

    /// The custom properties the stylesheet reads. Emitted on the column so a
    /// `:hover` rule can reach a themed colour that inline styles cannot.
    fn css_variables(&self) -> String {
        format!(
            "--yggui-conv-row-hover:{}; --yggui-conv-ink:{};",
            self.row_hover, self.ink
        )
    }
}

const PROSE_FONT: &str = "\"Source Serif 4\", \"Noto Serif\", \"Iowan Old Style\", Georgia, serif";
const UI_FONT: &str = "\"Inter Variable\", \"Inter\", system-ui, sans-serif";
const MONO_FONT: &str = "\"JetBrains Mono\", \"Iosevka Term\", ui-monospace, monospace";

/// What a work row DID, which is what its mark draws.
///
/// Keyed to the action, never to the tool's name, so a CLI calling its shell
/// tool `exec_command` and one calling it `Bash` wear the same mark.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorkMark {
    Command,
    FileChange,
    FileRead,
    Search,
    Thinking,
    Generic,
}

impl WorkMark {
    /// The stroked path, on a shared 15x15 box, in `currentColor`.
    fn path(self) -> &'static str {
        match self {
            WorkMark::Command => "M3.2 4.4 6 7.2 3.2 10M7.4 10.4h4.6",
            WorkMark::FileChange => "M4 11.6h2.2l5-5a1.2 1.2 0 0 0-1.7-1.7l-5 5V11.6Z",
            WorkMark::FileRead => {
                "M1.8 7.5S3.9 3.9 7.5 3.9s5.7 3.6 5.7 3.6-2.1 3.6-5.7 3.6S1.8 7.5 1.8 7.5Z"
            }
            WorkMark::Search => {
                "M6.8 10.4a3.6 3.6 0 1 0 0-7.2 3.6 3.6 0 0 0 0 7.2ZM9.6 9.6l2.6 2.6"
            }
            WorkMark::Thinking => {
                "M3 8.6a2 2 0 0 1 1.2-3.7 2.6 2.6 0 0 1 5-.5 2 2 0 0 1 .4 3.9M5 11.4h5"
            }
            WorkMark::Generic => {
                "M10.6 3.4a2.8 2.8 0 0 0-3.5 3.5l-3.6 3.6 1.4 1.4 3.6-3.6a2.8 2.8 0 0 0 3.5-3.5L10.1 6.4 8.6 4.9Z"
            }
        }
    }
}

/// The folded work row's style.
///
/// ⚠ ONE owner, and every branch emits the IDENTICAL property-key set. Dioxus
/// applies `style` property-by-property and never clears a key a later render
/// omits, so a branch that drops one leaves the previous branch's value painted
/// — and these rows are recycled by a virtual window, which means a row that
/// once failed would keep the failure ink forever.
fn work_row_style(tokens: &ConversationTokens, failed: bool, has_body: bool) -> String {
    format!(
        "display:flex; align-items:center; gap:8px; width:100%; box-sizing:border-box; \
         padding:4px 6px; border:none; border-radius:7px; background:transparent; \
         text-align:left; cursor:{}; color:{}; font-family:{}; font-size:11px; line-height:1.55;",
        if has_body { "pointer" } else { "default" },
        if failed { tokens.removed } else { tokens.meta },
        tokens.mono_font,
    )
}

/// A single hover-revealed control beside a turn — copy, revert, retry.
///
/// The host owns what the control MEANS; this module owns only that it is
/// quiet, that it appears on hover, and that it never moves the turn's layout
/// when it does (it is always in flow, only its opacity changes).
#[derive(Clone, PartialEq)]
pub struct TurnAction {
    pub label: String,
    pub on_activate: EventHandler<()>,
}

// ---------------------------------------------------------------------------
// The column
// ---------------------------------------------------------------------------

/// The reading column every turn shares, and the one place the stylesheet and
/// the theme variables are emitted.
#[component]
pub fn ConversationColumn(
    tokens: ConversationTokens,
    /// Extra `data-*` value the host stamps for its own probes. Optional.
    #[props(default = String::new())]
    surface_id: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            "data-yggui-conversation": "1",
            "data-yggui-conversation-surface": "{surface_id}",
            "data-yggui-conversation-theme": if tokens.is_dark { "dark" } else { "light" },
            style: format!(
                "{} display:flex; flex-direction:column; align-items:stretch; gap:26px; \
                 width:min({}px, 100%); margin:0 auto; min-width:0; box-sizing:border-box; \
                 padding:4px 0 56px 0; font-family:{}; color:{};",
                tokens.css_variables(),
                tokens.column_px,
                tokens.ui_font,
                tokens.ink,
            ),
            style { {CONVERSATION_CSS} }
            {children}
        }
    }
}

// ---------------------------------------------------------------------------
// Turns
// ---------------------------------------------------------------------------

/// The person's turn: a bounded card, set against the page and right-aligned.
///
/// Bounded because an ask is usually short and a card that stretches the column
/// makes a one-line question look like an essay; right-aligned because that is
/// the one cue that survives at a glance when the answer below it has no card
/// at all. The bottom-right corner is flattened — the card points back at the
/// person who wrote it.
#[component]
pub fn UserTurn(
    tokens: ConversationTokens,
    /// Already formatted by the host. This module never parses or localises a
    /// time — a component that formats dates is a component with a locale bug.
    #[props(default = String::new())]
    timestamp: String,
    /// Hover-revealed controls, in the order given.
    #[props(default = Vec::new())]
    actions: Vec<TurnAction>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "yggui-conv-turn",
            "data-yggui-conv-turn": "user",
            style: "display:flex; justify-content:flex-end; width:100%; min-width:0;",
            div {
                style: format!(
                    "display:flex; flex-direction:column; gap:8px; min-width:0; \
                     max-width:min(78%, 560px); box-sizing:border-box; padding:13px 17px; \
                     border-radius:18px 18px 5px 18px; background:{}; border:1px solid {}; \
                     box-shadow:{}; color:{}; font-family:{}; font-size:14px; line-height:1.62; \
                     text-wrap:pretty;",
                    tokens.ask_surface,
                    tokens.ask_hairline,
                    tokens.ask_shadow,
                    tokens.ink,
                    tokens.ui_font,
                ),
                div {
                    style: "min-width:0; overflow-wrap:anywhere;",
                    {children}
                }
                TurnFooter { tokens, timestamp, actions, align: "flex-end" }
            }
        }
    }
}

/// The machine's turn: no card, no bubble, no avatar — the page itself.
///
/// This asymmetry is the whole design. An answer can be two lines or two
/// hundred; wrapping it in a facing bubble makes the long case unreadable and
/// the short case look like small talk. Serif, one step up in size, full
/// column.
#[component]
pub fn AssistantTurn(
    tokens: ConversationTokens,
    #[props(default = String::new())] timestamp: String,
    /// A second metadata term after the timestamp — elapsed time, model name.
    /// The host composes the words; this only places them.
    #[props(default = String::new())]
    meta: String,
    /// Draws the live rule down the left edge while the answer is still being
    /// written.
    #[props(default = false)]
    streaming: bool,
    #[props(default = Vec::new())] actions: Vec<TurnAction>,
    children: Element,
) -> Element {
    let footer_text = if meta.trim().is_empty() {
        timestamp.clone()
    } else if timestamp.trim().is_empty() {
        meta.clone()
    } else {
        format!("{timestamp} · {meta}")
    };
    rsx! {
        div {
            class: "yggui-conv-turn",
            "data-yggui-conv-turn": "assistant",
            "data-yggui-conv-streaming": if streaming { "1" } else { "0" },
            // ⚠ The live rule changes only its COLOUR. It used to change the
            // padding too, so every answer slid 12px sideways the moment it
            // finished — a jump under the reader's eye, on every turn, caused
            // by decoration.
            style: format!(
                "display:flex; flex-direction:column; gap:8px; width:100%; min-width:0; \
                 box-sizing:border-box; padding:0 2px 0 {}px; border-left:2px solid {};",
                tokens.run_inset_px,
                if streaming { tokens.accent } else { "transparent" },
            ),
            div {
                style: format!(
                    "min-width:0; overflow-wrap:anywhere; color:{}; font-family:{}; \
                     font-size:15.5px; line-height:1.68; text-wrap:pretty; \
                     font-feature-settings:'kern' 1, 'liga' 1;",
                    tokens.ink, tokens.prose_font,
                ),
                {children}
            }
            TurnFooter { tokens, timestamp: footer_text, actions, align: "flex-start" }
        }
    }
}

/// A system / notice turn: centred, quiet, never a card.
#[component]
pub fn SystemTurn(tokens: ConversationTokens, children: Element) -> Element {
    rsx! {
        div {
            class: "yggui-conv-turn",
            "data-yggui-conv-turn": "system",
            style: format!(
                "display:flex; justify-content:center; width:100%; min-width:0; \
                 color:{}; font-family:{}; font-size:12.5px; line-height:1.6; \
                 text-align:center; padding:0 24px; box-sizing:border-box;",
                tokens.meta, tokens.ui_font,
            ),
            div { style: "min-width:0; overflow-wrap:anywhere; max-width:100%;", {children} }
        }
    }
}

/// The metadata strip under a turn: the time, then whatever the host revealed
/// on hover. Nothing here is ever load-bearing enough to be always-on.
#[component]
fn TurnFooter(
    tokens: ConversationTokens,
    timestamp: String,
    actions: Vec<TurnAction>,
    align: &'static str,
) -> Element {
    if timestamp.trim().is_empty() && actions.is_empty() {
        return rsx! {};
    }
    rsx! {
        div {
            "data-yggui-conv-turn-footer": "1",
            style: format!(
                "display:flex; align-items:center; justify-content:{align}; gap:10px; \
                 min-width:0; font-family:{}; font-size:10px; letter-spacing:0.06em; \
                 font-variant-numeric:tabular-nums; color:{};",
                tokens.ui_font, tokens.meta,
            ),
            if !actions.is_empty() {
                div {
                    class: "yggui-conv-actions",
                    style: "display:flex; align-items:center; gap:4px; min-width:0;",
                    for action in actions.iter().cloned() {
                        QuietButton {
                            key: "{action.label}",
                            tokens,
                            label: action.label.clone(),
                            on_activate: action.on_activate,
                        }
                    }
                }
            }
            if !timestamp.trim().is_empty() {
                span {
                    "data-yggui-conv-timestamp": "1",
                    style: "min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                    "{timestamp}"
                }
            }
        }
    }
}

/// The one button shape this surface has: a text control that is nearly
/// invisible until it is wanted.
#[component]
pub fn QuietButton(
    tokens: ConversationTokens,
    label: String,
    on_activate: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            class: "yggui-conv-quiet-button",
            r#type: "button",
            title: "{label}",
            style: format!(
                "border:none; background:transparent; color:{}; font-family:{}; font-size:10px; \
                 font-weight:600; letter-spacing:0.06em; padding:2px 7px; border-radius:6px; \
                 cursor:pointer; white-space:nowrap;",
                tokens.meta, tokens.ui_font,
            ),
            onclick: move |_| on_activate.call(()),
            "{label}"
        }
    }
}

// ---------------------------------------------------------------------------
// The work seam
// ---------------------------------------------------------------------------

/// The card consecutive work rows collect into.
///
/// One card rather than N loose rows because the reader's question at this
/// point is "did it do anything unusual", not "what was step 7" — and a run of
/// forty ungrouped rows buries the prose on either side of it. The header
/// carries the count so the size of the run is legible without expanding it.
#[component]
pub fn WorkGroup(
    tokens: ConversationTokens,
    /// The run's noun — "Work", "Tool calls", "Thinking".
    label: String,
    /// Total rows in the run, including any the host is not drawing.
    count: usize,
    /// How many the host has hidden. Zero hides the control entirely.
    #[props(default = 0)]
    hidden_count: usize,
    #[props(default = false)] expanded: bool,
    on_toggle_group: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let show_control = hidden_count > 0 || (expanded && count > WORK_GROUP_COLLAPSED_ROWS);
    rsx! {
        div {
            "data-yggui-conv-work-group": "1",
            "data-yggui-conv-work-count": "{count}",
            // The run's work sits on the SAME left edge as the answer above
            // it (`run_inset_px`). `width:auto` rather than `100%`: the column
            // stretches its children, so a percentage width plus a margin
            // overflows by exactly the inset.
            style: format!(
                "display:flex; flex-direction:column; gap:2px; width:auto; min-width:0; \
                 margin-left:{}px; box-sizing:border-box; padding:8px 9px; \
                 border-radius:12px; background:{}; border:1px solid {};",
                tokens.run_inset_px, tokens.work_surface, tokens.work_hairline,
            ),
            div {
                style: format!(
                    "display:flex; align-items:center; justify-content:space-between; gap:10px; \
                     padding:1px 5px 5px 5px; min-width:0; font-family:{}; font-size:9.5px; \
                     font-weight:640; letter-spacing:0.16em; text-transform:uppercase; color:{};",
                    tokens.ui_font, tokens.meta,
                ),
                span {
                    style: "min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                    "{label} · {count}"
                }
                if show_control && let Some(toggle) = on_toggle_group {
                    QuietButton {
                        tokens,
                        label: if hidden_count > 0 { format!("Show {hidden_count} more") } else { "Show less".to_string() },
                        on_activate: move |_| toggle.call(()),
                    }
                }
            }
            {children}
        }
    }
}

/// One unit of machine work, folded to a line that identifies it.
///
/// The whole row is the control: expand/collapse is not something a reader
/// should have to hover to discover. The headline is user text of unbounded
/// length, so it ellipsizes on one line and keeps its tooltip rather than
/// wrapping the row into a paragraph — a row that reflows is a row that moves
/// every row under it while the eye is on it.
#[component]
pub fn WorkRow(
    tokens: ConversationTokens,
    mark: WorkMark,
    /// The tool's own name, never translated.
    label: String,
    /// What it acted on — the command, the path, the query.
    #[props(default = String::new())]
    headline: String,
    #[props(default = 0)] added_lines: usize,
    #[props(default = 0)] removed_lines: usize,
    /// A non-zero exit. Dimmed toward warning, never painted `RED`: `RED` means
    /// a dead runtime, and a failing command is a normal event in a live
    /// session.
    #[props(default = false)]
    failed: bool,
    #[props(default = true)] folded: bool,
    on_toggle: Option<EventHandler<()>>,
    /// The expanded body — output, a diff, chips. Absent for a row with
    /// nothing to show, which then draws as a plain line.
    expanded_body: Option<Element>,
) -> Element {
    let label_ink = if failed {
        tokens.removed
    } else {
        tokens.work_ink
    };
    let tooltip = if headline.trim().is_empty() {
        label.clone()
    } else {
        format!("{label} — {headline}")
    };
    let has_body = expanded_body.is_some() && on_toggle.is_some();
    let stat_visible = added_lines + removed_lines > 0;
    rsx! {
        div {
            "data-yggui-conv-work-row": "1",
            "data-yggui-conv-work-tool": "{label}",
            "data-yggui-conv-work-folded": if folded { "1" } else { "0" },
            style: "display:flex; flex-direction:column; gap:4px; width:100%; min-width:0;",
            button {
                class: "yggui-conv-work-row",
                r#type: "button",
                title: "{tooltip}",
                style: work_row_style(&tokens, failed, has_body),
                onclick: move |_| {
                    if let Some(toggle) = on_toggle {
                        toggle.call(());
                    }
                },
                span {
                    style: "display:inline-flex; align-items:center; justify-content:center; \
                            width:15px; height:15px; flex:0 0 auto; opacity:0.85;",
                    WorkMarkGlyph { mark }
                }
                span {
                    style: format!(
                        "flex:0 0 auto; font-weight:700; letter-spacing:0.01em; color:{label_ink};"
                    ),
                    "{label}"
                }
                span {
                    style: "flex:1 1 auto; min-width:0; overflow:hidden; text-overflow:ellipsis; \
                            white-space:nowrap; opacity:0.88;",
                    "{headline}"
                }
                if stat_visible {
                    DiffStat { tokens, added_lines, removed_lines }
                }
                span {
                    style: format!(
                        "flex:0 0 auto; width:10px; text-align:center; opacity:{};",
                        if has_body { "0.55" } else { "0" },
                    ),
                    {if folded { "▸" } else { "▾" }}
                }
            }
            if !folded && let Some(body) = expanded_body {
                div {
                    "data-yggui-conv-work-body": "1",
                    style: format!(
                        "display:flex; flex-direction:column; gap:7px; margin:0 0 4px 23px; \
                         padding:9px 11px; border-radius:9px; background:{}; color:{}; \
                         font-family:{}; font-size:11px; line-height:1.62; \
                         white-space:pre-wrap; overflow-wrap:anywhere;",
                        tokens.well_surface, tokens.ink, tokens.mono_font,
                    ),
                    {body}
                }
            }
        }
    }
}

/// `+N −M`. The two colours have one owner so the pair cannot drift.
#[component]
pub fn DiffStat(
    tokens: ConversationTokens,
    #[props(default = 0)] added_lines: usize,
    #[props(default = 0)] removed_lines: usize,
) -> Element {
    rsx! {
        span {
            "data-yggui-conv-diff-stat": "1",
            style: format!(
                "flex:0 0 auto; display:inline-flex; gap:6px; font-family:{}; font-size:11px; \
                 font-weight:700; font-variant-numeric:tabular-nums;",
                tokens.mono_font,
            ),
            span { style: format!("color:{};", tokens.added), "+{added_lines}" }
            span { style: format!("color:{};", tokens.removed), "−{removed_lines}" }
        }
    }
}

/// Changed files as chips: the trailing path, not the absolute one — every file
/// in a repo shares the leading part, which pushes the identifying end off the
/// edge. Caps at [`CHANGED_FILE_CHIP_LIMIT`] and then counts.
#[component]
pub fn ChangedFileChips(tokens: ConversationTokens, files: Vec<String>) -> Element {
    if files.is_empty() {
        return rsx! {};
    }
    let overflow = files.len().saturating_sub(CHANGED_FILE_CHIP_LIMIT);
    rsx! {
        div {
            "data-yggui-conv-changed-files": "{files.len()}",
            style: "display:flex; flex-wrap:wrap; gap:6px; min-width:0;",
            for path in files.iter().take(CHANGED_FILE_CHIP_LIMIT) {
                span {
                    key: "{path}",
                    title: "{path}",
                    style: format!(
                        "max-width:100%; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; \
                         padding:2px 7px; border-radius:6px; background:{}; border:1px solid {}; \
                         color:{}; font-family:{}; font-size:10px;",
                        tokens.chip_surface, tokens.chip_hairline, tokens.meta, tokens.mono_font,
                    ),
                    {changed_file_label(path)}
                }
            }
            if overflow > 0 {
                span {
                    style: format!(
                        "padding:2px 4px; color:{}; font-family:{}; font-size:10px;",
                        tokens.meta, tokens.mono_font,
                    ),
                    "+{overflow}"
                }
            }
        }
    }
}

/// The trailing two segments of a path — enough to identify a file, short
/// enough to sit in a chip.
pub fn changed_file_label(path: &str) -> String {
    let parts = path.rsplit('/').take(2).collect::<Vec<_>>();
    if parts.len() < 2 {
        return path.to_string();
    }
    format!("{}/{}", parts[1], parts[0])
}

/// The mark, on a shared 15x15 box, stroked in `currentColor` so the row's own
/// tone reaches the glyph without a second palette.
#[component]
pub fn WorkMarkGlyph(mark: WorkMark) -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 15 15",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.35",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: mark.path() }
        }
    }
}

// ---------------------------------------------------------------------------
// Rhythm and state
// ---------------------------------------------------------------------------

/// A hairline rule with a label riding it — the beat between two turns.
///
/// Used sparingly and only for something the reader would otherwise have to
/// count for themselves: where a response began, where a session was resumed,
/// where a day changed.
#[component]
pub fn TurnDivider(tokens: ConversationTokens, label: String) -> Element {
    rsx! {
        div {
            "data-yggui-conv-divider": "1",
            style: "display:flex; align-items:center; gap:12px; width:100%; min-width:0; padding:2px 0;",
            span { style: format!("flex:1 1 auto; height:1px; background:{};", tokens.hairline) }
            span {
                style: format!(
                    "flex:0 0 auto; padding:3px 10px; border-radius:999px; border:1px solid {}; \
                     color:{}; font-family:{}; font-size:9.5px; font-weight:640; \
                     letter-spacing:0.16em; text-transform:uppercase; white-space:nowrap;",
                    tokens.hairline, tokens.meta, tokens.ui_font,
                ),
                "{label}"
            }
            span { style: format!("flex:1 1 auto; height:1px; background:{};", tokens.hairline) }
        }
    }
}

/// The agent is mid-turn.
///
/// Three staggered dots and a sentence the host writes. It is deliberately a
/// row in the timeline rather than an overlay: it occupies the place the next
/// answer will, so nothing jumps when the answer arrives.
#[component]
pub fn WorkingIndicator(
    tokens: ConversationTokens,
    #[props(default = String::from("Working"))] label: String,
) -> Element {
    rsx! {
        div {
            "data-yggui-conv-working": "1",
            style: format!(
                "display:flex; align-items:center; gap:9px; min-width:0; padding:0 2px; \
                 color:{}; font-family:{}; font-size:11.5px; letter-spacing:0.02em;",
                tokens.meta, tokens.ui_font,
            ),
            span {
                style: "display:inline-flex; align-items:center; gap:4px;",
                for (index, delay) in [0u32, 180, 360].into_iter().enumerate() {
                    span {
                        key: "{index}",
                        class: "yggui-conv-pulse-dot",
                        style: format!(
                            "width:5px; height:5px; border-radius:999px; background:{}; \
                             animation-delay:{delay}ms;",
                            tokens.accent,
                        ),
                    }
                }
            }
            span { "{label}" }
        }
    }
}

/// Nothing to show yet. A conversation surface with no conversation should say
/// what would put one there, not apologise.
#[component]
pub fn ConversationEmptyState(
    tokens: ConversationTokens,
    headline: String,
    #[props(default = String::new())] detail: String,
) -> Element {
    rsx! {
        div {
            "data-yggui-conv-empty": "1",
            style: "display:flex; flex-direction:column; align-items:center; justify-content:center; \
                    gap:7px; width:100%; min-height:220px; padding:48px 24px; box-sizing:border-box; \
                    text-align:center;",
            div {
                style: format!(
                    "color:{}; font-family:{}; font-size:15px; line-height:1.5;",
                    tokens.ink, tokens.prose_font,
                ),
                "{headline}"
            }
            if !detail.trim().is_empty() {
                div {
                    style: format!(
                        "max-width:420px; color:{}; font-family:{}; font-size:12px; line-height:1.6;",
                        tokens.meta, tokens.ui_font,
                    ),
                    "{detail}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pair has one owner precisely so it cannot drift; if these ever
    /// differ per theme in a way that loses the "went away" reading, this is
    /// the test that should be argued with first.
    #[test]
    fn the_diff_pair_and_the_failure_ink_are_the_same_colour() {
        for is_dark in [false, true] {
            let tokens = ConversationTokens::from_palette(is_dark, "#000", "#111", "#222");
            assert_ne!(
                tokens.added, tokens.removed,
                "added and removed must be distinguishable"
            );
            // A failed row tints with `removed`; asserting the token exists and
            // is not the host ink is what stops a future edit reaching for a
            // third red.
            assert_ne!(tokens.removed, tokens.ink);
        }
    }

    /// The CSS arm and the Rust arm must cover the SAME token set. A variable
    /// the sheet never defines resolves to nothing and the property is simply
    /// dropped — an invisible hairline or a transparent card, with no error
    /// anywhere. This is the only thing that keeps the two arms in step.
    #[test]
    fn every_css_variable_token_is_defined_by_the_theme_sheet() {
        let css = ConversationTokens::from_css_variables("#111", "#666", "#2f7cf6");
        let referenced = [
            css.meta,
            css.hairline,
            css.ask_surface,
            css.ask_hairline,
            css.ask_shadow,
            css.work_surface,
            css.work_hairline,
            css.work_ink,
            css.well_surface,
            css.chip_surface,
            css.chip_hairline,
            css.row_hover,
            css.composer_surface,
            css.send_glyph,
            css.added,
            css.removed,
        ];
        for value in referenced {
            let name = value
                .trim_start_matches("var(")
                .trim_end_matches(')')
                .trim();
            assert!(value.starts_with("var(--"), "{value} must be a variable");
            assert!(
                CONVERSATION_THEME_CSS.contains(&format!("{name}:")),
                "{name} is referenced but never defined by CONVERSATION_THEME_CSS"
            );
            // Both the light default and BOTH dark arms must set it, or a
            // theme switch leaves one token behind on the other theme's value.
            assert_eq!(
                CONVERSATION_THEME_CSS.matches(&format!("{name}:")).count(),
                3,
                "{name} must be set in the light root and both dark arms"
            );
        }
    }

    #[test]
    fn a_chip_label_keeps_the_identifying_end_of_a_path() {
        assert_eq!(
            changed_file_label("/home/user/gh/yggterm/crates/yggui/src/conversation.rs"),
            "src/conversation.rs"
        );
        assert_eq!(changed_file_label("Cargo.toml"), "Cargo.toml");
        assert_eq!(changed_file_label("a/b"), "a/b");
    }

    /// Every mark must draw something. An empty path is an invisible row mark,
    /// which reads as a broken row rather than as a missing icon.
    #[test]
    fn every_work_mark_has_a_path() {
        for mark in [
            WorkMark::Command,
            WorkMark::FileChange,
            WorkMark::FileRead,
            WorkMark::Search,
            WorkMark::Thinking,
            WorkMark::Generic,
        ] {
            assert!(!mark.path().trim().is_empty(), "{mark:?} has no path");
        }
    }

    /// ★ The fixed-property-key invariant, for the work row.
    ///
    /// A failed call and a normal one differ only in VALUES. If the failed
    /// branch ever grows a key the normal branch lacks, a row that once failed
    /// keeps that property forever — and these rows are recycled by a virtual
    /// window, so the ghost lands on an unrelated call.
    #[test]
    fn a_work_rows_style_keys_do_not_move_with_its_state() {
        let keys = |style: &str| -> Vec<String> {
            let mut names: Vec<String> = style
                .split(';')
                .filter_map(|declaration| declaration.split(':').next())
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty())
                .collect();
            names.sort();
            names
        };
        for is_dark in [false, true] {
            let tokens = ConversationTokens::from_palette(is_dark, "#111", "#666", "#2f7cf6");
            let base = work_row_style(&tokens, false, true);
            for (failed, has_body) in [(true, true), (false, false), (true, false)] {
                assert_eq!(
                    keys(&base),
                    keys(&work_row_style(&tokens, failed, has_body)),
                    "dark={is_dark} failed={failed} has_body={has_body}"
                );
            }
            // …and the values DO differ, or the assertion above is vacuous.
            assert_ne!(base, work_row_style(&tokens, true, true));
            assert_ne!(base, work_row_style(&tokens, false, false));
        }
    }

    /// The stylesheet reaches theme colours through custom properties because a
    /// `:hover` rule cannot be written inline. If the variable names here and
    /// in `CONVERSATION_CSS` ever disagree, hover silently stops tinting — a
    /// failure with no error anywhere.
    #[test]
    fn the_stylesheet_variables_are_the_ones_the_column_emits() {
        let tokens = ConversationTokens::from_palette(true, "#fff", "#ccc", "#7cc8ff");
        let variables = tokens.css_variables();
        for name in ["--yggui-conv-row-hover", "--yggui-conv-ink"] {
            assert!(
                variables.contains(name),
                "the column must emit {name}: {variables}"
            );
            assert!(
                CONVERSATION_CSS.contains(name),
                "the stylesheet must read {name}"
            );
        }
    }
}
