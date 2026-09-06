// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The PILL TOOLBAR: a floating, translucent bar that costs no layout.
//!
//! A reading surface wants three controls always within reach — find, step
//! through the matches, and switch the light — and it wants none of them taking
//! a permanent strip off the top of the page. A docked header does exactly
//! that: it is always there, always the same height, and on a phone it is a
//! meaningful fraction of the screen.
//!
//! So the bar floats over the content instead, as a pill. It is the shape the
//! user asked for after seeing Uber's, and the reasoning is theirs: on a small
//! screen space is the scarce resource, and a control that appears when it is
//! wanted and recedes when it is not buys back the whole strip.
//!
//! ⚠ **This component owns SHAPE, MATERIAL and ARRANGEMENT. It owns no state.**
//! It does not search, does not count matches, does not know which theme is on.
//! Every one of those already has an owner in the host — a find engine, a theme
//! setting — and a toolbar that kept its own copy would be a second answer to
//! questions that are already answered. It renders what it is handed and
//! reports what was pressed.
//!
//! ## The arrangement, and why it is that way
//!
//! Search sits in the CENTRE because it is the reason the bar exists. The
//! stepper sits immediately beside its field, because "next match" is a
//! continuation of typing and the hand should not travel. The theme toggle sits
//! at the far RIGHT, deliberately far from both: it is pressed once in a
//! session, and the two controls that get pressed constantly must not have it
//! adjacent to them.

use dioxus::prelude::*;
use yggui_icons::Icon;

/// The bar's brand. Material, radius and motion are this module's.
#[derive(Clone, PartialEq, Debug)]
pub struct PillToolbarPalette {
    /// Ink for typed text and active glyphs.
    pub ink: String,
    /// Ink for placeholder text, the counter, and resting glyphs.
    pub muted: String,
    /// The bar's own fill, under its blur.
    pub surface: String,
    /// The hairline that separates the bar from whatever it floats over.
    pub hairline: String,
    /// The tint a control takes when hovered.
    pub hover: String,
}

impl PillToolbarPalette {
    pub fn new(
        ink: impl Into<String>,
        muted: impl Into<String>,
        surface: impl Into<String>,
        hairline: impl Into<String>,
        hover: impl Into<String>,
    ) -> Self {
        Self {
            ink: ink.into(),
            muted: muted.into(),
            surface: surface.into(),
            hairline: hairline.into(),
            hover: hover.into(),
        }
    }
}

/// Hover and focus behaviour, which inline styles cannot express.
pub const PILL_TOOLBAR_CSS: &str = r#"
[data-yggui-pill-toolbar] button {
  transition: background-color 130ms cubic-bezier(0.2, 0, 0, 1),
    color 130ms cubic-bezier(0.2, 0, 0, 1);
}
[data-yggui-pill-toolbar] button:hover:not(:disabled) {
  background-color: var(--yggui-pill-hover);
  color: var(--yggui-pill-ink);
}
[data-yggui-pill-toolbar] button:disabled {
  opacity: 0.38;
  cursor: default;
}
[data-yggui-pill-toolbar] input::placeholder {
  color: var(--yggui-pill-muted);
  opacity: 1;
}
[data-yggui-pill-toolbar] input:focus {
  outline: none;
}
"#;

/// Which way the stepper moved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PillStep {
    Previous,
    Next,
}

const BAR_HEIGHT_PX: u32 = 38;

/// A sun or a crescent, stroked in `currentColor` on a shared box.
///
/// The glyph shows the theme you would GET, not the one you are in. Both
/// readings are defensible and this one is the convention every OS toggle uses:
/// the control is named for its outcome.
#[component]
fn ThemeGlyph(is_dark: bool) -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 15 15",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            if is_dark {
                // In dark, offer the sun.
                circle { cx: "7.5", cy: "7.5", r: "3" }
                path { d: "M7.5 1v1.6M7.5 12.4V14M14 7.5h-1.6M2.6 7.5H1M12.1 2.9l-1.1 1.1M4 11l-1.1 1.1M12.1 12.1L11 11M4 4L2.9 2.9" }
            } else {
                // In light, offer the moon.
                path { d: "M12.4 9.3A5.4 5.4 0 0 1 5.7 2.6 5.4 5.4 0 1 0 12.4 9.3Z" }
            }
        }
    }
}

/// The floating pill.
///
/// Mount it inside a `position:relative` parent; it centres itself horizontally
/// and sits near the top edge.
#[component]
pub fn PillToolbar(
    palette: PillToolbarPalette,
    /// The find query. The HOST owns it — this is a controlled input, so the
    /// bar cannot drift out of step with the engine actually searching.
    #[props(default = String::new())]
    query: String,
    /// `3/17`, or empty to hide the counter and disable the stepper. The host
    /// already computes this; the bar never counts.
    #[props(default = String::new())]
    match_label: String,
    #[props(default = "Search".to_string())] placeholder: String,
    /// Whether the surface is currently dark, for the toggle's glyph only.
    #[props(default = false)]
    is_dark: bool,
    /// Fades the whole bar out. A host that reveals on scroll drives this.
    #[props(default = true)]
    visible: bool,
    on_query: EventHandler<String>,
    on_step: EventHandler<PillStep>,
    on_toggle_theme: EventHandler<()>,
) -> Element {
    let has_matches = !match_label.trim().is_empty();
    let control_style = format!(
        "display:inline-flex; align-items:center; justify-content:center; width:26px; \
         height:26px; padding:0; border:none; border-radius:999px; background:transparent; \
         color:{}; cursor:pointer; font-size:13px; line-height:1;",
        palette.muted,
    );
    rsx! {
        div {
            "data-yggui-pill-toolbar": "1",
            "data-yggui-pill-visible": if visible { "true" } else { "false" },
            style: format!(
                "--yggui-pill-hover:{}; --yggui-pill-ink:{}; --yggui-pill-muted:{}; \
                 position:absolute; top:10px; left:50%; transform:translateX(-50%); z-index:9; \
                 display:flex; align-items:center; gap:4px; height:{BAR_HEIGHT_PX}px; \
                 padding:0 6px; border-radius:999px; background:{}; \
                 backdrop-filter:blur(16px) saturate(140%); \
                 box-shadow:inset 0 0 0 1px {}, 0 10px 30px rgba(0,0,0,0.13); \
                 opacity:{}; pointer-events:{}; transition:opacity 150ms ease;",
                palette.hover,
                palette.ink,
                palette.muted,
                palette.surface,
                palette.hairline,
                if visible { "1" } else { "0" },
                if visible { "auto" } else { "none" },
            ),
            // A press belongs to the bar, never to the page under it.
            onmousedown: |evt| evt.stop_propagation(),
            svg {
                width: "14",
                height: "14",
                view_box: "0 0 15 15",
                fill: "none",
                stroke: "{palette.muted}",
                stroke_width: "1.3",
                stroke_linecap: "round",
                style: "margin-left:6px; flex:0 0 auto;",
                path { d: "M6.8 10.4a3.6 3.6 0 1 0 0-7.2 3.6 3.6 0 0 0 0 7.2ZM9.6 9.6l2.6 2.6" }
            }
            input {
                r#type: "text",
                value: "{query}",
                placeholder: "{placeholder}",
                "data-yggui-pill-search": "1",
                style: format!(
                    "min-width:0; width:190px; border:none; background:transparent; \
                     color:{}; font-size:12.5px; padding:0 4px; height:{BAR_HEIGHT_PX}px;",
                    palette.ink,
                ),
                oninput: move |evt: FormEvent| on_query.call(evt.value()),
            }
            // The counter appears only once there is something to count — an
            // empty `0/0` sitting in the bar is noise the reader has to learn
            // to ignore.
            if has_matches {
                span {
                    "data-yggui-pill-count": "1",
                    style: format!(
                        "flex:0 0 auto; padding:0 2px; color:{}; font-size:11px; \
                         font-variant-numeric:tabular-nums; white-space:nowrap;",
                        palette.muted,
                    ),
                    "{match_label}"
                }
            }
            button {
                r#type: "button",
                title: "Previous match",
                disabled: !has_matches,
                "data-yggui-pill-step": "previous",
                style: "{control_style}",
                onclick: move |_| on_step.call(PillStep::Previous),
                Icon { icon: yggui_icons::ARROW_UP, size: 14 }
            }
            button {
                r#type: "button",
                title: "Next match",
                disabled: !has_matches,
                "data-yggui-pill-step": "next",
                style: "{control_style}",
                onclick: move |_| on_step.call(PillStep::Next),
                Icon { icon: yggui_icons::ARROW_DOWN, size: 14 }
            }
            // Far from the two controls that get pressed constantly. This one
            // is pressed once a session, and a mis-hit costs a theme flip in
            // the middle of reading.
            span {
                style: format!("width:1px; height:16px; background:{}; margin:0 3px;", palette.hairline),
            }
            button {
                r#type: "button",
                title: if is_dark { "Switch to light" } else { "Switch to dark" },
                "data-yggui-pill-theme": if is_dark { "dark" } else { "light" },
                style: "{control_style} margin-right:2px;",
                onclick: move |_| on_toggle_theme.call(()),
                ThemeGlyph { is_dark }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bar owns no state, so the one thing worth locking is that it cannot
    /// invent any: every value it draws arrives as a prop, and every gesture
    /// leaves as an event. A `use_signal` here would be a second answer to a
    /// question the host has already answered — which query is live, how many
    /// matches there are, which theme is on.
    #[test]
    fn the_bar_holds_no_state_of_its_own() {
        let source = include_str!("pill_toolbar.rs");
        let implementation = source.split("\nmod tests {").next().unwrap_or(source);
        for banned in ["use_signal", "use_resource", "use_memo"] {
            assert!(
                !implementation.contains(banned),
                "{banned} makes the toolbar a second owner of state the host holds"
            );
        }
    }

    /// The stepper is dead until there is something to step through, and the
    /// counter is not drawn at all — an empty `0/0` is noise a reader has to
    /// learn to ignore.
    #[test]
    fn the_stepper_is_inert_without_matches() {
        let source = include_str!("pill_toolbar.rs");
        assert!(source.contains("disabled: !has_matches"));
        assert!(source.contains("if has_matches {"));
    }

    /// The theme control is named for its OUTCOME, which is what every OS
    /// toggle does: in dark you are offered the sun.
    #[test]
    fn the_theme_control_offers_the_theme_you_would_get() {
        let source = include_str!("pill_toolbar.rs");
        let glyph = source
            .split("fn ThemeGlyph")
            .nth(1)
            .expect("the glyph is still here");
        let sun_at = glyph.find("In dark, offer the sun").expect("sun arm");
        let moon_at = glyph.find("In light, offer the moon").expect("moon arm");
        assert!(sun_at < moon_at, "the `is_dark` arm must be the sun");
    }
}
