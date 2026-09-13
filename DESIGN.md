<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# DESIGN.md — libyggterm

libyggterm is the platform layer (L1) of the fleet's design system. This
page is the pointer; the decisions live in the doors below. The layout this
file follows is the design/ 1.0.0 contract
(`yggdrasilhq/ydesign`, `docs/design-layout.md`): every repository that
ships UI carries `DESIGN.md` + `design/`.

## Doors

- [design/00-platform](design/00-platform.md) — the standing rules this repo owns: one visual decision one owner, fixed property-key sets, typed superset grammar, contracts name WHAT never WHERE, MPL-2.0 plain.
- [design/Inheritance](design/Inheritance.md) — the chain: Dioxus (L0) → this repo (L1) → app languages (L2) → project overlays (L3).
- [docs/spec-emd-renderer](docs/spec-emd-renderer.md) — what an emd document is; §8 names the extensibility contract (base extensions ship here; private extensions live in product pipeline repos).
- [docs/spec-emd-components](docs/spec-emd-components.md) — the typed analytical component grammar (version 1 vocabulary, bounds, evidence contracts).
- [docs/spec-app-architecture](docs/spec-app-architecture.md) — the app architecture the components serve.
- [crates/yggui-icons](crates/yggui-icons/) — the base icon blocks; the rendered catalogue page lives in the ydesign Icons notebook.
- [design/assets](design/assets/) — icons/, fonts/, components/, img/ (by kind, never a flat dump).

## The exhibition

The components and canonical patterns are rendered live as screenshot-able
notebooks by the **ydesign** app (`yggdrasilhq/ydesign`,
`design/notebooks/*.emd`): run `ydesign` inside yggterm, or read a notebook
from a CLI with `ydesign --notebook <id>`. Component work in this repo is
argued from its pages and from
`crates/yggui/examples/conversation_gallery.rs` + `scripts/gallery-shot.sh`
— never from source alone.

Where a ydesign notebook and this repo's code disagree, fix the drifted
layer and record the correction in both places in the same commit.

## The ladder

```
L0  Dioxus primitives
L1  yggui + yggui-contract + emd-renderer + yggui-icons   ← this repo
L2  app design languages (yedit, ychrome, ytop, kasten, …)
L3  project overlays (a product's own design notebook over L1)
```

Consult your own layer first; fall through to the adjacent lower layer when
a question is undefined there; when the lower layer lacks a component, **grow
the lower layer** — a new `yggui` component with a gallery entry — and never
hand-roll a second encoding inside one app. Admission keeps the platform's
own gate: a forcing consumer, and a second consumer before a widget becomes
a schema vocabulary kind.
