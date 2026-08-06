// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The SPLIT BUTTON: one primary action that remembers, plus a caret to the
//! rest.
//!
//! The shape exists because a surface that can start N kinds of thing tends to
//! grow N buttons, and the row then costs more attention than any single button
//! saves. yggterm's start page reached seven — "New Codex Session", "New Claude
//! Code Session", "New Terminal", "New Ychrome", "New Ychrome (Incognito)",
//! "New Yedit", "Open Yggdrasil Maker" — and the owner's verdict was that it
//! "looks ugly". Seven equal-weight buttons also tell a lie about frequency:
//! they imply the seventh is as likely as the first, when in practice a user
//! reaches for the same one most of the time.
//!
//! So: collapse a family into ONE control whose face is the member you last
//! used, with a caret for the others. The common case becomes a single click on
//! a button already showing the right word; the rare case costs one extra
//! click; and the row shrinks from N buttons to one per family.
//!
//! ⚠ **This component owns SHAPE, MATERIAL and ARRANGEMENT. It owns no state.**
//! Not which item is selected, not whether the menu is open. Both are handed in
//! and reported back, for the same reason [`crate::pill_toolbar`] holds no
//! query:
//!
//! - **Selection is the STICKY fact**, and stickiness must outlive the widget.
//!   The face has to be right on the first frame after a restart, which means
//!   the host had to have persisted it — so the host is where it lives. A
//!   component-owned copy would be a second answer that resets every mount.
//! - **Open/closed must be reachable from outside.** Hosts have a "close every
//!   transient overlay" action (yggterm's is `close_strip_dropdowns`); a menu
//!   holding its own flag is unreachable by it and stays open under a surface
//!   that has moved on.
//!
//! ## Why the caret is a separate hit target
//!
//! Splitting the button means the primary action never costs a menu. If the
//! whole face opened the menu, the frequent case — "start the thing I always
//! start" — would pay two clicks forever, which is the cost the component was
//! built to remove. The divider is drawn so the two targets read as two.

use dioxus::prelude::*;

/// The control's brand. Material and radius are this module's.
#[derive(Clone, PartialEq, Debug)]
pub struct SplitButtonPalette {
    /// Ink for the face of a neutral (unaccented) button and for menu labels.
    pub ink: String,
    /// Ink for secondary text: the menu's per-item detail line.
    pub muted: String,
    /// Fill for a neutral button and for the menu surface.
    pub surface: String,
    /// The hairline around the menu and between the two hit targets.
    pub hairline: String,
    /// The tint a menu item takes when hovered.
    pub hover: String,
    /// Fill for an ACCENTED button, and the tick beside the selected item.
    pub accent: String,
    /// Ink that reads on `accent`.
    pub on_accent: String,
}

impl SplitButtonPalette {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ink: impl Into<String>,
        muted: impl Into<String>,
        surface: impl Into<String>,
        hairline: impl Into<String>,
        hover: impl Into<String>,
        accent: impl Into<String>,
        on_accent: impl Into<String>,
    ) -> Self {
        Self {
            ink: ink.into(),
            muted: muted.into(),
            surface: surface.into(),
            hairline: hairline.into(),
            hover: hover.into(),
            accent: accent.into(),
            on_accent: on_accent.into(),
        }
    }
}

/// One member of the family.
#[derive(Clone, PartialEq, Debug)]
pub struct SplitButtonItem {
    /// Stable identity. This is what comes back on activation and what the host
    /// persists as the sticky choice — never the label, which is prose and may
    /// be translated or reworded.
    pub id: String,
    /// The face when this item is selected, and the menu row's title.
    pub label: String,
    /// One line under the menu row. Empty hides it.
    pub detail: String,
    /// Overrides [`SplitButtonPalette::accent`] when this item is the face, so a
    /// family can keep per-member brand colour (yggterm's Codex blue and Claude
    /// Code orange are load-bearing recognition cues). Empty means "neutral":
    /// the button renders in `surface`/`ink` rather than accented.
    pub accent: String,
}

impl SplitButtonItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: String::new(),
            accent: String::new(),
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn accent(mut self, accent: impl Into<String>) -> Self {
        self.accent = accent.into();
        self
    }
}

/// Hover and focus behaviour, which inline styles cannot express.
pub const SPLIT_BUTTON_CSS: &str = r#"
[data-yggui-split-button] button {
  transition: background-color 130ms cubic-bezier(0.2, 0, 0, 1),
    color 130ms cubic-bezier(0.2, 0, 0, 1);
}
[data-yggui-split-button] button:disabled {
  opacity: 0.38;
  cursor: default;
}
[data-yggui-split-menu] [data-yggui-split-item]:hover {
  background-color: var(--yggui-split-hover);
}
[data-yggui-split-button] button:focus-visible,
[data-yggui-split-menu] [data-yggui-split-item]:focus-visible {
  outline: 2px solid var(--yggui-split-accent);
  outline-offset: -2px;
}
"#;

/// A chevron, stroked in `currentColor`.
#[component]
fn Caret(open: bool) -> Element {
    rsx! {
        svg {
            width: "10",
            height: "10",
            view_box: "0 0 10 10",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.6",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            style: if open { "transform:rotate(180deg); transition:transform 130ms ease;" } else { "transition:transform 130ms ease;" },
            path { d: "M2 3.75 5 6.75 8 3.75" }
        }
    }
}

/// The split button.
///
/// Mount it inside a `position:relative` parent — the menu is absolutely
/// positioned under the control.
///
/// `selected_id` that matches no item falls back to the FIRST item rather than
/// rendering an empty face, so a persisted choice whose feature was removed
/// degrades to something pressable instead of a dead control.
#[component]
pub fn SplitButton(
    palette: SplitButtonPalette,
    /// The family, in menu order. Empty renders nothing at all — a control with
    /// no actions is not a disabled control, it is absent.
    items: Vec<SplitButtonItem>,
    /// The sticky choice, owned and persisted by the host.
    #[props(default = String::new())]
    selected_id: String,
    /// Whether the menu is showing. Host-owned so a global "close overlays" can
    /// reach it.
    #[props(default = false)]
    open: bool,
    /// Shown before the item label on the face, e.g. "New". Empty for none.
    #[props(default = String::new())]
    prefix: String,
    #[props(default = false)] disabled: bool,
    /// Fired by the primary press AND by picking from the menu. The host both
    /// performs the action and records the id as the new sticky choice — one
    /// handler, so the face cannot disagree with what was last run.
    on_activate: EventHandler<String>,
    /// Requested open state. The host stores it and passes it back as `open`.
    on_open_change: EventHandler<bool>,
) -> Element {
    if items.is_empty() {
        return rsx! {};
    }
    let selected = items
        .iter()
        .find(|item| item.id == selected_id)
        .unwrap_or(&items[0])
        .clone();

    let accented = !selected.accent.trim().is_empty();
    let face_bg = if accented {
        selected.accent.clone()
    } else {
        palette.surface.clone()
    };
    let face_ink = if accented {
        palette.on_accent.clone()
    } else {
        palette.ink.clone()
    };
    // The divider must read on whatever the face is, so it is drawn from the
    // face's own ink rather than the palette hairline, which is tuned for the
    // menu's surface and vanishes on an accented fill.
    let divider = if accented {
        "rgba(255,255,255,0.34)".to_string()
    } else {
        palette.hairline.clone()
    };
    let face_label = if prefix.trim().is_empty() {
        selected.label.clone()
    } else {
        format!("{} {}", prefix.trim(), selected.label)
    };

    let primary_style = format!(
        "display:inline-flex; align-items:center; justify-content:center; min-height:34px; \
         padding:0 12px; border:none; border-radius:8px 0 0 8px; background:{face_bg}; \
         color:{face_ink}; font-size:12px; font-weight:800; cursor:pointer; white-space:nowrap;",
    );
    let caret_style = format!(
        "display:inline-flex; align-items:center; justify-content:center; min-height:34px; \
         width:26px; padding:0; border:none; border-radius:0 8px 8px 0; background:{face_bg}; \
         color:{face_ink}; cursor:pointer; box-shadow:inset 1px 0 0 {divider};",
    );

    let activate_selected = selected.id.clone();
    rsx! {
        div {
            "data-yggui-split-button": "1",
            "data-yggui-split-selected": "{selected.id}",
            "data-yggui-split-open": if open { "true" } else { "false" },
            style: format!(
                "--yggui-split-hover:{}; --yggui-split-accent:{}; \
                 position:relative; display:inline-flex; align-items:stretch;",
                palette.hover, palette.accent,
            ),
            button {
                r#type: "button",
                "data-yggui-split-primary": "1",
                disabled,
                style: "{primary_style}",
                onmousedown: |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                },
                onclick: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    on_open_change.call(false);
                    on_activate.call(activate_selected.clone());
                },
                "{face_label}"
            }
            button {
                r#type: "button",
                "data-yggui-split-caret": "1",
                "aria-haspopup": "menu",
                "aria-expanded": if open { "true" } else { "false" },
                disabled,
                style: "{caret_style}",
                onmousedown: |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                },
                onclick: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    on_open_change.call(!open);
                },
                Caret { open }
            }
            if open {
                div {
                    "data-yggui-split-menu": "1",
                    role: "menu",
                    style: format!(
                        "position:absolute; top:calc(100% + 6px); left:0; z-index:40; \
                         min-width:max(100%, 232px); padding:5px; border-radius:10px; \
                         background:{}; box-shadow:inset 0 0 0 1px {}, 0 14px 34px rgba(0,0,0,0.22); \
                         display:flex; flex-direction:column; gap:1px;",
                        palette.surface, palette.hairline,
                    ),
                    for item in items.iter().cloned() {
                        {
                            let is_selected = item.id == selected.id;
                            let activate_id = item.id.clone();
                            rsx! {
                                div {
                                    key: "{item.id}",
                                    "data-yggui-split-item": "{item.id}",
                                    "data-yggui-split-item-selected": if is_selected { "true" } else { "false" },
                                    role: "menuitem",
                                    tabindex: "0",
                                    style: format!(
                                        "display:flex; align-items:flex-start; gap:8px; padding:7px 9px; \
                                         border-radius:7px; cursor:pointer; user-select:none;",
                                    ),
                                    onmousedown: |evt| {
                                        evt.prevent_default();
                                        evt.stop_propagation();
                                    },
                                    onclick: move |evt| {
                                        evt.prevent_default();
                                        evt.stop_propagation();
                                        on_open_change.call(false);
                                        on_activate.call(activate_id.clone());
                                    },
                                    // A fixed-width gutter, so labels align whether
                                    // or not their row carries the tick.
                                    div {
                                        style: format!(
                                            "flex:0 0 12px; padding-top:1px; font-size:11px; line-height:1.35; \
                                             color:{};",
                                            palette.accent,
                                        ),
                                        if is_selected { "✓" } else { "" }
                                    }
                                    div {
                                        style: "display:flex; flex-direction:column; gap:1px; min-width:0;",
                                        div {
                                            style: format!(
                                                "font-size:12px; font-weight:700; line-height:1.35; color:{};",
                                                palette.ink,
                                            ),
                                            "{item.label}"
                                        }
                                        if !item.detail.trim().is_empty() {
                                            div {
                                                style: format!(
                                                    "font-size:11px; line-height:1.4; color:{};",
                                                    palette.muted,
                                                ),
                                                "{item.detail}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family() -> Vec<SplitButtonItem> {
        vec![
            SplitButtonItem::new("codex", "Codex Session").accent("#2563eb"),
            SplitButtonItem::new("claude-code", "Claude Code Session").accent("#d97706"),
            SplitButtonItem::new("terminal", "Terminal"),
        ]
    }

    /// The face is chosen by id, never by position — that is what makes the
    /// stickiness survive a reordering of the family.
    #[test]
    fn the_selected_id_picks_the_face() {
        let items = family();
        let selected = items
            .iter()
            .find(|item| item.id == "claude-code")
            .expect("present");
        assert_eq!(selected.label, "Claude Code Session");
        assert_eq!(selected.accent, "#d97706");
    }

    /// A persisted choice whose feature was later removed must degrade to a
    /// pressable control, not an empty face.
    #[test]
    fn an_unknown_selection_falls_back_to_the_first_item() {
        let items = family();
        let resolved = items
            .iter()
            .find(|item| item.id == "yedit-that-was-removed")
            .unwrap_or(&items[0]);
        assert_eq!(resolved.id, "codex");
    }

    /// An item with no accent is neutral, not black-on-black: the host reads
    /// this to pick `surface`/`ink` instead of an accented fill.
    #[test]
    fn an_item_without_an_accent_reads_as_neutral() {
        let items = family();
        let terminal = items.iter().find(|item| item.id == "terminal").unwrap();
        assert!(terminal.accent.trim().is_empty());
        assert!(!items[0].accent.trim().is_empty());
    }
}
