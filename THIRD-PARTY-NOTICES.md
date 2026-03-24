# Third-party notices

libyggterm is licensed MPL-2.0. It builds against the crates below, all under
permissive licences compatible with MPL-2.0 and with GPL-3.0-or-later consumers.

| Crate | Licence |
|---|---|
| anyhow, base64, include_dir, once_cell, libc, serde, serde_json, tokio, time, tracing | MIT OR Apache-2.0 |
| dioxus, dioxus-desktop | MIT OR Apache-2.0 |
| wry, tao | Apache-2.0 OR MIT |
| webkit2gtk (Rust bindings) | MIT |
| png | MIT OR Apache-2.0 |

`webkit2gtk` bindings link the system WebKitGTK library (LGPL-2.1+/BSD), which
is dynamically linked and not distributed with this source.

## A note on dioxus and wry versions

yggterm consumes this library through a `[patch.crates-io]` that redirects
`dioxus-desktop`, `dioxus-interpreter-js` and `wry` to vendored forks in its own
tree. Cargo applies patches at the consuming workspace root, including over git
dependencies, so yggterm builds these crates against its forks.

**Standalone builds and CI here exercise upstream dioxus and wry, not yggterm's
forks.** That difference is real and is stated rather than hidden: a change that
passes here can still need checking inside yggterm.
