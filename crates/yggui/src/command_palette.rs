// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The COMMAND PALETTE: one centred surface that is a field and its results at
//! the same time.
//!
//! Every libyggterm app grows the same control eventually — a place to type
//! where you are going instead of finding it. Built per app it comes out as a
//! small input in a corner with a popover under it, and the popover is the
//! problem: a detached list under a cramped field reads as an autocomplete
//! afterthought, so people stop trusting it to hold anything but URLs.
//!
//! DESIGN.md ▸ Search in chrome asks for the opposite and names the reference:
//! *"the search result surface should wrap the search field itself into one
//! continuous shell, closer to VS Code command/search behavior than to a
//! detached popover under the field"*. That is this component. The field is the
//! top edge of the results surface rather than a separate control above it, so
//! there is one object on screen and the results are plainly the thing the
//! field is for.
//!
//! ⚠ **This component owns SHAPE, MATERIAL, ARRANGEMENT and KEY HANDLING. It
//! owns no state and no ranking.** It does not search, does not filter, does not
//! decide what a match is, and does not hold the query or the selection. Those
//! belong to the host — which already has the history, the commands, the tabs or
//! whatever else it is offering — and a palette keeping its own copy would be a
//! second answer to a question the host has already answered.
//!
//! ## Contract
//!
//! ```ignore
//! CommandPalette {
//!     palette: my_palette,
//!     query: query.read().clone(),
//!     items: ranked.read().clone(),        // ALREADY filtered and ordered
//!     selected: selected.read().clone(),   // index into `items`
//!     on_query: move |next| query.set(next),
//!     on_move: move |dir| selected.set(
//!         palette_index_after(*selected.read(), ranked.read().len(), dir)
//!     ),
//!     on_accept: move |id: String| run(id),
//!     on_dismiss: move |_| open.set(false),
//! }
//! ```
//!
//! and [`YGGUI_COMMAND_PALETTE_CSS`] goes into the host's style block.
//!
//! ⭐ **[`palette_index_after`] is exported on purpose.** The component reports
//! which WAY the user moved and the host applies it, so the host's selection
//! stays the single source of truth — but wraparound and clamping are fiddly
//! enough that every host writing their own is how two apps come to disagree
//! about what Down does on the last row. One rule, both callers.

use dioxus::prelude::*;

/// The palette's brand. Material, radius and motion are this module's.
#[derive(Clone, PartialEq, Debug)]
pub struct CommandPalettePalette {
    /// Ink for typed text and result labels.
    pub ink: String,
    /// Ink for the placeholder, secondary detail and shortcut hints.
    pub muted: String,
    /// The surface's own fill — the field and the list share it, because they
    /// are one object.
    pub surface: String,
    /// The hairline around the surface, and between the field and the results.
    pub hairline: String,
    /// The selected row's fill.
    pub selected: String,
    /// The dimming laid over whatever the palette is covering. A palette with no
    /// scrim reads as a floating card and the eye keeps going to the page.
    pub scrim: String,
}

impl CommandPalettePalette {
    pub fn new(
        ink: impl Into<String>,
        muted: impl Into<String>,
        surface: impl Into<String>,
        hairline: impl Into<String>,
        selected: impl Into<String>,
        scrim: impl Into<String>,
    ) -> Self {
        Self {
            ink: ink.into(),
            muted: muted.into(),
            surface: surface.into(),
            hairline: hairline.into(),
            selected: selected.into(),
            scrim: scrim.into(),
        }
    }
}

/// One offered row.
///
/// `id` is the host's own handle and comes straight back on accept; this
/// component never parses it. `label` is what the row is; `detail` is the
/// quieter half (a URL under a page title, a path under a file name); `hint` is
/// the far-right note — a shortcut, or what kind of thing this is.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct CommandPaletteItem {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub hint: String,
}

impl CommandPaletteItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: String::new(),
            hint: String::new(),
        }
    }
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

/// Which way the keyboard moved the selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaletteMove {
    Previous,
    Next,
    First,
    Last,
}

/// Where the selection lands, for one move.
///
/// ⭐ **It WRAPS**, in both directions. A palette is a short list the user is
/// steering by feel: pressing Down once more at the bottom to reach the top is
/// the behaviour every launcher has, and stopping dead there reads as the key
/// having failed rather than as a boundary.
///
/// An EMPTY list answers 0 rather than refusing. There is nothing to select, so
/// there is no wrong answer, and returning an `Option` would push a `None` case
/// into every host for a state where nothing can be accepted anyway.
pub fn palette_index_after(current: usize, len: usize, moved: PaletteMove) -> usize {
    if len == 0 {
        return 0;
    }
    // A selection that is already out of range — the list shrank under a
    // keystroke, which is what happens on every character typed into a filter —
    // is treated as sitting at the end. Clamping here rather than at each call
    // site is why a host cannot panic-index its own results.
    let current = current.min(len - 1);
    match moved {
        PaletteMove::Previous => {
            if current == 0 {
                len - 1
            } else {
                current - 1
            }
        }
        PaletteMove::Next => {
            if current + 1 >= len {
                0
            } else {
                current + 1
            }
        }
        PaletteMove::First => 0,
        PaletteMove::Last => len - 1,
    }
}

/// Hover, focus and scroll behaviour, which inline styles cannot express.
pub const YGGUI_COMMAND_PALETTE_CSS: &str = r#"
[data-yggui-command-palette] input:focus {
  outline: none;
}
[data-yggui-command-palette] input::placeholder {
  color: var(--yggui-palette-muted);
  opacity: 1;
}
[data-yggui-palette-row]:hover {
  background-color: var(--yggui-palette-selected);
}
[data-yggui-palette-results] {
  scrollbar-width: thin;
  scrollbar-color: var(--yggui-palette-hairline) transparent;
}
[data-yggui-palette-results]::-webkit-scrollbar {
  width: 10px;
}
[data-yggui-palette-results]::-webkit-scrollbar-thumb {
  background-color: var(--yggui-palette-hairline);
  border-radius: 999px;
  border: 3px solid transparent;
  background-clip: content-box;
}
[data-yggui-command-palette-scrim] {
  animation: yggui-palette-in 120ms cubic-bezier(0.05, 0.7, 0.1, 1);
}
@keyframes yggui-palette-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
"#;

/// How far down the viewport the palette's top edge sits.
///
/// Not centred vertically, and that is deliberate: the list grows DOWNWARD as
/// the user types, and a vertically-centred surface would shift its own field
/// up the screen on every keystroke — the one part the eye is fixed on.
const TOP_PERCENT: u32 = 18;
const SURFACE_WIDTH_PX: u32 = 620;
const FIELD_HEIGHT_PX: u32 = 48;
const RESULTS_MAX_HEIGHT_PX: u32 = 380;

/// The palette. Mount it at the top of the app's tree; it fixes itself to the
/// viewport and lays its own scrim.
#[component]
pub fn CommandPalette(
    palette: CommandPalettePalette,
    /// The query. The HOST owns it — a controlled input, so the field cannot
    /// drift out of step with the results beside it.
    #[props(default = String::new())]
    query: String,
    /// The rows to offer, ALREADY filtered and in the order they should appear.
    items: Vec<CommandPaletteItem>,
    /// Which row is selected, as an index into `items`. Out-of-range is
    /// tolerated and reads as the last row — the list shrinks under the user's
    /// own typing, and a palette must not go blank between a keystroke and the
    /// host's next render.
    #[props(default = 0)]
    selected: usize,
    #[props(default = "Type a command or search".to_string())] placeholder: String,
    /// What to say when there is nothing to offer. An empty list drawn as an
    /// empty box reads as a broken palette.
    #[props(default = "No matches".to_string())]
    empty_label: String,
    on_query: EventHandler<String>,
    on_move: EventHandler<PaletteMove>,
    /// The chosen row's `id`. Never fires on an empty list.
    on_accept: EventHandler<String>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let count = items.len();
    let selected = if count == 0 { 0 } else { selected.min(count - 1) };
    let accept_id = items.get(selected).map(|item| item.id.clone());

    rsx! {
        div {
            "data-yggui-command-palette-scrim": "1",
            style: format!(
                "position:fixed; inset:0; z-index:1200; display:flex; justify-content:center; \
                 align-items:flex-start; padding-top:{TOP_PERCENT}vh; background:{};",
                palette.scrim,
            ),
            // A click on the scrim dismisses; a click on the surface must not.
            onclick: move |_| on_dismiss.call(()),
            div {
                "data-yggui-command-palette": "1",
                style: format!(
                    "--yggui-palette-muted:{}; --yggui-palette-selected:{}; \
                     --yggui-palette-hairline:{}; \
                     width:min({SURFACE_WIDTH_PX}px, calc(100vw - 48px)); display:flex; \
                     flex-direction:column; border-radius:12px; background:{}; color:{}; \
                     box-shadow:inset 0 0 0 1px {}, 0 24px 60px rgba(0,0,0,0.22); \
                     overflow:hidden;",
                    palette.muted,
                    palette.selected,
                    palette.hairline,
                    palette.surface,
                    palette.ink,
                    palette.hairline,
                ),
                onclick: move |evt: MouseEvent| evt.stop_propagation(),
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                // THE FIELD — the top edge of the results surface, not a control
                // above it. No border of its own, no margin: the hairline below
                // is the only thing separating them, and only once there is
                // something to separate.
                input {
                    "data-yggui-palette-input": "1",
                    r#type: "text",
                    value: "{query}",
                    placeholder: "{placeholder}",
                    autofocus: true,
                    style: format!(
                        "height:{FIELD_HEIGHT_PX}px; padding:0 16px; border:none; \
                         background:transparent; color:{}; font-size:15px; \
                         font-family:inherit; width:100%; box-sizing:border-box;",
                        palette.ink,
                    ),
                    oninput: move |evt: FormEvent| on_query.call(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| {
                        // ⛔ The arrows and Enter are CONSUMED here. Without the
                        // prevent_default the caret walks the text while the
                        // list moves, so the field loses the user's place on
                        // every step through the results.
                        match evt.key() {
                            Key::ArrowDown => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::Next);
                            }
                            Key::ArrowUp => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::Previous);
                            }
                            Key::Home => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::First);
                            }
                            Key::End => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::Last);
                            }
                            Key::Enter => {
                                evt.prevent_default();
                                if let Some(id) = accept_id.clone() {
                                    on_accept.call(id);
                                }
                            }
                            Key::Escape => {
                                evt.prevent_default();
                                on_dismiss.call(());
                            }
                            _ => {}
                        }
                    },
                }
                // THE RESULTS, under the same roof. The hairline appears only
                // when there is a list, so an empty palette is one clean field
                // rather than a field with a rule under nothing.
                div {
                    "data-yggui-palette-results": "1",
                    style: format!(
                        "max-height:{RESULTS_MAX_HEIGHT_PX}px; overflow-y:auto; \
                         border-top:1px solid {}; padding:6px;",
                        palette.hairline,
                    ),
                    if count == 0 {
                        div {
                            "data-yggui-palette-empty": "1",
                            style: format!(
                                "padding:14px 10px; font-size:13px; color:{};",
                                palette.muted,
                            ),
                            "{empty_label}"
                        }
                    }
                    for (index, item) in items.iter().enumerate() {
                        {
                            let id = item.id.clone();
                            let is_selected = index == selected;
                            rsx! {
                                div {
                                    key: "{item.id}",
                                    "data-yggui-palette-row": "{item.id}",
                                    "data-yggui-palette-row-selected": if is_selected { "true" } else { "false" },
                                    style: format!(
                                        "display:flex; align-items:center; gap:10px; padding:8px 10px; \
                                         border-radius:8px; cursor:pointer; background:{};",
                                        if is_selected { palette.selected.as_str() } else { "transparent" },
                                    ),
                                    onclick: move |_| on_accept.call(id.clone()),
                                    span {
                                        style: format!(
                                            "flex:0 1 auto; min-width:0; font-size:13.5px; color:{}; \
                                             white-space:nowrap; overflow:hidden; text-overflow:ellipsis;",
                                            palette.ink,
                                        ),
                                        "{item.label}"
                                    }
                                    if !item.detail.trim().is_empty() {
                                        span {
                                            style: format!(
                                                "flex:1 1 auto; min-width:0; font-size:12px; color:{}; \
                                                 white-space:nowrap; overflow:hidden; text-overflow:ellipsis;",
                                                palette.muted,
                                            ),
                                            "{item.detail}"
                                        }
                                    } else {
                                        span { style: "flex:1 1 auto;" }
                                    }
                                    if !item.hint.trim().is_empty() {
                                        span {
                                            style: format!(
                                                "flex:0 0 auto; font-size:11px; color:{}; \
                                                 letter-spacing:0.02em;",
                                                palette.muted,
                                            ),
                                            "{item.hint}"
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

    /// Product source only — a string quoted inside a test must not satisfy a
    /// source lock.
    fn product() -> String {
        let src = include_str!("command_palette.rs");
        match src.find("\n#[cfg(test)]") {
            Some(i) => src[..i].to_string(),
            None => src.to_string(),
        }
    }

    #[test]
    fn the_selection_wraps_in_both_directions() {
        assert_eq!(palette_index_after(0, 3, PaletteMove::Next), 1);
        assert_eq!(
            palette_index_after(2, 3, PaletteMove::Next),
            0,
            "Down at the bottom reaches the top — stopping dead reads as the \
             key having failed"
        );
        assert_eq!(palette_index_after(0, 3, PaletteMove::Previous), 2);
        assert_eq!(palette_index_after(1, 3, PaletteMove::Previous), 0);
        assert_eq!(palette_index_after(1, 3, PaletteMove::First), 0);
        assert_eq!(palette_index_after(0, 3, PaletteMove::Last), 2);
    }

    /// The list shrinks under the user's own typing, so a selection that is
    /// out of range is the NORMAL state between a keystroke and the next
    /// render — not a bug to panic on.
    #[test]
    fn a_stale_selection_is_clamped_rather_than_indexed() {
        assert_eq!(palette_index_after(99, 3, PaletteMove::Next), 0);
        assert_eq!(palette_index_after(99, 3, PaletteMove::Previous), 1);
        assert_eq!(palette_index_after(99, 3, PaletteMove::Last), 2);
        // …and an empty list answers rather than refusing: there is nothing to
        // select, so there is no wrong answer, and an Option here would push a
        // None case into every host for a state where nothing can be accepted.
        for moved in [
            PaletteMove::Next,
            PaletteMove::Previous,
            PaletteMove::First,
            PaletteMove::Last,
        ] {
            assert_eq!(palette_index_after(7, 0, moved), 0, "{moved:?}");
        }
    }

    /// ⛔ The arrows and Enter must be CONSUMED. Without it the caret walks the
    /// query while the list moves, and the user loses their place in the field
    /// on every step through the results.
    #[test]
    fn the_navigation_keys_are_consumed_not_merely_observed() {
        let src = product();
        let block = src
            .split("onkeydown:")
            .nth(1)
            .expect("the field handles keys");
        for key in ["ArrowDown", "ArrowUp", "Home", "End", "Enter", "Escape"] {
            let at = block
                .find(&format!("Key::{key} =>"))
                .unwrap_or_else(|| panic!("{key} is not handled at all"));
            let arm = &block[at..(at + 160).min(block.len())];
            assert!(
                arm.contains("evt.prevent_default();"),
                "{key} is observed but not consumed:\n{arm}"
            );
        }
    }

    /// The whole point of the shape: ONE surface. A field drawn as its own
    /// bordered control with the results in a separate box below is the
    /// detached-popover form DESIGN.md rejects.
    #[test]
    fn the_field_and_the_results_share_one_surface() {
        let src = product();
        let at = src.find("\"data-yggui-palette-input\"").expect("the field");
        let field = &src[at..src[at..].find("oninput:").unwrap() + at];
        assert!(
            field.contains("border:none;") && field.contains("background:transparent;"),
            "the field must be the top edge of the results surface, not a \
             control floating above it:\n{field}"
        );
    }

    /// An empty list drawn as an empty box reads as a broken palette.
    #[test]
    fn nothing_to_offer_still_says_something() {
        let src = product();
        assert!(src.contains("\"data-yggui-palette-empty\""));
        assert!(src.contains("{empty_label}"));
    }

    #[test]
    fn an_item_carries_its_own_handle_back_unparsed() {
        let item = CommandPaletteItem::new("tab:17", "Release notes")
            .detail("https://example.invalid/notes")
            .hint("Tab");
        assert_eq!(item.id, "tab:17");
        assert_eq!(item.hint, "Tab");
        // The builder leaves the optional halves empty rather than inventing
        // them, so a host that sets neither draws neither.
        let bare = CommandPaletteItem::new("cmd:quit", "Quit");
        assert!(bare.detail.is_empty() && bare.hint.is_empty());
    }
}
