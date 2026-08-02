// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The conversation surface, both themes, with fixture data.
//!
//! A design cannot be reviewed from source, and rebuilding a host application
//! to look at a hairline is how an afternoon disappears. This renders every
//! component the module exports against a transcript shaped like a real agent
//! session, so a screenshot of it is the artefact a design decision is argued
//! from.
//!
//! ```sh
//! cargo run -p yggui --example conversation_gallery
//! # side by side, one theme per pane:
//! YGGUI_GALLERY_THEME=light cargo run -p yggui --example conversation_gallery
//! ```

use dioxus::prelude::*;
use yggui::conversation::{
    AssistantTurn, ChangedFileChips, ConversationColumn, ConversationEmptyState,
    ConversationTokens, SystemTurn, TurnAction, TurnDivider, UserTurn, WorkGroup, WorkMark,
    WorkRow, WorkingIndicator,
};

fn main() {
    dioxus::launch(Gallery);
}

/// The host palettes the gallery borrows, so what is on screen here is what is
/// on screen in the app rather than a set of colours invented for a demo.
const DARK: (&str, &str, &str, &str) = ("#161c22", "#dde8f3", "#c9d5e0", "#7cc8ff");
const LIGHT: (&str, &str, &str, &str) = ("#ffffff", "#24303a", "#6f7c86", "#2f7cf6");

#[component]
fn Gallery() -> Element {
    let dark_first = std::env::var("YGGUI_GALLERY_THEME").as_deref() != Ok("light");
    rsx! {
        div {
            style: "display:flex; width:100vw; height:100vh; margin:0; overflow:hidden;",
            if dark_first {
                Pane { is_dark: true }
                Pane { is_dark: false }
            } else {
                Pane { is_dark: false }
                Pane { is_dark: true }
            }
        }
    }
}

#[component]
fn Pane(is_dark: bool) -> Element {
    let (page, ink, muted, accent) = if is_dark { DARK } else { LIGHT };
    let tokens = ConversationTokens::from_palette(is_dark, ink, muted, accent);
    let mut plan_folded = use_signal(|| false);
    let mut work_folded = use_signal(|| true);
    let mut group_expanded = use_signal(|| false);

    rsx! {
        div {
            style: "flex:1 1 50%; min-width:0; height:100%; overflow:auto; background:{page}; padding:0 28px;",
            ConversationColumn {
                tokens,
                surface_id: if is_dark { "gallery-dark" } else { "gallery-light" },

                SystemTurn { tokens, "Resumed from ~/gh/yggterm · Claude Code · 41 turns" }

                UserTurn {
                    tokens,
                    timestamp: "20:14",
                    actions: vec![TurnAction {
                        label: "Copy".into(),
                        on_activate: EventHandler::new(|_| {}),
                    }],
                    "The web view still reads like a log. Make the transcript feel like something worth reading — and keep the tool calls out of the way unless I ask for them."
                }

                AssistantTurn {
                    tokens,
                    timestamp: "20:14",
                    meta: "1m 22s",
                    actions: vec![TurnAction {
                        label: "Copy".into(),
                        on_activate: EventHandler::new(|_| {}),
                    }],
                    p {
                        style: "margin:0 0 12px 0;",
                        "Two things were fighting each other. The surface drew every entry at the same weight, so a nine-line answer and a forty-row run of shell commands took the same amount of the page — and the reader had to do the sorting the layout should have done for them."
                    }
                    p {
                        style: "margin:0 0 12px 0;",
                        "So the prose keeps the column and the work collapses into a seam beneath it. What you asked is a card; what the machine answered is the page."
                    }
                }

                WorkGroup {
                    tokens,
                    label: "Work",
                    count: 9,
                    hidden_count: if group_expanded() { 0 } else { 3 },
                    expanded: group_expanded(),
                    on_toggle_group: move |_| group_expanded.toggle(),

                    WorkRow {
                        tokens,
                        mark: WorkMark::Search,
                        label: "Grep",
                        headline: "ConversationWebView|PreviewRunBlock  crates/yggterm-shell/src",
                        folded: work_folded(),
                        on_toggle: move |_| work_folded.toggle(),
                        expanded_body: rsx! {
                            "crates/yggterm-shell/src/shell.rs:86988: fn ConversationWebView(\ncrates/yggterm-shell/src/shell.rs:88534: fn PreviewRunBlock("
                        },
                    }
                    WorkRow {
                        tokens,
                        mark: WorkMark::FileRead,
                        label: "Read",
                        headline: "crates/yggui/src/conversation.rs",
                    }
                    WorkRow {
                        tokens,
                        mark: WorkMark::FileChange,
                        label: "Edit",
                        headline: "one owner for the chat surface, and it lives in yggui",
                        added_lines: 214,
                        removed_lines: 96,
                        folded: plan_folded(),
                        on_toggle: move |_| plan_folded.toggle(),
                        expanded_body: rsx! {
                            ChangedFileChips {
                                tokens,
                                files: vec![
                                    "crates/yggui/src/conversation.rs".into(),
                                    "crates/yggui/src/lib.rs".into(),
                                    "crates/yggterm-shell/src/shell.rs".into(),
                                    "DESIGN.md".into(),
                                    "docs/pending-bugs.md".into(),
                                ],
                            }
                            "Applied 3 hunks."
                        },
                    }
                    WorkRow {
                        tokens,
                        mark: WorkMark::Command,
                        label: "Bash",
                        headline: "cargo test -p yggui conversation",
                        folded: true,
                        on_toggle: move |_| {},
                        expanded_body: rsx! { "" },
                    }
                    WorkRow {
                        tokens,
                        mark: WorkMark::Command,
                        label: "Bash",
                        headline: "cargo check --workspace --all-targets",
                        failed: true,
                    }
                    WorkRow {
                        tokens,
                        mark: WorkMark::Thinking,
                        label: "Thinking",
                        headline: "the two renderers disagree about which one owns the transcript",
                    }
                }

                TurnDivider { tokens, label: "Response · 4 files" }

                AssistantTurn {
                    tokens,
                    timestamp: "20:16",
                    meta: "streaming",
                    streaming: true,
                    p {
                        style: "margin:0;",
                        "The failing check was the second reader — the shell was parsing the JSONL itself while the daemon was already doing it. Removing it is what makes the design possible at all: you cannot style two things that disagree about what they are showing."
                    }
                }

                WorkingIndicator { tokens, label: "Working for 8s" }

                TurnDivider { tokens, label: "Empty state" }

                ConversationEmptyState {
                    tokens,
                    headline: "Nothing has been said in this session yet.",
                    detail: "Open the terminal surface and type — the transcript appears here as the agent writes it.",
                }
            }
        }
    }
}
