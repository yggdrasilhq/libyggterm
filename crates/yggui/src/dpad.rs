// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The SCROLL D-PAD: one four-way control, for every surface that scrolls.
//!
//! A terminal, an agent transcript and a document reader all have the same
//! problem — content taller than the viewport, a wheel that is too slow for it,
//! and a keyboard route (Home/End/PageUp/PageDown) that is invisible until
//! someone is told. They had two different answers to it, in one file, and only
//! one of them was any good: the terminal's is a 3×3 pad on a glass panel with
//! the arrows where a D-pad puts them, and the transcript's was a squashed 3×2
//! with the "go to bottom" arrow sitting in the hole where the pad's centre
//! belongs.
//!
//! ⚠ **The geometry is the point.** A D-pad is recognised by its SHAPE before
//! any glyph is read — up above, down below, the two page arrows either side of
//! a dead centre. Collapse it to two rows to save 30px and it stops being a
//! D-pad and becomes four buttons in a box, which is exactly how the second one
//! read. So this component owns the grid, and a host chooses only where it sits
//! and what the four actions do.
//!
//! ## What a host still owns
//!
//! - **What the actions MEAN.** This module never scrolls anything: it reports
//!   [`DpadAction`] and the host moves its own viewport. A terminal's "bottom"
//!   is the prompt, a transcript's is the newest turn, and a document's is the
//!   last line — three different things behind one gesture.
//! - **Where it sits**, via [`DpadPlacement`].
//! - **Its brand**, via [`DpadPalette`].

use dioxus::prelude::*;

/// The four-way vocabulary. **↑/↓ are the EDGES and ←/→ are PAGES** — learned
/// once, it works on every surface that mounts this pad.
///
/// The pairing is deliberate and it is not the obvious one. Up/down as pages
/// and left/right as edges reads more natural in the abstract, but a reader who
/// wants the top of a long transcript wants it in ONE press, and a reader
/// paging through it presses many times: the cheap, repeatable gesture belongs
/// on the horizontal axis where the thumb already rests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DpadAction {
    Top,
    PageUp,
    PageDown,
    Bottom,
}

impl DpadAction {
    /// A stable token for probes and for a host's own scripting. Snake case
    /// because it crosses into JS, where these become action names.
    pub fn token(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
            Self::Bottom => "bottom",
        }
    }

    /// The `data-*` value, in the kebab case DOM attributes use.
    pub fn dom_token(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::PageUp => "page-up",
            Self::PageDown => "page-down",
            Self::Bottom => "bottom",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Top => "↑",
            Self::PageUp => "←",
            Self::PageDown => "→",
            Self::Bottom => "↓",
        }
    }

    /// Where this action sits on the 3×3 grid, as `(column, row)`, 1-based.
    fn cell(self) -> (u8, u8) {
        match self {
            Self::Top => (2, 1),
            Self::PageUp => (1, 2),
            Self::PageDown => (3, 2),
            Self::Bottom => (2, 3),
        }
    }

    /// Reading order, so a host's tooltips and a probe's expectations agree.
    pub const ALL: [Self; 4] = [Self::Top, Self::PageUp, Self::PageDown, Self::Bottom];
}

/// Where the pad sits inside its (positioned) parent.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DpadPlacement {
    #[default]
    TopRight,
    BottomRight,
}

impl DpadPlacement {
    fn anchor_css(self) -> &'static str {
        match self {
            Self::TopRight => "top:12px; right:12px;",
            Self::BottomRight => "bottom:14px; right:14px;",
        }
    }
}

/// The pad's brand. Everything else — geometry, material, motion — is this
/// module's.
#[derive(Clone, PartialEq, Debug)]
pub struct DpadPalette {
    /// Glyph ink on a key.
    pub ink: String,
    /// The dead centre's ink, which is deliberately quieter than a key's.
    pub muted: String,
}

impl DpadPalette {
    pub fn new(ink: impl Into<String>, muted: impl Into<String>) -> Self {
        Self {
            ink: ink.into(),
            muted: muted.into(),
        }
    }
}

/// Hover-reveal, which an inline style cannot express. A host that wants the
/// pad always visible passes `hover_reveal: false` and can skip this sheet.
pub const DPAD_CSS: &str = r#"
[data-yggui-dpad][data-yggui-dpad-hover-reveal="1"] {
  opacity: 0;
  pointer-events: none;
  transition: opacity 120ms ease;
}
[data-yggui-dpad][data-yggui-dpad-hover-reveal="1"][data-yggui-dpad-visible="true"] {
  opacity: 1;
  pointer-events: auto;
}
[data-yggui-dpad] button {
  transition: background-color 120ms ease;
}
[data-yggui-dpad] button:hover {
  background: rgba(255, 255, 255, 0.26);
}
"#;

const KEY_PX: u32 = 26;

/// The four-way scroll control.
///
/// Mount it inside a `position:relative` parent. It swallows its own
/// `mousedown` and `click` so pressing a key never reaches the surface
/// underneath — on a terminal that would move the caret, and on a transcript it
/// would collapse a work row.
#[component]
pub fn ScrollDpad(
    palette: DpadPalette,
    /// Stamped as `data-yggui-dpad-surface` so a probe can tell two mounted
    /// pads apart.
    #[props(default = String::new())]
    surface_id: String,
    /// Start hidden and reveal on the host's own hover signal (it flips
    /// `data-yggui-dpad-visible`). `false` keeps it always on screen.
    #[props(default = true)]
    hover_reveal: bool,
    #[props(default = DpadPlacement::TopRight)] placement: DpadPlacement,
    on_action: EventHandler<DpadAction>,
) -> Element {
    let key_style = format!(
        "display:flex; align-items:center; justify-content:center; width:{KEY_PX}px; \
         height:{KEY_PX}px; border:none; border-radius:7px; background:rgba(255,255,255,0.16); \
         color:{}; font-size:15px; font-weight:800; line-height:1; padding:0; cursor:pointer; \
         box-shadow:inset 0 0 0 1px rgba(255,255,255,0.20);",
        palette.ink,
    );
    // The hole in the middle of a D-pad. It is not a button and never has been;
    // it is what makes the four keys read as one control rather than as four.
    let centre_style = format!(
        "display:flex; align-items:center; justify-content:center; width:{KEY_PX}px; \
         height:{KEY_PX}px; border-radius:7px; color:{}; font-size:13px; font-weight:800; \
         opacity:0.7;",
        palette.muted,
    );
    rsx! {
        div {
            "data-yggui-dpad": "1",
            "data-yggui-dpad-surface": "{surface_id}",
            "data-yggui-dpad-hover-reveal": if hover_reveal { "1" } else { "0" },
            "data-yggui-dpad-visible": if hover_reveal { "false" } else { "true" },
            style: format!(
                "position:absolute; {} z-index:8; display:grid; \
                 grid-template-columns:repeat(3, {KEY_PX}px); \
                 grid-template-rows:repeat(3, {KEY_PX}px); gap:4px; padding:6px; \
                 border-radius:8px; background:rgba(22,27,34,0.58); \
                 backdrop-filter:blur(12px) saturate(130%); \
                 box-shadow:inset 0 0 0 1px rgba(255,255,255,0.10), 0 12px 28px rgba(0,0,0,0.22);",
                placement.anchor_css(),
            ),
            // A press belongs to the pad, never to what is behind it.
            onmousedown: |evt| {
                evt.prevent_default();
                evt.stop_propagation();
            },
            onclick: |evt| {
                evt.prevent_default();
                evt.stop_propagation();
            },
            for action in DpadAction::ALL {
                button {
                    key: "{action.token()}",
                    r#type: "button",
                    "data-yggui-dpad-action": "{action.dom_token()}",
                    style: {
                        let (column, row) = action.cell();
                        format!("{key_style} grid-column:{column}; grid-row:{row};")
                    },
                    onclick: move |evt: MouseEvent| {
                        evt.prevent_default();
                        evt.stop_propagation();
                        on_action.call(action);
                    },
                    "{action.glyph()}"
                }
            }
            div {
                "data-yggui-dpad-centre": "1",
                style: "{centre_style} grid-column:2; grid-row:2;",
                "+"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ THE SHAPE IS THE CONTROL.
    ///
    /// A D-pad is recognised before a single glyph is read: up on top, down
    /// below, the page keys either side of a dead centre. The transcript's own
    /// version collapsed this to two rows and put "go to bottom" in the middle
    /// cell — four buttons in a box, which is what the user saw and called
    /// ugly. Nothing may occupy `(2, 2)` but the hole.
    #[test]
    fn the_pad_is_a_cross_with_a_hole_in_the_middle() {
        assert_eq!(DpadAction::Top.cell(), (2, 1));
        assert_eq!(DpadAction::Bottom.cell(), (2, 3));
        assert_eq!(DpadAction::PageUp.cell(), (1, 2));
        assert_eq!(DpadAction::PageDown.cell(), (3, 2));
        for action in DpadAction::ALL {
            assert_ne!(action.cell(), (2, 2), "{action:?} is standing in the hole");
        }
        // Four distinct cells, on three columns and three rows.
        let cells: std::collections::BTreeSet<_> =
            DpadAction::ALL.iter().map(|a| a.cell()).collect();
        assert_eq!(cells.len(), 4);
    }

    /// ↑/↓ are the edges and ←/→ are pages, on every surface. A host that
    /// re-pairs them breaks a gesture learned somewhere else in the same app.
    #[test]
    fn the_vertical_axis_is_the_edges_and_the_horizontal_is_pages() {
        assert_eq!(DpadAction::Top.glyph(), "↑");
        assert_eq!(DpadAction::Bottom.glyph(), "↓");
        assert_eq!(DpadAction::PageUp.glyph(), "←");
        assert_eq!(DpadAction::PageDown.glyph(), "→");
    }

    /// The DOM token and the script token differ in case, and both are stable.
    /// A probe keys on one and a host's JS on the other; letting either drift
    /// silently unbinds a key.
    #[test]
    fn every_action_carries_both_of_its_stable_tokens() {
        for action in DpadAction::ALL {
            assert!(!action.token().is_empty());
            assert!(!action.dom_token().is_empty());
            assert_eq!(action.token().replace('_', "-"), action.dom_token());
        }
    }
}
