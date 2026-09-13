<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Inheritance

Layer: **yggui (L1)** — this repository IS the platform layer of the design
system: `yggui` (the component vocabulary), `yggui-contract` (the wire and
orientation contracts), `emd-renderer` (what a document IS), `yggui-icons`
(the icon blocks).

Parent: Dioxus components (L0 — component and event substrate). The yggui
layer adds semantic theme tokens, typography, row anatomy, focus, ribbon,
rail and feedback patterns. Dioxus alone does not prescribe the yggui visual
identity.

Children: every L2 app design language (yedit, ychrome, ytop, kasten, …)
and, through them, every L3 project overlay. The base design notebooks ARE this repository
(`design/notebooks/*.emd`): the component vocabulary exhibited. Component
work here is argued from those pages, never from source alone. The **ydesign**
app is the reader; it ships no notebooks of its own.

Undefined decisions inherit DOWNWARD is false by definition: children
inherit from this layer; this layer invents nothing a parent (Dioxus)
already decided. Explicit local decisions in children override named rules
here, with rationale and specimens; structural accessibility, state
ownership and truthful feedback remain requirements at every layer.
