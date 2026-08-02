# Changelog

Consumers pin this library by **tag**, so a tag is the release unit. Entries
are written from the git record, not from memory.

## v0.3.1 — 2026-08-02

- **`search_field_shell_style` goes box-only.** It used to emit an inline
  `background`, hairline `box-shadow` and `transition`; an inline fill
  out-specifies a host stylesheet, so the titlebar search field stayed flat and
  inert while every other field in the host learned hover, focus and a themed
  fill. The function now emits layout only, and the host skins the element
  through its field stylesheet (`data-yggui-field`). The `dark_surface`
  parameter is kept for API stability and no longer read.

## v0.3.0 — 2026-08-02

- **`emd-renderer` joins the library, under MPL-2.0.** emd (extended markdown)
  is the markdown-superset document model and parser — one typed block tree
  that every document surface renders, edits, and splices back into the SOURCE
  file, with lossless round-trip as the invariant. It lived in yggterm as a
  GPL-3.0-or-later workspace crate; it is a platform organ of the app pipeline
  (yedit and ztlkasten's document surfaces, breezed, jyas-webapp all consume
  it), and by the licence-by-role rule a library apps must LINK is MPL, not
  GPL. Relicensed MPL-2.0 at the move, on sole copyright.

  It is pure — no Dioxus, no theme, one dependency (`pulldown-cmark`) — so a
  server-side consumer such as a zettelkasten graph indexer can use it without
  a UI stack. Raw HTML is dropped by construction rather than escaped, so
  note-derived content cannot reach a host's JS context.

  The spec moved with it: [`docs/spec-emd-renderer.md`](docs/spec-emd-renderer.md)
  is now the one owner of how this engine is supposed to behave, and yggterm
  points at it rather than keeping a copy.

- **The workspace version now tracks the tag.** It read `0.1.0` while the tree
  was tagged `v0.2.0`, because nothing consumes the number — yggterm pins by
  tag. A version nobody reads is a version that lies; it is `0.3.0` here.

## v0.2.0 — 2026-08-02

- The document split gutter enters the contract, with one place that clamps the
  ratio so neither half can be dragged away entirely.

## v0.1.1 — 2026-08-02

- The side-rail stamps become named constants, so an app and its host name the
  same thing instead of agreeing by coincidence on a string literal.

## v0.1.0 — 2026-08-02

- **First release: `yggui` and `yggui-contract` extracted from yggterm and
  relicensed MPL-2.0** on sole copyright. Plain MPL, deliberately without the
  "Incompatible With Secondary Licenses" notice, so a GPL application linking
  this library — which is exactly what yggterm is — stays permitted.

- **Fixed on the way out: three dependencies `yggui` declared and never used.**
  `dioxus-desktop`, `wry` and `webkit2gtk` resolved inside yggterm only through
  its `[patch.crates-io]` redirect to a vendored `wry` fork. Standing alone they
  were fatal — upstream `dioxus-desktop` 0.7.9 wants `wry ^0.53.5` and therefore
  `webkit2gtk =2.0.1`, against the `=2.0.2` pinned here, with no resolution.
  Nobody outside yggterm could have built the library whose whole purpose is
  being linked by other people. CI now builds this workspace alone, against
  upstream, so the property cannot rot again unseen.
