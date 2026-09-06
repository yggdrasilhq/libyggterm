//! The one icon source for libyggterm surfaces.
//!
//! Contract (see ydesign notebook 13, *iconfont-iconography*):
//!
//! - **One owner.** Icons come from here. A new inline `<svg` in an app layer
//!   is the defect this crate exists to prevent.
//! - **The crispness recipe**: lucide's 24-grid paths, rendered at wrapper
//!   size, `stroke-width="1.5"` (lucide's stock 2.0 blobs at UI sizes — this
//!   is the measured ZCode difference), round caps and joins, `currentColor`,
//!   `fill:none`.
//! - **Sizing contract**: render through [`Icon`] with an explicit pixel size;
//!   the wrapper div is the optical box (16px is the default UI size).
//!
//! Icon paths are lucide (ISC license, <https://lucide.dev>), taken from the
//! v1.17 set. Crate code carries the workspace licence (see root `LICENSE`
//! and `NOTICE`); the path data keeps its ISC provenance.

use dioxus::prelude::*;

macro_rules! lucide {
    ($name:ident, $body:expr) => {
        /// Full inline svg — render through [`Icon`], never as raw text.
        pub const $name: &str = concat!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='100%' height='100%' ",
            "viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.5' ",
            "stroke-linecap='round' stroke-linejoin='round'>",
            $body,
            "</svg>"
        );
    };
}

lucide!(ARROW_UP, "<path d='m5 12 7-7 7 7'/><path d='M12 19V5'/>");
lucide!(ARROW_DOWN, "<path d='M12 5v14'/><path d='m19 12-7 7-7-7'/>");
lucide!(ARROW_LEFT, "<path d='m12 19-7-7 7-7'/><path d='M19 12H5'/>");
lucide!(ARROW_RIGHT, "<path d='M5 12h14'/><path d='m12 5 7 7-7 7'/>");
lucide!(CHEVRON_UP, "<path d='m18 15-6-6-6 6'/>");
lucide!(CHEVRON_DOWN, "<path d='m6 9 6 6 6-6'/>");
lucide!(CHEVRON_LEFT, "<path d='m15 18-6-6 6-6'/>");
lucide!(CHEVRON_RIGHT, "<path d='m9 18 6-6-6-6'/>");
lucide!(X, "<path d='M18 6 6 18'/><path d='m6 6 12 12'/>");
lucide!(PLUS, "<path d='M5 12h14'/><path d='M12 5v14'/>");
lucide!(MINUS, "<path d='M5 12h14'/>");
lucide!(
    ELLIPSIS,
    "<circle cx='12' cy='12' r='1'/><circle cx='19' cy='12' r='1'/><circle cx='5' cy='12' r='1'/>"
);
lucide!(
    ELLIPSIS_VERTICAL,
    "<circle cx='12' cy='5' r='1'/><circle cx='12' cy='12' r='1'/><circle cx='12' cy='19' r='1'/>"
);
lucide!(SEARCH, "<circle cx='11' cy='11' r='8'/><path d='m21 21-4.34-4.34'/>");
lucide!(SQUARE_PEN, "<path d='M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z'/><path d='m15 5 4 4'/>");
lucide!(PENCIL, "<path d='M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z'/><path d='m15 5 4 4'/>");
lucide!(COPY, "<rect width='14' height='14' x='8' y='8' rx='2' ry='2'/><path d='M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2'/>");
lucide!(ROTATE_CW, "<path d='M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8'/><path d='M21 3v5h-5'/>");
lucide!(GIT_BRANCH, "<line x1='6' x2='6' y1='3' y2='15'/><circle cx='18' cy='6' r='3'/><circle cx='6' cy='18' r='3'/><path d='M18 9a9 9 0 0 1-9 9'/>");
lucide!(SETTINGS, "<path d='M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z'/><circle cx='12' cy='12' r='3'/>");
lucide!(SHARE, "<path d='M21 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6'/><path d='m21 3-9 9'/><path d='M15 3h6v6'/>");
lucide!(MESSAGE_CIRCLE, "<path d='M7.9 20A9 9 0 1 0 4 16.1L2 22Z'/>");
lucide!(HASH, "<line x1='4' x2='20' y1='9' y2='9'/><line x1='4' x2='20' y1='15' y2='15'/><line x1='10' x2='8' y1='3' y2='21'/><line x1='16' x2='14' y1='3' y2='21'/>");

/// One icon, one optical box. The wrapper is a flex-centered div of exactly
/// `size` pixels; the svg inside is 100% of it and inherits `currentColor`.
#[component]
pub fn Icon(
    /// A constant from this crate (`ARROW_UP`, `COPY`, …) — never a literal.
    icon: &'static str,
    /// Optical box in px. 16 is the UI default; 12–14 for dense rows.
    #[props(default = 16)]
    size: i32,
) -> Element {
    rsx! {
        div {
            style: "display:inline-flex; align-items:center; justify-content:center; \
                    width:{size}px; height:{size}px; flex:none; color:inherit; line-height:0;",
            dangerous_inner_html: icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consts_are_complete_svgs() {
        for svg in [
            ARROW_UP, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, CHEVRON_UP, CHEVRON_DOWN, CHEVRON_LEFT,
            CHEVRON_RIGHT, X, PLUS, MINUS, ELLIPSIS, ELLIPSIS_VERTICAL, SEARCH, SQUARE_PEN,
            PENCIL, COPY, ROTATE_CW, GIT_BRANCH, SETTINGS, SHARE, MESSAGE_CIRCLE, HASH,
        ] {
            assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
            assert!(svg.contains("stroke-width='1.5'"));
            assert!(svg.contains("viewBox='0 0 24 24'"));
        }
    }
}
