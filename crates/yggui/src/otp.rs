// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The six-cell login code entry, and the paste plumbing it needs to be usable.
//!
//! A code widget looks trivial and is not: the code arrives by SMS or email on
//! the same device, so **pasting has to work from wherever focus happens to
//! be**, and on Android neither of the obvious routes does. The long-press
//! paste menu is suppressed on a near-invisible input, and
//! `navigator.clipboard.readText()` is blocked outright in a WebView. That is
//! why this ships as a component with a bridge rather than as six `<input>`s in
//! each app.
//!
//! Contract (matching what the consuming apps already document):
//!
//! ```ignore
//! // once, at startup
//! use_future(move || async move {
//!     let _ = document::eval(&install_otp_paste_bridge_script()).await;
//! });
//!
//! OtpCodeEntry {
//!     cells: login_code.read().clone(),   // Vec<String>, len == YGGUI_OTP_CODE_LEN
//!     on_update: move |next: Vec<String>| login_code.set(next),
//!     on_complete: move |code: String| submit_code(code),
//! }
//! ```
//!
//! and `YGGUI_OTP_CSS` goes into the host's style block.
//!
//! **`on_complete` fires by itself** when the last cell fills — a code entry
//! that still needs a Submit tap after the sixth digit is asking the user to
//! confirm something they can already see is finished.
//!
//! **Focus is the component's job.** A six-cell entry where every keystroke
//! must be preceded by a click is six small broken boxes, not one field: a
//! single-character fill carries focus to the next cell, focus SELECTS (so
//! typing over a filled cell replaces it instead of being swallowed by
//! `maxlength`), Backspace on an empty cell walks the deletion backwards, and
//! the arrows move between cells. Measured on practice.gour.top (2026-09-10):
//! without this the second keystroke died silently and a six-character code
//! needed six click-type rounds.
//!
//! **Cell ink pairs with the cell surface, not with the host's ink.** The
//! background comes from `tokens.composer_surface`, which
//! `CONVERSATION_THEME_CSS` flips under dark mode; a host-fixed ink does not
//! flip with it, and the cells go dark-on-dark
//! (`--yggui-conv-composer-surface: #1d242c` under a light theme's near-black
//! text — measured the same day). The text and caret are therefore styled in
//! [`YGGUI_OTP_CSS`] from `--yggui-conv-work-ink`, which lives on the same
//! sheet as the background and flips with it.

use dioxus::prelude::*;

use crate::conversation::ConversationTokens;

/// How many cells a code has. A constant because the host allocates the vector.
pub const YGGUI_OTP_CODE_LEN: usize = 6;

/// The code, if every cell is filled. `None` while it is not.
///
/// Exported because a host needs the same answer this component uses — for
/// enabling its own Continue button, or for checking a code it already had.
/// Two implementations of "is it complete" is how a widget and its form start
/// disagreeing about whether the user is done.
pub fn complete_otp(cells: &[String]) -> Option<String> {
    if cells.len() != YGGUI_OTP_CODE_LEN {
        return None;
    }
    if cells.iter().any(|cell| cell.trim().is_empty()) {
        return None;
    }
    Some(cells.iter().map(|cell| cell.trim()).collect())
}

/// Which characters a code is made of.
///
/// ⚠ Not cosmetic. A digits-only entry silently refuses every keystroke of an
/// alphanumeric code, which reads to the user as "the box is broken" rather
/// than "wrong alphabet" — reported from the field 2026-08-09, where a
/// six-character code like `KYAXU8` could not be entered or pasted at all.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpAlphabet {
    /// `0-9`. The default, so existing callers are unaffected.
    #[default]
    Digits,
    /// `A-Z` and `0-9`, upper-cased on the way in so a code shown as `KYAXU8`
    /// can be typed or pasted in either case.
    Alphanumeric,
}

/// Keep only digits, and only as many as a code holds.
///
/// Retained as the digits-only shorthand; [`chars_for_otp`] is the general form.
pub fn digits_for_otp(raw: &str) -> Vec<String> {
    chars_for_otp(raw, OtpAlphabet::Digits)
}

/// Pull a code out of whatever the user typed or pasted.
///
/// The paste path is where this earns its keep: a pasted message is usually
/// `"Your code is 481920"` or `"481920 — do not share"`, and getting the code
/// out of it is the difference between paste working and paste appearing
/// broken.
///
/// ⛔ **The two alphabets need genuinely different strategies, and using the
/// digits one for letters is the bug.** Filtering characters works for digits
/// because prose contributes none. For an alphanumeric code the same filter
/// turns `"Your Practice sign-in code is: KYAXU8"` into `"YOURPRACTICESIGN…"`.
/// So the alphanumeric path looks for a *token* of exactly the code length,
/// preferring one that contains a digit — real codes almost always mix, while
/// an English word of the same length rarely does.
pub fn chars_for_otp(raw: &str, alphabet: OtpAlphabet) -> Vec<String> {
    match alphabet {
        OtpAlphabet::Digits => raw
            .chars()
            .filter(char::is_ascii_digit)
            .take(YGGUI_OTP_CODE_LEN)
            .map(|digit| digit.to_string())
            .collect(),
        OtpAlphabet::Alphanumeric => {
            let upper = raw.to_uppercase();
            let mut tokens = upper
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|token| token.chars().count() == YGGUI_OTP_CODE_LEN);
            // Prefer a token that mixes letters and digits: that is what a code
            // looks like, and it is what tells `KYAXU8` from `PLEASE`.
            let mixed = tokens
                .clone()
                .find(|token| token.chars().any(|c| c.is_ascii_digit()));
            if let Some(token) = mixed.or_else(|| tokens.next()) {
                return token.chars().map(|c| c.to_string()).collect();
            }
            // Nothing code-shaped in there — the ordinary case of typing one
            // character into one cell.
            upper
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .take(YGGUI_OTP_CODE_LEN)
                .map(|c| c.to_string())
                .collect()
        }
    }
}

/// Spread pasted (or typed) digits across the cells from a starting index.
fn fill_cells(current: &[String], digits: &[String], from: usize) -> Vec<String> {
    let mut next = current.to_vec();
    next.resize(YGGUI_OTP_CODE_LEN, String::new());
    // A full-length paste always lands at the start, whichever cell had focus:
    // the user pasting a whole code means the whole code, not "the last four
    // digits from cell three".
    let start = if digits.len() == YGGUI_OTP_CODE_LEN {
        0
    } else {
        from.min(YGGUI_OTP_CODE_LEN)
    };
    for (offset, digit) in digits.iter().enumerate() {
        let Some(slot) = next.get_mut(start + offset) else {
            break;
        };
        *slot = digit.clone();
    }
    next
}

/// Focus one cell by index, optionally selecting its contents.
///
/// Four callers, one contract: a single-character fill advances
/// (`index + 1`), Backspace on an empty cell walks backwards (`index - 1`),
/// the arrow keys move, and focus itself selects so the next keystroke
/// replaces a filled cell. An out-of-range index is a harmless "no-cell"
/// probe — advancing from the last cell must not throw focus on the floor.
fn focus_otp_cell_script(index: usize, select: bool) -> String {
    format!(
        "(function(){{ \
            var c = document.querySelector('[data-yggui-otp-cell=\"{index}\"]'); \
            if (!c) {{ return \"no-cell\"; }} \
            c.focus(); \
            if ({select}) {{ c.select(); }} \
            return \"focused\"; \
        }})()",
        index = index,
        select = select,
    )
}

pub const YGGUI_OTP_CSS: &str = r#"
.yggui-otp {
  display: flex;
  gap: 10px;
  justify-content: center;
  align-items: center;
}
.yggui-otp-cell {
  width: 46px;
  height: 56px;
  text-align: center;
  font-size: 22px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  border-radius: 12px;
  outline: none;
  /* ⛔ Ink pairs with the background, which is `composer_surface` and flips
     under dark mode. A host-fixed ink goes dark-on-dark the moment the OS
     does (practice.gour.top, 2026-09-10). `--yggui-conv-work-ink` lives on
     the same sheet as the surface and flips with it; the fallback is the
     light-arm ink-ish default for a host that ships neither. */
  color: var(--yggui-conv-work-ink, #1b232b);
  caret-color: var(--yggui-conv-work-ink, #1b232b);
  transition: border-color 140ms cubic-bezier(0.2, 0, 0, 1),
    box-shadow 140ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-otp-cell:focus {
  border-color: var(--yggui-otp-focus);
  box-shadow: 0 0 0 3px var(--yggui-otp-ring);
}
.yggui-otp-paste {
  display: block;
  margin: 12px auto 0 auto;
  padding: 8px 16px;
  border-radius: 999px;
  cursor: pointer;
  font-size: 12.5px;
  font-weight: 600;
  transition: background-color 140ms cubic-bezier(0.2, 0, 0, 1);
}
.yggui-otp-paste:hover {
  background-color: var(--yggui-otp-hover);
}
"#;

/// The document-level paste listener.
///
/// A paste event is delivered to whatever has focus, and on these screens that
/// is often nothing at all — so this catches it on `document` and routes the
/// digits to the widget, rather than relying on a cell being focused first.
/// Idempotent: installing it twice must not double-handle a paste.
pub fn install_otp_paste_bridge_script() -> String {
    r#"(function () {
  if (window.__ygguiOtpPasteBridge) { return "already-installed"; }
  window.__ygguiOtpPasteBridge = true;
  function deliver(text) {
    if (!text) { return; }
    var target = document.querySelector('[data-yggui-otp-sink="1"]');
    if (!target) { return; }
    target.value = String(text);
    target.dispatchEvent(new Event("input", { bubbles: true }));
  }
  document.addEventListener("paste", function (event) {
    var data = event.clipboardData || window.clipboardData;
    if (!data) { return; }
    var text = data.getData("text");
    if (!text) { return; }
    event.preventDefault();
    deliver(text);
  }, true);
  // The Android side of the same job: the native bridge, when the host has
  // registered it. Absent on desktop and in a browser, where the listener
  // above is enough.
  window.__ygguiOtpPasteFromNative = function () {
    try {
      if (window.AndroidClipboard && window.AndroidClipboard.getText) {
        deliver(window.AndroidClipboard.getText());
        return "native";
      }
    } catch (error) { /* fall through to the async path */ }
    if (navigator.clipboard && navigator.clipboard.readText) {
      navigator.clipboard.readText().then(deliver).catch(function () {});
      return "async";
    }
    return "unavailable";
  };
  return "installed";
})()"#
        .to_string()
}

/// The script the "Paste code" button runs. Separate from the installer so a
/// host can bind it to its own control.
pub fn otp_paste_from_native_script() -> String {
    "window.__ygguiOtpPasteFromNative ? window.__ygguiOtpPasteFromNative() : \"not-installed\""
        .to_string()
}

/// Six cells, a hidden sink the paste bridge writes into, and — on mobile — a
/// button for the case where neither paste route fires on its own.
#[component]
pub fn OtpCodeEntry(
    tokens: ConversationTokens,
    cells: Vec<String>,
    on_update: EventHandler<Vec<String>>,
    on_complete: EventHandler<String>,
    /// Defaults to [`OtpAlphabet::Digits`] so existing callers are unchanged.
    #[props(default)]
    alphabet: OtpAlphabet,
) -> Element {
    let mut cells_state = cells.clone();
    cells_state.resize(YGGUI_OTP_CODE_LEN, String::new());

    // Text and caret are deliberately NOT `tokens.ink` here: the background is
    // `tokens.composer_surface`, which CONVERSATION_THEME_CSS flips under dark
    // mode while a host-fixed ink does not — the pairing lives in the
    // component's own CSS instead (see YGGUI_OTP_CSS).
    let cell_style = format!(
        "background:{}; border:1px solid {}; font-family:{};",
        tokens.composer_surface, tokens.ask_hairline, tokens.prose.ui_font,
    );

    let apply = move |next: Vec<String>| {
        on_update.call(next.clone());
        if let Some(code) = complete_otp(&next) {
            on_complete.call(code);
        }
    };

    rsx! {
        div {
            "data-yggui-otp": "1",
            style: format!(
                "--yggui-otp-focus:{}; --yggui-otp-ring:{}; --yggui-otp-hover:{};",
                tokens.accent,
                if tokens.is_dark { "rgba(124,200,255,0.16)" } else { "rgba(47,124,246,0.12)" },
                tokens.row_hover,
            ),
            style { {YGGUI_OTP_CSS} }
            div {
                class: "yggui-otp",
                for index in 0..YGGUI_OTP_CODE_LEN {
                    input {
                        key: "{index}",
                        class: "yggui-otp-cell",
                        "data-yggui-otp-cell": "{index}",
                        r#type: "text",
                        inputmode: match alphabet {
                            OtpAlphabet::Digits => "numeric",
                            // "text" rather than "numeric": a numeric keypad on
                            // mobile cannot type the letters at all.
                            OtpAlphabet::Alphanumeric => "text",
                        },
                        autocapitalize: match alphabet {
                            OtpAlphabet::Digits => "off",
                            // A code shown as "KYAXU8" arrives upper-case; the
                            // on-screen keyboard should not fight the inbox.
                            OtpAlphabet::Alphanumeric => "characters",
                        },
                        spellcheck: "false",
                        // ⛔ "off" on EVERY cell — never "one-time-code".
                        // That attribute is the exact signal password managers
                        // key on: clicking the first cell hovered the manager's
                        // dropdown over the box (owner-reported 2026-09-11).
                        // The component already owns code delivery — the paste
                        // bridge and auto-advance — so browser OTP fill adds
                        // nothing but the jump-in. The vendor opt-outs below
                        // name the managers that ignore plain "off".
                        autocomplete: "off",
                        "data-1p-ignore": "true",
                        "data-lpignore": "true",
                        "data-bwignore": "true",
                        "data-form-fill-ignore": "true",
                        maxlength: "1",
                        style: "{cell_style}",
                        value: "{cells_state.get(index).cloned().unwrap_or_default()}",
                        onfocus: move |_| {
                            // Focus must SELECT: typing into a filled cell has to
                            // replace it — maxlength otherwise swallows the
                            // keystroke and the field reads as stuck.
                            spawn(async move {
                                let _ = document::eval(&focus_otp_cell_script(index, true)).await;
                            });
                        },
                        oninput: {
                            let current = cells_state.clone();
                            move |evt: FormEvent| {
                                let digits = chars_for_otp(&evt.value(), alphabet);
                                if digits.is_empty() {
                                    // A cleared cell is a real edit, not a no-op:
                                    // the user is backing out of a wrong digit.
                                    let mut next = current.clone();
                                    if let Some(slot) = next.get_mut(index) {
                                        slot.clear();
                                    }
                                    apply(next);
                                    return;
                                }
                                apply(fill_cells(&current, &digits, index));
                                // One keystroke, one cell, caret carried to the
                                // next: a six-character code is six keystrokes,
                                // not six click-type rounds.
                                if digits.len() == 1 && index + 1 < YGGUI_OTP_CODE_LEN {
                                    let script = focus_otp_cell_script(index + 1, true);
                                    spawn(async move {
                                        let _ = document::eval(&script).await;
                                    });
                                }
                            }
                        },
                        onkeydown: {
                            let current = cells_state.clone();
                            move |evt: KeyboardEvent| {
                                let cell_empty = current
                                    .get(index)
                                    .map(|cell| cell.is_empty())
                                    .unwrap_or(true);
                                match evt.key() {
                                    // Backspacing out of an EMPTY cell walks the
                                    // deletion backwards: the previous cell is
                                    // cleared and takes the caret in one press.
                                    Key::Backspace if cell_empty && index > 0 => {
                                        evt.prevent_default();
                                        let mut next = current.clone();
                                        if let Some(slot) = next.get_mut(index - 1) {
                                            slot.clear();
                                        }
                                        apply(next);
                                        let script = focus_otp_cell_script(index - 1, true);
                                        spawn(async move {
                                            let _ = document::eval(&script).await;
                                        });
                                    }
                                    Key::ArrowLeft if index > 0 => {
                                        evt.prevent_default();
                                        let script = focus_otp_cell_script(index - 1, true);
                                        spawn(async move {
                                            let _ = document::eval(&script).await;
                                        });
                                    }
                                    Key::ArrowRight if index + 1 < YGGUI_OTP_CODE_LEN => {
                                        evt.prevent_default();
                                        let script = focus_otp_cell_script(index + 1, true);
                                        spawn(async move {
                                            let _ = document::eval(&script).await;
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        },
                    }
                }
            }
            // The paste sink. Present on every platform because the document
            // listener needs somewhere to deliver to; visually gone, but NOT
            // `display:none` — a hidden input receives no events.
            input {
                "data-yggui-otp-sink": "1",
                r#type: "text",
                "aria-hidden": "true",
                autocomplete: "off",
                "data-1p-ignore": "true",
                "data-lpignore": "true",
                "data-bwignore": "true",
                "data-form-fill-ignore": "true",
                tabindex: "-1",
                style: "position:absolute; width:1px; height:1px; opacity:0; pointer-events:none; \
                        border:none; padding:0; margin:0;",
                oninput: {
                    let current = cells_state.clone();
                    move |evt: FormEvent| {
                        let digits = chars_for_otp(&evt.value(), alphabet);
                        if digits.is_empty() {
                            return;
                        }
                        apply(fill_cells(&current, &digits, 0));
                    }
                },
            }
            // Android WebView suppresses the long-press paste menu on a
            // near-invisible input and blocks `clipboard.readText()`, so the
            // mobile build gets an explicit control that goes through the
            // native bridge.
            if cfg!(feature = "mobile") {
                button {
                    class: "yggui-otp-paste",
                    r#type: "button",
                    style: format!(
                        "border:1px solid {}; background:transparent; color:{}; font-family:{};",
                        tokens.hairline, tokens.ink, tokens.prose.ui_font,
                    ),
                    onclick: move |_| {
                        spawn(async move {
                            let _ = document::eval(&otp_paste_from_native_script()).await;
                        });
                    },
                    "Paste code"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_is_complete_only_when_every_cell_is_filled() {
        let full: Vec<String> = "481920".chars().map(|c| c.to_string()).collect();
        assert_eq!(complete_otp(&full).as_deref(), Some("481920"));

        let mut short = full.clone();
        short[3] = String::new();
        assert!(complete_otp(&short).is_none());

        // Wrong length is never "complete", however full it looks — the host
        // allocates this vector and a resize bug must not read as a code.
        assert!(complete_otp(&full[..5]).is_none());
        assert!(complete_otp(&[]).is_none());
    }

    /// The whole reason paste is handled at all: the code arrives inside a
    /// sentence, not on its own.
    #[test]
    fn pasting_a_whole_sms_keeps_only_the_code() {
        assert_eq!(
            digits_for_otp("Your Jyas code is 481920 — do not share").join(""),
            "481920"
        );
        assert_eq!(digits_for_otp("481920").join(""), "481920");
        assert_eq!(digits_for_otp("no digits here"), Vec::<String>::new());
        // More digits than a code holds must not overflow into nothing.
        assert_eq!(digits_for_otp("1234567890").join(""), "123456");
    }

    /// A full-length paste lands at the START whichever cell had focus.
    /// Otherwise pasting while cell 3 is focused writes the last three digits
    /// and silently drops the first three.
    #[test]
    fn a_full_paste_ignores_which_cell_was_focused() {
        let empty = vec![String::new(); YGGUI_OTP_CODE_LEN];
        let digits = digits_for_otp("481920");
        assert_eq!(fill_cells(&empty, &digits, 3).join(""), "481920");
        assert_eq!(fill_cells(&empty, &digits, 0).join(""), "481920");
    }

    /// A single typed digit lands where the user typed it.
    #[test]
    fn a_typed_digit_lands_in_its_own_cell() {
        let empty = vec![String::new(); YGGUI_OTP_CODE_LEN];
        let filled = fill_cells(&empty, &digits_for_otp("7"), 2);
        assert_eq!(filled[2], "7");
        assert!(filled[0].is_empty());
        assert!(complete_otp(&filled).is_none());
    }

    /// Installing twice must not double-handle a paste — the guard is the whole
    /// difference between one code and the same code entered twice.
    #[test]
    fn the_paste_bridge_installs_once_and_names_its_sink() {
        let script = install_otp_paste_bridge_script();
        assert!(script.contains("__ygguiOtpPasteBridge"));
        assert!(script.contains("already-installed"));
        assert!(script.contains(r#"[data-yggui-otp-sink="1"]"#));
        // The Android bridge is named exactly as the hosts register it.
        assert!(script.contains("window.AndroidClipboard"));
        assert!(otp_paste_from_native_script().contains("__ygguiOtpPasteFromNative"));
    }

    #[test]
    fn alphanumeric_paste_finds_the_code_inside_the_sentence() {
        // The exact shape practice.gour.top sends. Filtering characters -- which
        // is right for digits -- would yield "YOURPRACTICESIGN…" here.
        assert_eq!(
            chars_for_otp(
                "Hi, Your Practice sign-in code is: KYAXU8",
                OtpAlphabet::Alphanumeric
            )
            .join(""),
            "KYAXU8"
        );
    }

    #[test]
    fn alphanumeric_prefers_the_token_that_mixes_letters_and_digits() {
        // "PLEASE" is also six characters. A code almost always carries a digit;
        // an English word of the same length rarely does.
        assert_eq!(
            chars_for_otp("PLEASE use code AB12CD now", OtpAlphabet::Alphanumeric).join(""),
            "AB12CD"
        );
    }

    #[test]
    fn alphanumeric_uppercases_so_either_case_pastes() {
        assert_eq!(
            chars_for_otp("kyaxu8", OtpAlphabet::Alphanumeric).join(""),
            "KYAXU8"
        );
    }

    #[test]
    fn alphanumeric_accepts_a_single_typed_letter() {
        // The ordinary case: one character into one cell. No token matches, so
        // the fallback must still let the keystroke through -- this is exactly
        // what the digits-only filter refused.
        assert_eq!(chars_for_otp("k", OtpAlphabet::Alphanumeric).join(""), "K");
    }

    #[test]
    fn digits_alphabet_is_unchanged_by_the_alphanumeric_work() {
        assert_eq!(
            chars_for_otp("Your code is 481920", OtpAlphabet::Digits).join(""),
            "481920"
        );
        assert_eq!(
            chars_for_otp("no digits here", OtpAlphabet::Digits),
            Vec::<String>::new()
        );
    }

    #[test]
    fn the_focus_helper_names_its_cell_and_selects_on_request() {
        let script = focus_otp_cell_script(2, true);
        assert!(script.contains(r#"[data-yggui-otp-cell="2"]"#));
        assert!(script.contains(".select()"));

        // The arrow-key shape moves without selecting — the flag is a runtime
        // JS guard, so the assertion is the guard value, not the call text.
        assert!(focus_otp_cell_script(2, true).contains("if (true)"));
        assert!(focus_otp_cell_script(3, false).contains("if (false)"));

        // Advancing from the last cell probes one past the end and must stay a
        // harmless "no-cell" probe, never a thrown focus.
        let past_end = focus_otp_cell_script(YGGUI_OTP_CODE_LEN, true);
        assert!(past_end.contains(r#"[data-yggui-otp-cell="6"]"#));
        assert!(past_end.contains("no-cell"));
    }
}
