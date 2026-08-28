<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# DESIGN.md — libyggterm

libyggterm is the platform layer of the fleet's design system: `yggui` (the
component vocabulary), `yggui-contract` (the wire and orientation
contracts), and `emd-renderer` (what a document IS — `yggui::prose` says how
it reads).

## The layer ladder

```
L0  Dioxus primitives
L1  yggui + yggui-contract + emd-renderer   ← this repo
L2  app design languages (yedit, ychrome, ytop, kasten, …)
L3  project overlays (a product's own design notebook over L1)
```

Consult your own layer first; fall through to the adjacent lower layer when
a question is undefined there; when the lower layer lacks a component, **grow
the lower layer** — a new `yggui` component with a gallery entry — and never
hand-roll a second encoding inside one app. Admission keeps the platform's
own gate: a forcing consumer, and a second consumer before a widget becomes
a schema vocabulary kind.

## Where the language is exhibited

The prose constitutions live in yggterm's `DESIGN.md` (brand intent, control
language, status vocabulary, motion) and in the app repos' overlays. The
**exhibition** — the components and canonical patterns rendered live, as
screenshot-able notebooks — is the app `ydesign` (`yggdrasilhq/ydesign`):
run `ydesign` inside yggterm, or read a notebook from a CLI with
`ydesign --notebook <id>`. Component work in this repo is argued from its
pages and from `crates/yggui/examples/conversation_gallery.rs` +
`scripts/gallery-shot.sh` — never from source alone.

Where a ydesign notebook and this repo's code disagree, fix the drifted
layer and record the correction in both places in the same commit.

## The standing rules this repo owns

- **One visual decision, one owner.** Token structs and style functions are
  the only sources of faces, sizes, spacing and state styling. A host spells
  none of them.
- **Fixed property-key sets.** Every style function emits the same property
  keys in every state — values differ, keys never (Dioxus never clears a
  dropped key).
- **Typed superset grammar.** emd grows as typed `MdBlock`/`MdInline`
  variants with source ranges; the round-trip stays byte-faithful; raw HTML
  is dropped by construction.
- **Contracts name WHAT, never WHERE.** `ChromeSlot` vs `SidebarEdge`; an
  identifier that fuses them is a bug waiting for the first mirror flip.
- **MPL-2.0, plain.** Never add the Exhibit-B secondary-licences notice —
  the GPL-app-links-MPL-lib combination is load-bearing for the whole app
  ecosystem.

Per-file Exhibit A headers state the licence; `NOTICE` and the IP register
row carry the licensing facts.
