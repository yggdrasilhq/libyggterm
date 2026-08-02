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

`yggui` does not name `dioxus-desktop`, `wry` or `webkit2gtk` directly — those
were dead dependencies carried over from the shell crate it was carved out of,
and removing them is what lets this workspace resolve standalone. `dioxus` with
the `desktop` feature still pulls them transitively.

yggterm redirects `dioxus-desktop`, `dioxus-interpreter-js` and `wry` to vendored
forks via `[patch.crates-io]`, which Cargo applies at the consuming workspace
root, including over git dependencies. **So the transitive versions differ:
`webkit2gtk` 2.0.1 here, 2.0.2 under yggterm.** That difference is real and is
stated rather than hidden — a change green here can still need checking in
yggterm.
