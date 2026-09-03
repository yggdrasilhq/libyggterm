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
//! ## The inline completion (the omnibox flourish, raised)
//!
//! The one piece of the field the component will DRAW without owning: hand it
//! `completion` (the full text: typing plus suggested tail) and
//! `completion_typed_len` (a byte offset), and the field adopts the text with
//! the tail SELECTED — the next keystroke types over it, Enter accepts it,
//! exactly like the browser omnibox. The host computes it per keystroke from
//! [`palette_index_after`]'s siblings on its own side; [`palette_completion_js`]
//! is the guarded write-back both this component and any host-side field reuse.
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

/// The completion-ADOPTION script: the omnibox's inline autocomplete, raised
/// into the component layer by owner ruling ("input bugs should be fixed in
/// the libyggterm components themselves, and the omnibox flourishes are
/// extras"). The host computes WHAT completes the query — it owns the history,
/// the commands, whatever intelligence ranks the field — and hands the
/// component `completion` + `completion_typed_len`; this builder is HOW the
/// field comes to show it.
///
/// The script is [r]AF-deferred and GUARDED, because this write-back is
/// asynchronous and the user is still typing into the field it writes to (the
/// omnibox pill measured the race: a fast typist's next keystroke lands before
/// the frame, and an unguarded frame overwrites the character just typed). At
/// the moment the frame runs, the field must still hold the text the
/// completion was computed FROM (`typed_prefix`) or the completed text
/// already; anything else means the user has moved on, and this completion is
/// silently dropped — a later `oninput` has produced the right one.
///
/// `typed_len`/`completed_len` are BYTE offsets (they come from
/// `str::len()`), the same convention the omnibox pill uses; the selection is
/// therefore only faithful while the address stays ASCII, which is the case
/// the omnibox itself shipped.
///
/// Returns `None` when there is nothing to adopt — a completion that does not
/// extend the typed prefix, or a boundary that is not a char boundary — so the
/// caller never evals a script that would fight the field.
pub fn palette_completion_js(
    completed: &str,
    typed_prefix: &str,
    typed_len: usize,
    completed_len: usize,
) -> Option<String> {
    if typed_len > completed_len
        || !completed.is_char_boundary(typed_len)
        || !completed.is_char_boundary(completed_len)
        || completed.as_bytes().get(..typed_len)? != typed_prefix.as_bytes()
    {
        return None;
    }
    let completed_js = serde_json::to_string(completed).ok()?;
    let typed_prefix_js = serde_json::to_string(typed_prefix).ok()?;
    Some(format!(
        r#"requestAnimationFrame(function(){{
    var el = document.querySelector('[data-yggui-palette-input]');
    if (!el) return;
    // Still what we completed from? A completion for text the user has left
    // behind must not land on top of what they typed since.
    if (el.value !== {typed_prefix_js} && el.value !== {completed_js}) return;
    if (el.value !== {completed_js}) el.value = {completed_js};
    if (el.setSelectionRange) el.setSelectionRange({typed_len}, {completed_len});
}});"#
    ))
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
/* A SELECTED row must not amputate a long URL. Every row ellipsizes at rest —
   a dropdown of full URLs is noise — but the row the user is ON is the row
   they are reading, so it stops clipping and scrolls instead. The owner asked
   for exactly this: selection "should not auto cutoff and should scroll (very
   useful for long urls)". */
[data-yggui-palette-row][data-yggui-palette-row-selected="true"] {
  overflow-x: auto;
  scrollbar-width: thin;
  scrollbar-color: var(--yggui-palette-hairline) transparent;
}
[data-yggui-palette-row][data-yggui-palette-row-selected="true"] [data-yggui-palette-row-text] {
  overflow: visible;
  text-overflow: clip;
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

/// THE TEXT-KILL KEYS, as one embedded helper: `__ygguiTextKill(op)` applies an
/// emacs-style edit to WHATEVER editable element holds focus and returns the
/// new value (null when it did not apply).
///
/// ⛔ WHY A GLOBAL HELPER AND NOT COMPONENT KEYS: the owner's ruling is that
/// text editing belongs to the COMPONENT LAYER, not to each surface — "input
/// bugs should be fixed in the libyggterm components themselves, and the
/// omnibox flourishes are extras". One helper, three ops, every input box:
/// `kill-end` (Ctrl+K), `del-forward` (Ctrl+D), `kill-word-forward` (Alt+D).
/// The host shells install this once and bind the keys once, and every input
/// box in the app grows the same emacs fingers.
///
/// Guards, in the helper because every caller needs them: it refuses anything
/// that is not an `input`/`textarea`, and it refuses xterm's hidden textarea —
/// the terminal owns Ctrl+D (EOF) and Ctrl+K absolutely. After editing it
/// DISPATCHES a bubbling `input` event, so controlled inputs (Dioxus `oninput`
/// handlers) hear the change exactly as if the user had typed it — no host
/// desyncs from a value only the DOM saw.
pub const YGGUI_TEXT_KILL_JS: &str = r#"
window.__ygguiTextKill = function (op) {
  var el = document.activeElement;
  if (!el) return null;
  var tag = (el.tagName || '').toLowerCase();
  if (tag !== 'input' && tag !== 'textarea') return null;
  if (el.closest && el.closest('.xterm')) return null;
  var s = el.selectionStart, e = el.selectionEnd, v = el.value;
  if (s == null || e == null) return null;
  var nv = v, caret = s;
  if (op === 'kill-end') {
    nv = v.slice(0, s); caret = s;
  } else if (op === 'del-forward') {
    if (s !== e) { nv = v.slice(0, s) + v.slice(e); caret = s; }
    else if (e < v.length) { nv = v.slice(0, s) + v.slice(e + 1); caret = s; }
  } else if (op === 'kill-word-forward') {
    var rest = v.slice(e);
    var m = rest.match(/^\s*\S+/);
    var cut = m ? m[0].length : 0;
    nv = v.slice(0, s) + rest.slice(cut); caret = s;
  } else {
    return null;
  }
  if (nv === v) return nv;
  el.value = nv;
  try { el.setSelectionRange(caret, caret); } catch (_) {}
  el.dispatchEvent(new Event('input', { bubbles: true }));
  return nv;
};
"#;

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
    /// The field's GENERATION. Bump it to make the field adopt `query` as its
    /// text; leave it alone and the field keeps whatever the user typed.
    ///
    /// ⛔ THE FIELD IS UNCONTROLLED, AND THE CONTROLLED LOOK IS THE LIE THE
    /// USER TYPES INTO. Owner report (the ychrome command palette, twice): "it
    /// just does not let me type — janky and aggressive." A controlled
    /// `value:` re-sets the DOM text on every render, so every keystroke races
    /// the host's re-render — a fast typist's next key lands before the frame
    /// that re-writes the value, and the frame clobbers it. The same race the
    /// omnibox pill fixed at its own layer, one floor down. So the field owns
    /// its DOM text (`initial_value`, remounted only when `revision` moves)
    /// and reports edits upward; the host's `query` stays the ranking truth
    /// but never fights the caret. Bump `revision` when the HOST changed the
    /// query on its own: accept clears the field, begin-edit resets it.
    #[props(default = 0u64)]
    revision: u64,
    /// The inline completion: the FULL text the field should show (the user's
    /// typing plus the suggested tail), and the byte offset where the typing
    /// ends. `None`/`0` mean no completion is being offered and the field is
    /// left alone.
    ///
    /// The HOST computes it — from its history, its commands, whatever ranks
    /// its rows — because ranking is the host's job (see the module head). The
    /// component only ADOPTS it into the field, with the tail SELECTED so the
    /// next keystroke types over it and Enter accepts it: Chrome's inline
    /// autocomplete, applied through [`palette_completion_js`]'s guarded
    /// write-back. The host recomputes it per keystroke from `on_query`; the
    /// effect below fires whenever the pair moves.
    #[props(default)]
    completion: Option<String>,
    #[props(default = 0usize)]
    completion_typed_len: usize,
    on_query: EventHandler<String>,
    on_move: EventHandler<PaletteMove>,
    /// The chosen row's `id`. Never fires on an empty list.
    on_accept: EventHandler<String>,
    on_dismiss: EventHandler<()>,
) -> Element {
    // The kill helper defines itself once, before anything can press a key.
    // Idempotent by guard, so a re-run costs one `if`.
    use_effect(move || {
        document::eval(YGGUI_TEXT_KILL_JS);
    });
    // THE OMNIBOX FLOURISH, raised: whenever the host moves the completion,
    // adopt it into the field with the suggested tail selected. The script
    // itself is rAF-deferred and stale-guarded (see [`palette_completion_js`]),
    // so a completion computed for a keystroke the field has already left
    // behind lands nowhere.
    use_effect(use_reactive(
        (&completion, &completion_typed_len),
        move |(completion, typed_len): (Option<String>, usize)| {
            let Some(text) = completion else { return };
            let prefix = text.get(..typed_len).unwrap_or_default().to_string();
            if let Some(script) = palette_completion_js(&text, &prefix, typed_len, text.len()) {
                let _ = document::eval(&script);
            }
        },
    ));
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
                //
                // ⛔ UNCONTROLLED: `initial_value` + a `revision`-keyed remount,
                // never a `value:` attribute (the write-back race is the
                // does-not-let-me-type defect — see the prop's doc).
                input {
                    key: "{revision}",
                    "data-yggui-palette-input": "1",
                    r#type: "text",
                    initial_value: "{query}",
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
                        //
                        // ⛔ HOME AND END BELONG TO THE CARET. They used to move
                        // the LIST (First/Last), which read as "aggressive": a
                        // text field where Home and End refuse to move the text
                        // fights the one reflex every editor owns. The list's
                        // first/last moves live on PageUp/PageDown, which no
                        // text field claims.
                        //
                        // ⛔ AND THE EMACS KILLS ARE THE FIELD'S OWN: Ctrl+K
                        // kills to end of line, Ctrl+D deletes forward,
                        // Alt+D kills the word forward — owner request, applied
                        // through the shared text-kill helper so the DOM text,
                        // the caret and the host's query all move together.
                        let mods = evt.modifiers();
                        let key = evt.key();
                        if mods.contains(Modifiers::CONTROL)
                            && let Key::Character(ch) = &key
                        {
                            let op = match ch.to_lowercase().as_str() {
                                "k" => Some("kill-end"),
                                "d" => Some("del-forward"),
                                _ => None,
                            };
                            if let Some(op) = op {
                                evt.prevent_default();
                                evt.stop_propagation();
                                document::eval(&format!("__ygguiTextKill('{op}')"));
                                return;
                            }
                        }
                        if mods.contains(Modifiers::ALT)
                            && let Key::Character(ch) = &key
                            && ch.to_lowercase() == "d"
                        {
                            evt.prevent_default();
                            evt.stop_propagation();
                            document::eval("__ygguiTextKill('kill-word-forward')");
                            return;
                        }
                        match key {
                            Key::ArrowDown => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::Next);
                            }
                            Key::ArrowUp => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::Previous);
                            }
                            Key::PageUp => {
                                evt.prevent_default();
                                on_move.call(PaletteMove::First);
                            }
                            Key::PageDown => {
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
                                        // The full text rides the row's title, so
                                        // even an ellipsized row answers a hover.
                                        title: if item.detail.trim().is_empty() {
                                            item.label.clone()
                                        } else {
                                            format!("{} — {}", item.label, item.detail)
                                        },
                                        style: format!(
                                            "flex:0 1 auto; min-width:0; font-size:13.5px; color:{}; \
                                             white-space:nowrap; overflow:hidden; text-overflow:ellipsis;",
                                            palette.ink,
                                        ),
                                        "data-yggui-palette-row-text": "1",
                                        "{item.label}"
                                    }
                                    if !item.detail.trim().is_empty() {
                                        span {
                                            style: format!(
                                                "flex:1 1 auto; min-width:0; font-size:12px; color:{}; \
                                                 white-space:nowrap; overflow:hidden; text-overflow:ellipsis;",
                                                palette.muted,
                                            ),
                                            "data-yggui-palette-row-text": "1",
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

    /// The completion ADOPTS only what extends the typing. A builder handed a
    /// completion that does not begin with the typed prefix — or a boundary
    /// that is not a char boundary — refuses rather than writing a field the
    /// host's model does not agree with.
    #[test]
    fn a_completion_that_does_not_extend_the_typing_is_refused() {
        let completed = "https://example.com";
        let script = palette_completion_js(completed, "htt", 3, completed.len())
            .expect("an honest completion builds");
        assert!(script.contains("requestAnimationFrame"), "{script}");
        // The tail is what gets selected, so the next keystroke types over it.
        assert!(script.contains("setSelectionRange(3, 19)"), "{script}");
        // The stale guard: the field must still hold what was completed from.
        assert!(script.contains("el.value !== \"htt\""), "{script}");
        // Not an extension of the typing: no script at all.
        assert_eq!(palette_completion_js(completed, "ftp", 3, completed.len()), None);
        // A byte offset inside a multi-byte char cannot select.
        assert_eq!(palette_completion_js("héllo", "h", 2, 6), None);
        // …but the honest prefix of one does build.
        assert!(palette_completion_js("héllo", "h", 1, 6).is_some());
        // A completion shorter than what was typed is a regression, not a tail.
        assert_eq!(palette_completion_js("ht", "htt", 3, 2), None);
    }

    /// The component must stay WIRED to the builder — the flourish dies
    /// silently the day someone deletes the effect and the tests still pass.
    #[test]
    fn the_field_adopts_the_hosts_completion() {
        let src = product();
        assert!(
            src.contains("use_reactive(\n        (&completion, &completion_typed_len),"),
            "the completion effect is gone from the component"
        );
        assert!(src.contains("palette_completion_js(&text, &prefix, typed_len, text.len())"));
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
        for key in ["ArrowDown", "ArrowUp", "PageUp", "PageDown", "Enter", "Escape"] {
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

    /// ⛔ HOME AND END BELONG TO THE CARET (owner report: the palette felt
    /// "aggressive" — a text field whose Home and End move a LIST instead of
    /// the text fights the one reflex every editor owns). The list's jump-to-
    /// ends live on PageUp/PageDown; Home and End must appear in the field's
    /// key handling ONLY as prose, never as consumed arms.
    #[test]
    fn home_and_end_move_the_caret_not_the_list() {
        let src = product();
        let block = src
            .split("onkeydown:")
            .nth(1)
            .expect("the field handles keys");
        for key in ["Key::Home =>", "Key::End =>"] {
            assert!(
                !block.contains(key),
                "{key} is consumed by the list again — the caret must keep it"
            );
        }
        assert!(
            block.contains("Key::PageUp =>") && block.contains("Key::PageDown =>"),
            "the list's first/last moves must live on PageUp/PageDown"
        );
    }

    /// ⛔ THE FIELD IS UNCONTROLLED. A `value:` attribute re-sets the DOM text
    /// on every render, and a fast typist's next keystroke loses that race —
    /// the owner's "it just does not let me type" (ychrome command palette).
    /// The field owns its DOM text (`initial_value`, remounted only when the
    /// host bumps `revision`) and reports edits upward.
    #[test]
    fn the_field_is_uncontrolled_and_the_host_moves_it_by_revision() {
        let src = product();
        let at = src
            .find("key: \"{revision}\"")
            .expect("the field's revision key exists");
        let field = &src[at..(at + 1_200).min(src.len())];
        assert!(
            field.contains("initial_value:"),
            "the field must seed its text once (uncontrolled), not be re-set per render"
        );
        assert!(
            !field.lines().any(|line| line.trim_start().starts_with("value:")),
            "a `value:` attribute is back — the write-back race is the \
             does-not-let-me-type defect"
        );
        assert!(
            src.contains("revision: u64,"),
            "the revision prop is gone — hosts have no way to move the field"
        );
    }

    /// The emacs kills are the field's own keys (owner request): Ctrl+K kills
    /// to end of line, Ctrl+D deletes forward, Alt+D kills the word forward —
    /// each consumed (prevent_default + stop_propagation) so no shell chord or
    /// browser default races the edit, each applied through the shared
    /// text-kill helper so the host hears the change as an ordinary input.
    #[test]
    fn the_emacs_kills_are_field_keys() {
        let src = product();
        for (needle, op) in [
            ("\"k\" => Some(\"kill-end\")", "kill-end"),
            ("\"d\" => Some(\"del-forward\")", "del-forward"),
        ] {
            assert!(src.contains(needle), "the Ctrl kill for {op} is gone");
        }
        assert!(
            src.contains("__ygguiTextKill('kill-word-forward')"),
            "Alt+D's kill-word-forward is gone"
        );
        let at = src.find("if mods.contains(Modifiers::CONTROL)").expect("ctrl arm");
        let arm = &src[at..(at + 900).min(src.len())];
        assert!(
            arm.contains("evt.prevent_default();") && arm.contains("evt.stop_propagation();"),
            "a kill key must be consumed, not observed"
        );
        // The helper itself must refuse the terminal's textarea, or Ctrl+D
        // (EOF) and Ctrl+K die inside every running session.
        assert!(
            src.contains("closest('.xterm')"),
            "the kill helper lost its xterm guard"
        );
    }

    /// A SELECTED row must not amputate a long URL (owner: selection "should
    /// not auto cutoff and should scroll — very useful for long urls"). Rest
    /// rows keep the ellipsis; the selected row scrolls, and every row carries
    /// its full text as a hover title.
    #[test]
    fn a_selected_row_scrolls_instead_of_cutting_off() {
        let src = product();
        assert!(
            src.contains("[data-yggui-palette-row][data-yggui-palette-row-selected=\"true\"] {\n  overflow-x: auto;")
                || (src.contains("[data-yggui-palette-row][data-yggui-palette-row-selected=\"true\"]")
                    && src.contains("overflow-x: auto")),
            "the selected row lost its horizontal scroll"
        );
        assert!(
            src.contains("data-yggui-palette-row-text"),
            "the row text spans lost their scroll marker"
        );
        assert!(
            src.contains("title:"),
            "rows must carry their full text as a hover title"
        );
    }

    /// The text-kill helper is EXPORTED so a host shell can install it once and
    /// give every input box in the app the same emacs fingers — the owner's
    /// layer ruling: editing lives in the component library, not per surface.
    #[test]
    fn the_text_kill_helper_is_exported_to_hosts() {
        assert!(src_yaml_has_text_kill_export());
    }

    fn src_yaml_has_text_kill_export() -> bool {
        let lib = include_str!("lib.rs");
        lib.contains("YGGUI_TEXT_KILL_JS")
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
