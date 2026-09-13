<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Start here, the base design language

**edition 2026-09-07 · rev 1**

ydesign is the *base* design language of the yggdrasil app fleet, shipped as
live notebooks. "Base" because every app builds over it. The layering is the
whole point:

| Layer | What it is | Where it lives |
|---|---|---|
| **L0 · Dioxus** | UI primitives | upstream |
| **L1 · yggui** | The component vocabulary, the type system, the contracts | libyggterm (`crates/yggui`, `yggui-contract`, `emd-renderer`) |
| **L2 · App languages** | yedit, ychrome, ytop, kasten, each app's own patterns over L1 | each app's repo |
| **L3 · Project overlays** | A product's own design notebook, layered over L1 | that product's repo |

## The consultation ladder

Work at your own layer first. When your layer does not answer a question:

1. **Fall through one layer**, read the same concern in the layer below.
   An app that cannot decide a row's spacing consults the row engine (L1),
   never invents a number (L2).
2. **If the lower layer lacks the component, grow the lower layer.** A new
   component belongs in `yggui` (with a gallery entry in the *Component
   gallery* notebook), not hand-rolled inside the app that wanted it. The
   admission gate is the platform's own: a forcing consumer, and a second
   consumer before the widget becomes a vocabulary kind.
3. **Never ship a second encoding of a decided thing.** If a rule exists,
   one segmented control, one field skin, one row engine, one status
   vocabulary, a second implementation is the bug, however local it feels.

## The working habit: argue from pixels

A design judgement is argued from a **pixel screenshot**, never from source
and never from a description. The instruments:

- Component work: render the gallery example and shoot it,
  `cargo run -p yggui --example conversation_gallery`, then
  `libyggterm/scripts/gallery-shot.sh <example> out.png` (headless sway +
  grim; build the example *first*, outside the capture window).
- Live surfaces: `server app screenshot <out.png>` against the running
  shell. For document surfaces and splits use the faithful backend
  (`--backend os`); the composite paste hides DOM layers.
- Judge, fix at the **component layer**, and re-shoot. "Ugly" that is fixed
  in the call site comes back in the next call site.

## Where authority lives

- **DESIGN.md**, routing and working procedure. Palette, typography and
  brand rationale belong in the design notebooks it points to.
- **These notebooks and the app's design/ notebooks**, the visual authority:
  components, specimens, semantic tokens and their rationale.
- **Inheritance.md**, parent layers and explicit override scope, from Dioxus
  through yggui into an app. Undefined decisions inherit transitively.
- **libyggterm specs**, `docs/spec-app-architecture.md` (the tier decision:
  who paints the pixels) and `docs/spec-emd-renderer.md` (what a document is).
- **`.agents/skills/libyggterm-surfaces/SKILL.md`**, the app-platform
  contract: surfaces, transport, lifecycle, the widget vocabulary.

When DESIGN.md duplicates a visual decision, move that decision and its
rationale into the appropriate notebook and leave a pointer.

## Evidence labels and editions

Every book opens with an `edition <date> · rev <n>` line; a critique names the
edition it reviewed, and an accepted critique bumps the rev and links its
verification. Every claim carries one of four labels, used identically in
every book and project layer:

| Label | Claim |
|---|---|
| **observed** | Measured from a real surface, a screenshot, a probe, a grep, with the instrument named. |
| **reconstructed** | Redrawn to teach, with invented content; useful ideas, not proof. |
| **proposed** | Offered for comment; becomes a decision only when its component change lands with its own proof. |
| **verified** | Re-checked against the artifact on a stated date (who ran what, what it showed). |

A chapter that cannot name its label is asserting by tone, refuse it in
review. The per-chapter conversion status (front matter, legacy illustration,
working specimen, reviewed reference) lives in the
[chapter inventory](chapter-inventory.md); no chapter may claim a status the
inventory does not grant it.

## The collection, in reading order

Read in this order; each book assumes the ones before it:

1. **Foundations**, semantic colour, typography, relationships: the visual
   language under everything.
2. **Component gallery**, every component an app may reach for, and the
   one-owner rule each encodes.
3. **The catalogue**, the design exhibited: real pixels with the choices
   behind them.
4. **Sidebars**, the row engine, status vocabulary, partitioning, header
   anatomy.
5. **Forms & settings**, section cards, the one field skin, the short-phrase
   rule.
6. **Motion & feedback**, toasts and anchors, the stage curtain, the shared
   blink clock.
7. **emd & notebooks**, the extended-markdown contracts and authoring rules.
8. **Ribbons**, commands that belong to the workspace.
9. **Complex sidebars**, task-first identity and progressive disclosure.
10. **List views**, one reading grid, distinct purposes.
11. **Icons**, the one icon source and the crispness recipe.
12. **Inheritance**, what the base decides, what an app may override.
13. **ZCode reversed**, a measured external reference and its Dioxus port
    path.
14. **Worked examples** (Examples mode), canonical surfaces rebuilt as live
    schemas to screenshot against.
15. **Roadmap**, the defect and demand ledger; where an agent files before
    fixing.

## How other apps consume this

Run `ydesign init <worktree> --id <project>` to create missing scaffolding
and register its notebooks. A local projects.json makes all registered
layers selectable in one shelf; each notebook remains independently readable.
App identity belongs in its notebook, not in the shared base palette.

An app's campaign memory points at these notebooks (the shelf ships inside
the app: run `ydesign` inside yggterm). A project that defines its own layer
(L3) writes its own design notebook in its own repo, states what it overrides,
and defers to this base set for everything it does not. The fallback is
always the adjacent lower layer, never a local invention.
