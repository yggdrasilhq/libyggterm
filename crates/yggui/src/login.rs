// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The shared login surface: a marketing pane floating beside a rounded,
//! borderless auth card.
//!
//! Owner directive 2026-09-28 (the practice desktop login made it loud): the
//! old split-login look — two edge-attached columns meeting at a hairline —
//! reads as a seam, not a composition. The ZCode grammar is the answer: the
//! working surface FLOATS. The auth panel is a rounded card detached from
//! every window edge, casting one soft shadow over a quiet page background;
//! the marketing area is a borderless field beside it. No hairlines anywhere.
//!
//! Hosts bring their own brand and their own form; this module owns the
//! surface grammar. The tokens in [`LOGIN_SURFACE_CSS`] follow the
//! three-arm theme contract (`:root`, OS-dark, explicit dark) the rest of
//! yggui uses, so a host that drives `data-theme` gets dark for free.
//! Web, desktop and mobile render the same component — plain `rsx!`.

use dioxus::prelude::*;

/// Tokens for the floating login card, in the three-arm theme shape.
pub const LOGIN_SURFACE_CSS: &str = r#"
:root {
  --yggui-login-card-radius: 18px;
  --yggui-login-card-gap: 22px;
  --yggui-login-card-surface: #ffffff;
  --yggui-login-card-shadow:
    0 18px 50px rgba(10, 20, 40, 0.13),
    0 2px 8px rgba(10, 20, 40, 0.07);
}
@media (prefers-color-scheme: dark) {
  :root:not(.light):not([data-theme="light"]) {
    --yggui-login-card-radius: 18px;
    --yggui-login-card-gap: 22px;
    --yggui-login-card-surface: #151b26;
    --yggui-login-card-shadow:
      0 20px 56px rgba(0, 0, 0, 0.45),
      0 2px 10px rgba(0, 0, 0, 0.30);
  }
}
:root.dark, :root[data-theme="dark"] {
  --yggui-login-card-radius: 18px;
  --yggui-login-card-gap: 22px;
  --yggui-login-card-surface: #151b26;
  --yggui-login-card-shadow:
    0 20px 56px rgba(0, 0, 0, 0.45),
    0 2px 10px rgba(0, 0, 0, 0.30);
}
"#;

/// The floating auth card: a rounded, borderless surface detached from the
/// window edges. Wrap the app's login FORM in this; let the marketing side
/// of the layout stay a plain borderless field. The card casts its shadow
/// over whatever the host paints behind it, so the host page background must
/// stay visible around the card (give the card's wrapper an inset margin —
/// an edge-attached card is just a seam with rounded corners).
#[component]
pub fn FloatingLoginCard(
    /// CSS padding inside the card. Apps with dense forms pass a smaller
    /// value; the login-panel default suits the split-login layout.
    #[props(default = "34px".to_string())]
    padding: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            // The card owns its inner rhythm: a grid with a generous gap,
            // so a host's heading never touches its first control (owner
            // report 2026-09-28 — "Sign in to Practice" kissed the Google
            // button). Hosts override with .login-panel-head rules, never
            // by un-padding the card.
            style: "display:grid; align-content:start; gap: var(--yggui-login-card-gap); min-width:0; min-height:0; box-sizing:border-box; \
                    border-radius: var(--yggui-login-card-radius); \
                    background: var(--yggui-login-card-surface); \
                    box-shadow: var(--yggui-login-card-shadow); \
                    padding: {padding}; \
                    overflow:hidden;",
            {children}
        }
    }
}
