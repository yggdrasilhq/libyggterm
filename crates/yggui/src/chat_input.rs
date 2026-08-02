// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The COMPOSER: one rounded box a person writes into.
//!
//! The other half of [`crate::conversation`]. A transcript surface without a
//! composer is a reader, and every app that shows an agent timeline also has to
//! let someone answer it — so the two live together and read as one thing.
//!
//! **One box, no rule above it.** The composer is not a docked toolbar with a
//! separator; it is a single object floating at the foot of the reading column,
//! and its two controls live INSIDE it — context at the upper left, send at the
//! lower right — so the box stays one shape at any height.
//!
//! Platform-neutral like the rest of the design language: no `dioxus::desktop`,
//! no filesystem. The host owns the value and every effect; this owns the shape
//! and the keyboard contract.

use dioxus::prelude::*;

use crate::conversation::ConversationTokens;

/// One entry in the composer's context menu — another conversation, a file, a
/// chart. The host decides what a key MEANS; the composer only hands it back.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChatContextOption {
    pub key: String,
    pub label: String,
}

/// Which keystroke sends.
///
/// A prop rather than a constant because the two conventions genuinely
/// disagree and both are defensible: a chat app where most messages are one
/// line wants Enter, and a surface where a message is usually a paragraph wants
/// the newline to be the cheap key. Hardcoding either one makes the component
/// unusable for the other.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ComposerSendShortcut {
    /// Enter sends; Shift+Enter inserts a newline.
    #[default]
    Enter,
    /// Shift+Enter sends; Enter inserts a newline.
    ShiftEnter,
}

impl ComposerSendShortcut {
    /// The hint printed beside the send control. Shown, not implied — a
    /// keyboard contract the user has to discover by experiment is a keyboard
    /// contract most users never find.
    pub fn hint(self) -> &'static str {
        match self {
            ComposerSendShortcut::Enter => "Enter",
            ComposerSendShortcut::ShiftEnter => "Shift+Enter",
        }
    }

    fn sends(self, shift_held: bool) -> bool {
        match self {
            ComposerSendShortcut::Enter => !shift_held,
            ComposerSendShortcut::ShiftEnter => shift_held,
        }
    }
}

/// Hover, focus and the context menu — the parts inline styles cannot express.
pub const CHAT_INPUT_CSS: &str = r#"
.yggui-composer {
  transition: border-color 160ms cubic-bezier(0.2, 0, 0, 1),
    box-shadow 160ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-composer:focus-within {
  border-color: var(--yggui-composer-focus);
  box-shadow: 0 0 0 3px var(--yggui-composer-ring);
}
.yggui-composer-textarea {
  resize: none;
  border: none;
  outline: none;
  background: transparent;
  width: 100%;
  font: inherit;
  color: inherit;
}
.yggui-composer-textarea::placeholder {
  color: var(--yggui-composer-placeholder);
  opacity: 1;
}
.yggui-composer-icon-button {
  transition: background-color 140ms cubic-bezier(0.2, 0, 0, 1),
    color 140ms cubic-bezier(0.2, 0, 0, 1), opacity 140ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-composer-icon-button:disabled {
  opacity: 0.4;
  cursor: default;
}
.yggui-composer-icon-button:not(:disabled):hover {
  background-color: var(--yggui-composer-hover);
}
.yggui-composer-menu-row {
  transition: background-color 120ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-composer-menu-row:hover,
.yggui-composer-menu-row:focus-visible {
  background-color: var(--yggui-composer-hover);
}
"#;

fn composer_variables(tokens: &ConversationTokens) -> String {
    format!(
        "--yggui-composer-focus:{}; --yggui-composer-ring:{}; \
         --yggui-composer-placeholder:{}; --yggui-composer-hover:{};",
        tokens.accent,
        if tokens.is_dark {
            "rgba(124,200,255,0.16)"
        } else {
            "rgba(47,124,246,0.12)"
        },
        tokens.meta,
        tokens.row_hover,
    )
}

/// The composer.
///
/// The host owns `value` and every effect; this owns the shape, the two
/// controls and the keyboard contract. It never clears itself — a component
/// that empties the box on send has decided the send succeeded, and it is not
/// the one that knows.
#[component]
pub fn YggChatInputBox(
    tokens: ConversationTokens,
    value: String,
    #[props(default = String::new())] placeholder: String,
    /// The new-conversation state: the box is taller because it is the only
    /// thing on the page and a single-line slot there reads like a search
    /// field, not an invitation.
    #[props(default = false)]
    expanded: bool,
    /// A send is in flight. The control disables rather than disappearing —
    /// a vanishing button relayouts the box under the pointer.
    #[props(default = false)]
    sending: bool,
    #[props(default = false)] context_disabled: bool,
    #[props(default = Vec::new())] context_options: Vec<ChatContextOption>,
    /// A message about the LAST context attempt, shown under the box. Empty
    /// hides it.
    #[props(default = String::new())]
    context_error: String,
    #[props(default = ComposerSendShortcut::Enter)] send_shortcut: ComposerSendShortcut,
    on_input: EventHandler<String>,
    on_submit: EventHandler<()>,
    on_select_context: Option<EventHandler<String>>,
) -> Element {
    let mut menu_open = use_signal(|| false);
    let mut filter = use_signal(String::new);
    let can_send = !sending && !value.trim().is_empty();
    let context_available = !context_disabled && on_select_context.is_some();

    let needle = filter().to_lowercase();
    let visible: Vec<ChatContextOption> = context_options
        .iter()
        .filter(|option| needle.is_empty() || option.label.to_lowercase().contains(&needle))
        .cloned()
        .collect();

    rsx! {
        div {
            "data-yggui-composer": "1",
            "data-yggui-composer-expanded": if expanded { "1" } else { "0" },
            style: format!(
                "{} position:relative; display:flex; flex-direction:column; gap:6px; \
                 width:min({}px, 100%); margin:0 auto; min-width:0; font-family:{};",
                composer_variables(&tokens),
                tokens.column_px,
                tokens.ui_font,
            ),
            style { {CHAT_INPUT_CSS} }
            if menu_open() && context_available {
                ContextMenu {
                    tokens,
                    options: visible,
                    filter: filter(),
                    on_filter: move |text| filter.set(text),
                    on_pick: move |key: String| {
                        menu_open.set(false);
                        filter.set(String::new());
                        if let Some(select) = on_select_context {
                            select.call(key);
                        }
                    },
                    on_dismiss: move |_| menu_open.set(false),
                }
            }
            div {
                class: "yggui-composer",
                style: format!(
                    "position:relative; display:block; box-sizing:border-box; width:100%; \
                     padding:{}; border-radius:20px; background:{}; border:1px solid {}; \
                     box-shadow:{}; color:{}; font-size:14.5px; line-height:1.6;",
                    // Room for the two inset controls, on the sides they sit.
                    if expanded { "44px 18px 46px 18px" } else { "13px 18px 40px 18px" },
                    tokens.composer_surface,
                    tokens.ask_hairline,
                    tokens.ask_shadow,
                    tokens.ink,
                ),
                if context_available {
                    button {
                        class: "yggui-composer-icon-button",
                        r#type: "button",
                        title: "Add context",
                        "aria-label": "Add context",
                        style: format!(
                            "position:absolute; top:8px; left:8px; display:inline-flex; \
                             align-items:center; justify-content:center; width:26px; height:26px; \
                             border-radius:8px; border:1px solid {}; background:transparent; \
                             color:{}; cursor:pointer; padding:0;",
                            tokens.hairline, tokens.meta,
                        ),
                        onclick: move |_| {
                            let next = !menu_open();
                            menu_open.set(next);
                            if !next {
                                filter.set(String::new());
                            }
                        },
                        PlusGlyph {}
                    }
                }
                textarea {
                    class: "yggui-composer-textarea",
                    placeholder: "{placeholder}",
                    value: "{value}",
                    rows: if expanded { "4" } else { "2" },
                    style: format!(
                        "display:block; min-height:{}px; color:{};",
                        if expanded { 96 } else { 40 },
                        tokens.ink,
                    ),
                    oninput: move |evt| on_input.call(evt.value()),
                    onkeydown: move |evt| {
                        if evt.key() != Key::Enter {
                            return;
                        }
                        if send_shortcut.sends(evt.modifiers().shift()) {
                            // The newline must not ALSO be typed — a send that
                            // leaves a stray blank line behind is a send the
                            // user has to clean up after.
                            evt.prevent_default();
                            on_submit.call(());
                        }
                    },
                }
                div {
                    style: "position:absolute; right:10px; bottom:9px; display:flex; \
                            align-items:center; gap:9px;",
                    span {
                        style: format!(
                            "font-size:10px; letter-spacing:0.06em; color:{}; white-space:nowrap;",
                            tokens.meta,
                        ),
                        {send_shortcut.hint()}
                    }
                    button {
                        class: "yggui-composer-icon-button",
                        r#type: "button",
                        title: "Send",
                        "aria-label": "Send",
                        disabled: !can_send,
                        style: format!(
                            "display:inline-flex; align-items:center; justify-content:center; \
                             width:30px; height:30px; border-radius:999px; border:none; padding:0; \
                             background:{}; color:{}; cursor:pointer;",
                            if can_send { tokens.accent } else { tokens.row_hover },
                            if can_send { tokens.send_glyph } else { tokens.meta },
                        ),
                        onclick: move |_| on_submit.call(()),
                        SendGlyph {}
                    }
                }
            }
            if !context_error.trim().is_empty() {
                div {
                    "data-yggui-composer-notice": "1",
                    style: format!(
                        "padding:0 6px; font-size:11px; line-height:1.5; color:{};",
                        tokens.removed,
                    ),
                    "{context_error}"
                }
            }
        }
    }
}

/// The searchable context list. Opens ABOVE the box, because the box sits at
/// the foot of the page and a menu below it would open off-screen.
#[component]
fn ContextMenu(
    tokens: ConversationTokens,
    options: Vec<ChatContextOption>,
    filter: String,
    on_filter: EventHandler<String>,
    on_pick: EventHandler<String>,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            "data-yggui-composer-menu": "1",
            style: format!(
                "display:flex; flex-direction:column; gap:2px; max-height:280px; overflow:auto; \
                 padding:8px; border-radius:14px; background:{}; border:1px solid {}; \
                 box-shadow:{};",
                tokens.composer_surface, tokens.ask_hairline, tokens.ask_shadow,
            ),
            input {
                r#type: "text",
                placeholder: "Search conversations",
                value: "{filter}",
                style: format!(
                    "box-sizing:border-box; width:100%; margin-bottom:4px; padding:7px 10px; \
                     border-radius:9px; border:1px solid {}; background:transparent; color:{}; \
                     font-family:{}; font-size:12.5px; outline:none;",
                    tokens.hairline, tokens.ink, tokens.ui_font,
                ),
                oninput: move |evt| on_filter.call(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Escape {
                        on_dismiss.call(());
                    }
                },
            }
            if options.is_empty() {
                div {
                    style: format!(
                        "padding:10px; font-size:12px; color:{}; text-align:center;",
                        tokens.meta,
                    ),
                    "Nothing to link."
                }
            }
            for option in options.iter().cloned() {
                button {
                    key: "{option.key}",
                    class: "yggui-composer-menu-row",
                    r#type: "button",
                    title: "{option.label}",
                    style: format!(
                        "display:block; width:100%; box-sizing:border-box; padding:8px 10px; \
                         border:none; border-radius:9px; background:transparent; color:{}; \
                         font-family:{}; font-size:13px; text-align:left; cursor:pointer; \
                         overflow:hidden; text-overflow:ellipsis; white-space:nowrap;",
                        tokens.ink, tokens.ui_font,
                    ),
                    onclick: move |_| on_pick.call(option.key.clone()),
                    "{option.label}"
                }
            }
        }
    }
}

/// A `+`, drawn — not the character. A text plus inherits the body face and
/// sits off-centre in a square button at every size.
#[component]
fn PlusGlyph() -> Element {
    rsx! {
        svg {
            width: "14",
            height: "14",
            view_box: "0 0 14 14",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.6",
            stroke_linecap: "round",
            path { d: "M7 2.6v8.8M2.6 7h8.8" }
        }
    }
}

#[component]
fn SendGlyph() -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 15 15",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.7",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M7.5 12.2V3.1M3.6 7l3.9-3.9L11.4 7" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two conventions are exact opposites, and a component that got this
    /// backwards would send on every newline the user typed.
    #[test]
    fn the_send_shortcut_is_the_one_the_host_declared() {
        assert!(ComposerSendShortcut::Enter.sends(false));
        assert!(!ComposerSendShortcut::Enter.sends(true));
        assert!(ComposerSendShortcut::ShiftEnter.sends(true));
        assert!(!ComposerSendShortcut::ShiftEnter.sends(false));
        // The hint must name the key that actually sends, or it teaches the
        // user the wrong contract with total confidence.
        assert_eq!(ComposerSendShortcut::Enter.hint(), "Enter");
        assert_eq!(ComposerSendShortcut::ShiftEnter.hint(), "Shift+Enter");
    }

    #[test]
    fn the_default_shortcut_is_enter() {
        assert_eq!(ComposerSendShortcut::default(), ComposerSendShortcut::Enter);
    }

    /// Same rule as the conversation column: a `:focus-within` ring cannot be
    /// written inline, so it reads a custom property — and a name that drifts
    /// between the two sides fails silently.
    #[test]
    fn the_stylesheet_variables_are_the_ones_the_composer_emits() {
        let tokens = ConversationTokens::from_palette(false, "#111", "#666", "#2f7cf6");
        let variables = composer_variables(&tokens);
        for name in [
            "--yggui-composer-focus",
            "--yggui-composer-ring",
            "--yggui-composer-placeholder",
            "--yggui-composer-hover",
        ] {
            assert!(variables.contains(name), "composer must emit {name}");
            assert!(CHAT_INPUT_CSS.contains(name), "stylesheet must read {name}");
        }
    }
}
