// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The FROSTED BAR: one floating glass panel that carries a row of controls,
//! and the glass KEYS that sit on it.
//!
//! Extracted 2026-09-04 from two panels that had already earned the look —
//! the terminal D-pad's glass panel and the omnibox's rail bar, which the
//! owner commissioned against the D-pad's recipe and then ruled should be a
//! component: *"it should be made into a generic yggui component base so that
//! other libyggterm apps may use the component as they feel fit."* The
//! recipe was measured twice before it was extracted; do not retune it per
//! app — a frosted panel that differs per surface is two brands, not one.
//!
//! ## What a host owns
//!
//! - **What the keys DO** — [`FrostedKey`] reports a press and nothing else.
//! - **Where the bar sits** — a bar positions nothing: it is a plain flex
//!   row the host places (in a rail's column, over a viewport corner, on its
//!   own line). Append margin/width through [`FrostedBar`]'s `style` prop.
//! - **What else rides the panel** — a bar may carry non-key children (the
//!   omnibox's address pill lives on the same panel); style them against the
//!   `data-yggui-frosted-bar` stamp, the way the shell's pill-on-glass rule
//!   does.
//!
//! ## The recipe (do not retune per app)
//!
//! Panel: a faint dark fill (`rgba(22,27,34,0.10)`) blurred and saturated
//! over whatever is behind (`backdrop-filter: blur(14px) saturate(130%)`),
//! one white inset ring, one soft drop shadow, 12px radius. Keys: a lighter
//! white fill with its own inset ring, 7px radius. Both wear the values the
//! D-pad shipped, because the D-pad's look is what the owner pointed at.

use dioxus::prelude::*;

/// The frosted panel's recipe, inline-style form. `extra` is appended
/// verbatim, so a host adds margin/width without a second background
/// sneaking in and flattening the glass.
pub fn frosted_panel_style(extra: &str) -> String {
    format!(
        "display:flex; flex-wrap:wrap; align-items:center; column-gap:2px; row-gap:1px; \
         padding:5px 8px 6px; border-radius:12px; user-select:none; \
         background:rgba(22,27,34,0.10); backdrop-filter:blur(14px) saturate(130%); \
         box-shadow:inset 0 0 0 1px rgba(255,255,255,0.10), 0 12px 28px rgba(0,0,0,0.14); \
         {extra}"
    )
}

/// A glass key's recipe, inline-style form. `enabled` only dims — a disabled
/// key keeps its body so the bar's shape does not breathe as verbs come and
/// go (the omnibox's back/forward do exactly that).
pub fn frosted_key_style(enabled: bool) -> String {
    format!(
        "border:none; background:rgba(255,255,255,0.16); font-size:15px; line-height:1; \
         padding:5px 9px; border-radius:7px; cursor:{}; opacity:{}; \
         box-shadow:inset 0 0 0 1px rgba(255,255,255,0.20);",
        if enabled { "pointer" } else { "default" },
        if enabled { "0.9" } else { "0.35" },
    )
}

/// Hover and focus behaviour the inline styles cannot express: a key
/// brightens under the pointer, and a keyboard-focus key shows the ring —
/// a glass panel whose keys give no focus answer reads as mouse-only.
pub const YGGUI_FROSTED_BAR_CSS: &str = r#"
[data-yggui-frosted-key]:hover {
  background: rgba(255, 255, 255, 0.26);
}
[data-yggui-frosted-key]:focus-visible {
  outline: none;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.45);
}
"#;

/// The floating glass panel. Mount [`YGGUI_FROSTED_BAR_CSS`] beside it once
/// per surface (idempotent static content), and put [`FrostedKey`]s — or any
/// other control that belongs on the glass — inside.
#[component]
pub fn FrostedBar(
    children: Element,
    /// Appended verbatim after the recipe — margin, width, order. Never a
    /// `background` or `box-shadow`: those are the recipe, and a host that
    /// re-specifies them is shipping a second brand.
    #[props(default = String::new())]
    style: String,
) -> Element {
    rsx! {
        div {
            "data-yggui-frosted-bar": "1",
            style: "{frosted_panel_style(&style)}",
            {children}
        }
    }
}

/// One glass key on a [`FrostedBar`]. Reports presses; owns nothing else.
#[component]
pub fn FrostedKey(
    on_press: EventHandler<()>,
    /// Disabled keys keep their body (see [`frosted_key_style`]) and swallow
    /// the press.
    #[props(default = true)]
    enabled: bool,
    /// The tooltip IS the key's name — a glyph alone does not survive a
    /// hover-less audit.
    title: String,
    children: Element,
) -> Element {
    rsx! {
        button {
            "data-yggui-frosted-key": "1",
            disabled: !enabled,
            title: "{title}",
            style: "{frosted_key_style(enabled)}",
            onclick: move |_| {
                if enabled {
                    on_press.call(());
                }
            },
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Product source only — a string quoted inside a test must not satisfy a
    /// source lock.
    fn product() -> String {
        let src = include_str!("frosted_bar.rs");
        match src.find("\n#[cfg(test)]") {
            Some(i) => src[..i].to_string(),
            None => src.to_string(),
        }
    }

    /// THE RECIPE, locked: both surfaces the owner pointed at (the D-pad and
    /// the omnibox rail bar) wear exactly these values. A drift here is a
    /// second brand.
    #[test]
    fn the_recipe_is_the_dpad_glass_verbatim() {
        let panel = frosted_panel_style("");
        for want in [
            "background:rgba(22,27,34,0.10)",
            "backdrop-filter:blur(14px) saturate(130%)",
            "inset 0 0 0 1px rgba(255,255,255,0.10)",
            "0 12px 28px rgba(0,0,0,0.14)",
            "border-radius:12px",
        ] {
            assert!(panel.contains(want), "panel lost `{want}`: {panel}");
        }
        let key = frosted_key_style(true);
        for want in [
            "background:rgba(255,255,255,0.16)",
            "border-radius:7px",
            "inset 0 0 0 1px rgba(255,255,255,0.20)",
        ] {
            assert!(key.contains(want), "key lost `{want}`: {key}");
        }
    }

    /// A disabled key keeps its body and only dims, so the bar does not
    /// breathe as verbs come and go — and the press is the component's to
    /// swallow, not every caller's.
    #[test]
    fn a_disabled_key_dims_but_keeps_its_body() {
        assert!(frosted_key_style(false).contains("opacity:0.35"));
        assert!(frosted_key_style(true).contains("opacity:0.9"));
        let src = product();
        let at = src
            .find("onclick: move |_|")
            .expect("the key handles its own press");
        assert!(
            src[at..at + 80].contains("if enabled"),
            "the key must swallow presses while disabled"
        );
    }

    /// The host's `extra` rides AFTER the recipe, so a margin cannot
    /// accidentally shadow the glass.
    #[test]
    fn host_style_appends_after_the_recipe() {
        let panel = frosted_panel_style("margin:2px 2px 0; width:100%;");
        assert!(panel.ends_with("margin:2px 2px 0; width:100%;"));
    }
}
