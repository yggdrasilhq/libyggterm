# libyggterm

The app-hosting library for the Yggdrasil ecosystem — the ncurses analog for
yggterm's GUI surfaces.

A program run inside a yggterm terminal (local or over ssh) can take over
yggterm's surfaces: the viewport becomes the app's page, the right rails carry
its icons and metadata panels, the cwd tree becomes its navigation. `ychrome` is
the pilot app; `yedit` is the second.

| Crate | What it is |
|---|---|
| `yggui` | The reusable surface components: chrome, rails, drag-tree, motion, notifications, theme |
| `yggui-contract` | The wire contract shared by an app and its host |
| `emd-renderer` | emd (extended markdown): the document model and parser every document surface shares. Pure — no UI stack. Spec: [`docs/spec-emd-renderer.md`](docs/spec-emd-renderer.md) |

## Licence

MPL-2.0. This is a deliberate choice: apps that link libyggterm must be free to
ship under their own terms, including proprietary ones. GPL would foreclose that
ecosystem; LGPL fights Rust's static linking. MPL is file-scoped copyleft —
improvements to *these* files come back, and everything an app builds on top
does not have to.

⛔ **Never add the "Incompatible With Secondary Licenses" notice.** Plain MPL-2.0
is GPL-compatible by design, and yggterm (GPL-3.0-or-later) links this library.
That notice would break exactly the combination this licence exists to permit.

Extracted from yggterm on 2026-08-02 and relicensed MPL-2.0 at extraction on
sole copyright. Commits before that date carry yggterm's GPL declaration in
their trees; the licence of this work as distributed is MPL-2.0.

The rule that decides what lives here: **a library apps must LINK is MPL; an
application is GPL.** `emd-renderer` moved in on 2026-08-02 under that rule —
it is the document engine of the whole app pipeline (yedit and ztlkasten's
document surfaces, breezed, jyas-webapp), not a part of the terminal.

## Building

```sh
cargo build --workspace
```

⚠ **Standalone builds here resolve `dioxus` against upstream.** yggterm
consumes this library through a `[patch.crates-io]` redirecting
`dioxus-desktop`, `dioxus-interpreter-js` and `wry` to vendored forks in its own
tree, and Cargo applies patches at the consuming workspace root — including over
git dependencies. `yggui` no longer names those crates directly, but `dioxus`
with the `desktop` feature still pulls them transitively, so the patched
versions differ from what is built here (`webkit2gtk` 2.0.1 standalone vs 2.0.2
under yggterm). A change that is green here can still need checking inside
yggterm. See `THIRD-PARTY-NOTICES.md`.

## Consuming it

yggterm depends on this repo as a git dependency pinned to a tag. For
day-to-day work on both at once, override locally without a publish-pull cycle
by adding to `yggterm/.cargo/config.toml` (git-ignored):

```toml
[patch."https://github.com/yggdrasilhq/libyggterm"]
yggui = { path = "../libyggterm/crates/yggui" }
yggui-contract = { path = "../libyggterm/crates/yggui-contract" }
```

The committed dependency stays the pinned git tag, so a clean clone of yggterm
always builds. The override is a local convenience only — never commit it.
