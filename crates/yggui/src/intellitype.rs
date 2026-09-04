// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! THE INTELLITYPE LAW: every text input offers the omnibox's desirable
//! tricks — a prefill of what you are probably typing, SELECTED, so your next
//! keystroke types over it and Enter accepts it — drawn from a history of
//! what that field has accepted before. Default ON. Two named opt-outs:
//!
//! 1. **Secrets** (`IntelliField::secret(true)`, or never calling
//!    [`intelli_record`] for one): an API key, a password, a one-time code —
//!    a field that must neither remember nor suggest. This is not a style
//!    choice; it is the difference between a convenience and a leak.
//! 2. **A field with no `scope`**: history needs an identity to be history
//!    *of this field*. A host that does not name its field simply gets a
//!    plain input — but a field that handles text a user retypes and does
//!    not name itself should carry a comment saying why.
//!
//! The pieces, so a host can take the whole field or just the law:
//!
//! - [`IntelliField`] — the whole thing prebuilt: uncontrolled input, named
//!   scope, records on Enter and on blur, prefills as you type.
//! - [`intelli_on_input`] / [`intelli_on_accept`] — the mechanics for a host
//!   that renders its own `<input>` and wires its own `oninput` (the same
//!   contract the omnibox pill and the command palette already follow: the
//!   host renders, this computes, [`intellitype_js`] writes the field).
//! - [`intellitype_js`] — the guarded write-back, rAF-deferred and dropped
//!   when the field has left the text the completion was computed from (the
//!   fast-typist race the omnibox measured; never write a value derived from
//!   a keystroke the field has already left behind).
//!
//! History is per-process ([`intelli_record`]); a host that wants it to
//! outlive the process feeds its own store through the same two functions.

use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// Per-scope accepted values, most recent first, deduplicated. In-process by
/// design: persistence is the host's call (it knows where its data lives),
/// and a library that writes files on every keystroke is a library that
/// surprises every app.
thread_local! {
    static HISTORIES: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
    static NEXT_FIELD_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// A DOM-unique id for a mounted field, so the write-back can find ITS input
/// and no other. (Dioxus 0.7 ships no use_id; a process-wide counter is all
/// the uniqueness a document needs.)
fn next_field_id() -> u64 {
    NEXT_FIELD_ID.with(|n| {
        let v = n.get() + 1;
        n.set(v);
        v
    })
}

/// How many accepted values one scope remembers. A prefill is "what I typed
/// here lately", not an archive; beyond this the oldest entries fall off.
const SCOPE_HISTORY_CAP: usize = 50;

/// Record an ACCEPTED value for `scope` (the user committed the text — Enter,
/// blur-after-edit, navigation). Whitespace-only values are never history.
pub fn intelli_record(scope: &str, value: &str) {
    let value = value.trim();
    if scope.is_empty() || value.is_empty() {
        return;
    }
    HISTORIES.with(|h| {
        let mut h = h.borrow_mut();
        let entries = h.entry(scope.to_string()).or_default();
        entries.retain(|e| e != value);
        entries.insert(0, value.to_string());
        entries.truncate(SCOPE_HISTORY_CAP);
    });
}

/// Forget one value (a host that learns a secret landed in a scope strips it
/// here; a stale entry a user cannot get away from is a prefill that lies).
pub fn intelli_forget(scope: &str, value: &str) {
    HISTORIES.with(|h| {
        if let Some(entries) = h.borrow_mut().get_mut(scope) {
            entries.retain(|e| e != value);
        }
    });
}

/// The completion for `typed` within `scope`: the most recent accepted value
/// that extends what was typed, as the FULL text (the field shows it with the
/// tail selected). `None` when nothing extends the typing — including when
/// the typed text IS a whole accepted value (offering nothing new is how the
/// prefill stays a guess and never a fight).
pub fn intelli_complete(scope: &str, typed: &str) -> Option<String> {
    let typed = typed.strip_suffix('\n').unwrap_or(typed);
    if typed.is_empty() {
        return None;
    }
    HISTORIES.with(|h| {
        let entries = h.borrow().get(scope).cloned().unwrap_or_default();
        complete_from(&entries, typed)
    })
}

/// The pure decision, testable without the store: most-recent-first entries,
/// case-insensitive prefix, candidate must EXTEND the typing, and the tail
/// beyond the typed byte length must be a char boundary — a completion that
/// cannot select cleanly is not offered.
fn complete_from(entries: &[String], typed: &str) -> Option<String> {
    for candidate in entries {
        if candidate.len() > typed.len()
            && candidate.is_char_boundary(typed.len())
            && candidate[..typed.len()].eq_ignore_ascii_case(typed)
        {
            return Some(candidate.clone());
        }
    }
    None
}

/// THE MECHANICS for a host-rendered input: call from the field's `oninput`.
/// Returns `(completed, typed_len, completed_len)` when the store prefilling
/// is worth showing — the caller writes it through [`intellitype_js`].
///
/// ⛔ INSERTION VS DELETION, the omnibox's rule: complete only a genuine
/// insertion beyond where the user's typing last ended (`prev_typed_len` —
/// the draft's length when no completion was active). The first backspace
/// over a prefill must CLEAR it, not re-offer it; a field that re-completes
/// over a deletion is fighting the user out of their own edit.
pub fn intelli_on_input(
    scope: &str,
    prev_draft_len: usize,
    prev_typed_len: Option<usize>,
    value: &str,
) -> Option<(String, usize, usize)> {
    let prev = prev_typed_len.unwrap_or(prev_draft_len);
    if value.len() <= prev {
        return None;
    }
    let completed = intelli_complete(scope, value)?;
    let completed_len = completed.len();
    Some((completed, value.len(), completed_len))
}

/// The write-back for a host-rendered input found by CSS `selector`.
/// rAF-deferred and stale-guarded: if the field no longer holds the text the
/// completion was computed from, the completion is dropped — a later
/// `oninput` has produced the right one. `None` when there is nothing to
/// adopt (the caller never evals a script that would fight the field).
pub fn intellitype_js(
    selector: &str,
    completed: &str,
    typed_prefix: &str,
    typed_len: usize,
    completed_len: usize,
) -> Option<String> {
    if typed_len > completed_len
        || !completed.is_char_boundary(typed_len)
        || !completed.is_char_boundary(completed_len)
        || completed.as_bytes().get(..typed_len)? != typed_prefix.as_bytes()
        || selector.contains('\'')
    {
        return None;
    }
    let completed_js = serde_json::to_string(completed).ok()?;
    let typed_prefix_js = serde_json::to_string(typed_prefix).ok()?;
    Some(format!(
        r#"requestAnimationFrame(function(){{
    var el = document.querySelector('{selector}');
    if (!el) return;
    // Still what we completed from? A completion for text the user has left
    // behind must not land on top of what they typed since.
    if (el.value !== {typed_prefix_js} && el.value !== {completed_js}) return;
    if (el.value !== {completed_js}) el.value = {completed_js};
    if (el.setSelectionRange) el.setSelectionRange({typed_len}, {completed_len});
}});"#
    ))
}

/// The whole field, prebuilt. Uncontrolled (`initial_value` + a
/// `revision`-keyed remount — never a controlled `value:`, the write-back
/// race that made the first command palette "not let me type"). Records on
/// Enter and on blur; prefills as you type with the tail selected.
///
/// `secret: true` turns ALL of it off — no recording, no prefill — and is
/// the one prop an API-key or password field may never forget.
#[component]
pub fn IntelliField(
    /// The field's history identity — who is asking, and which field. e.g.
    /// `"ychrome/startpage-search"`. An unnamed field is a plain input.
    scope: String,
    #[props(default = String::new())]
    initial_value: String,
    /// The field's GENERATION: bump to make the field adopt `initial_value`.
    #[props(default = 0u64)]
    revision: u64,
    /// ⛔ THE OPT-OUT: a secret field never records and never prefill.
    #[props(default = false)]
    secret: bool,
    #[props(default = "Type".to_string())]
    placeholder: String,
    /// Inline style passthrough (layout belongs to the host; this never
    /// touches the intellitype behaviour).
    #[props(default = String::new())]
    style: String,
    on_input: EventHandler<String>,
    on_accept: EventHandler<String>,
) -> Element {
    let field_id = next_field_id();
    let selector = format!("#intelli-{field_id}");
    let mut typed_len = use_signal(|| 0usize);
    let mut draft_len = use_signal(|| initial_value.len());
    // The field's MODEL value: what Enter accepts and blur records. A prefill
    // makes this the COMPLETED text (the DOM's too) — accepting the typed
    // prefix alone would throw away the very suggestion the user took.
    let mut draft_value = use_signal(|| initial_value.clone());
    // A String prop cannot ride three move closures; each takes its own.
    let scope_input = scope.clone();
    let scope_enter = scope.clone();
    let scope_blur = scope.clone();

    rsx! {
        input {
            key: "{revision}",
            id: "intelli-{field_id}",
            "data-yggui-intellitype": if secret { "secret" } else { "on" },
            "data-yggui-intellitype-scope": "{scope}",
            r#type: if secret { "password" } else { "text" },
            initial_value: "{initial_value}",
            placeholder: "{placeholder}",
            style: "{style}",
            oninput: move |evt: FormEvent| {
                let value = evt.value();
                let prev_draft = draft_len();
                let prev_typed = typed_len();
                let mut next_typed = value.len();
                if !secret {
                    if let Some((completed, typed, completed_len)) =
                        intelli_on_input(&scope_input, prev_draft, Some(prev_typed), &value)
                    {
                        let prefix = completed.get(..typed).unwrap_or_default().to_string();
                        if let Some(script) =
                            intellitype_js(&selector, &completed, &prefix, typed, completed_len)
                        {
                            let _ = document::eval(&script);
                        }
                        next_typed = typed;
                        draft_value.set(completed);
                    } else {
                        draft_value.set(value.clone());
                    }
                } else {
                    draft_value.set(String::new());
                }
                typed_len.set(next_typed);
                draft_len.set(value.len());
                on_input.call(value);
            },
            onkeydown: move |evt: KeyboardEvent| {
                if evt.key() == Key::Enter {
                    let value = draft_value().clone();
                    if !secret {
                        intelli_record(&scope_enter, &value);
                    }
                    typed_len.set(0);
                    on_accept.call(value);
                }
            },
            onblur: move |_| {
                // The model value, not a DOM read: blur has no payload, and
                // the model is what a prefill may have completed.
                if !secret {
                    intelli_record(&scope_blur, &draft_value());
                }
                typed_len.set(0);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product() -> String {
        let src = include_str!("intellitype.rs");
        match src.find("\n#[cfg(test)]") {
            Some(i) => src[..i].to_string(),
            None => src.to_string(),
        }
    }

    #[test]
    fn the_prefill_extends_the_typing_and_nothing_else_offers() {
        let entries = vec![
            "https://en.wikipedia.org/wiki/Favicon".to_string(),
            "emudhra digital signature".to_string(),
        ];
        // The most recent match wins.
        assert_eq!(
            complete_from(&entries, "http"),
            Some("https://en.wikipedia.org/wiki/Favicon".to_string())
        );
        // Case-insensitive on the typed part; the candidate's casing stands.
        assert_eq!(
            complete_from(&entries, "EMUDHRA"),
            Some("emudhra digital signature".to_string())
        );
        // The typed text being a whole value offers nothing — a prefill that
        // re-fights a completed field is the aggression the law forbids.
        assert_eq!(complete_from(&entries, "emudhra digital signature"), None);
        // Nothing extends it.
        assert_eq!(complete_from(&entries, "zzz"), None);
    }

    #[test]
    fn the_first_backspace_clears_and_never_re_offers() {
        // value.len() <= prev_typed_len is a deletion; the mechanics refuse.
        assert_eq!(intelli_on_input("s", 10, Some(4), "http"), None);
        // A genuine insertion beyond the typing may prefill.
        let entries = vec!["https://example.test/page".to_string()];
        HISTORIES.with(|h| {
            h.borrow_mut().insert("s".to_string(), entries);
        });
        assert_eq!(
            intelli_on_input("s", 4, Some(4), "https://"),
            Some(("https://example.test/page".to_string(), 8, 25))
        );
    }

    #[test]
    fn the_write_back_is_guarded_against_the_fast_typist() {
        let script = intellitype_js("#field", "https://example.test", "https", 5, 20)
            .expect("an honest completion builds");
        assert!(script.contains("requestAnimationFrame"));
        assert!(script.contains("el.value !== \"https\""));
        assert!(script.contains("setSelectionRange(5, 20)"));
        // Not an extension of the typing, mid-char boundary, or a selector
        // that would break the quoting: none of these may reach the field.
        assert_eq!(intellitype_js("#f", "ab", "abc", 3, 2), None);
        assert_eq!(intellitype_js("#f", "héllo", "h", 2, 6), None);
        assert_eq!(intellitype_js("#f'", "ab", "a", 1, 2), None);
    }

    #[test]
    fn recording_is_deduped_recent_first_and_capped() {
        intelli_record("t::dedupe", "alpha");
        intelli_record("t::dedupe", "beta");
        intelli_record("t::dedupe", "alpha");
        intelli_record("t::dedupe", "  "); // whitespace is never history
        let entries = HISTORIES.with(|h| h.borrow().get("t::dedupe").cloned().unwrap());
        assert_eq!(entries, vec!["alpha".to_string(), "beta".to_string()]);
        intelli_forget("t::dedupe", "alpha");
        let entries = HISTORIES.with(|h| h.borrow().get("t::dedupe").cloned().unwrap());
        assert_eq!(entries, vec!["beta".to_string()]);
    }

    /// The LAW is executable: the field stamps its opt-out state, and the
    /// secret path is INSIDE the component so a host cannot forget it.
    #[test]
    fn the_field_stamps_secrets_and_the_law_is_in_the_source() {
        let src = product();
        assert!(src.contains("data-yggui-intellitype\": if secret { \"secret\" }"));
        assert!(src.contains("type: if secret { \"password\" }"));
        assert!(src.contains("if !secret"), "the secret path guards recording AND prefill");
        assert!(src.contains("never records and never prefill"));
    }
}
