<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Roadmap, demanded components

**edition 2026-09-07 · rev 2**

Components the apps have demanded, each with its forcing consumer and the
layer that grows. This page is the design system's *defect and demand
ledger*: "the sidebar needs polish" becomes a filed, scoped, owned row. The
per-chapter conversion status that pairs with it lives in the
[chapter inventory](chapter-inventory.md).

The admission rule is the platform's own: a component enters the vocabulary
with a **forcing consumer**, and becomes a schema kind when a **second**
consumer wants it. Between those two points it lives in the app that needed
it first, visibly, here, so the next app finds it instead of reinventing it.

## The queue

| **Keyboard navigability law (owner 2026-09-10)** — every interactive list/row navigable with Up/Down arrows, Windows-style, throughout the UI; basic keyboard presence is intuition, not a feature | every document app + the shell | yggterm-shell row renderer: roving focus + arrow handling on document list-rows | Named by the owner |
| **Hover accent from the theme engine** — interactive rows hover to an accent that SUITS the active theme, derived by algorithm, never hardcoded; blue family when the theme has no accent | shell row renderer | Algorithm spec consulted (gpt-6-astra, chain 0010): OKLCH blend 84/16 toward ink, C clamp 0.025-0.060, L delta 0.04-0.10, contrast >= 4.5 with search-and-gamut-map | Spec ready, implementation pending |
| **Theme picker modal with preset accent combinations** — the theme engine modal supplies pre-existing combinations (Deep Teal #265F63, Ink Wash #24343B, Slate Blue #4568A0, Warm Bronze #93642F, Plum #805780, each with derived hover per the algorithm) | theme engine modal | Presets in chain 0010 | Named by the owner |

| Demand | Who needs it | Layer that grows | State |
|---|---|---|---|
| **Mini-app conversions still owed**, Live Sessions row anatomy, the ychrome sidebar, the ychrome vault sidebar (as the Complex-sidebars working example) | every document app | own chapters with real mini-apps (studies exist; the shared row/renderer pieces their hosts own) | Row anatomy DESIGNED (gpt-6-astra, chain 0005, mock 0005-sessions-mock.png) and staged as specimen study 04; host-renderer conversion still pending |
| **Left-ruler scroll component**, marks on a vertical rail (user turns in a transcript; months/years in a photo library; headings as bookmarks) | transcripts, media, readers | `yggui`, one component owning rail geometry; hosts supply `{offset, depth, label}` | Spec'd (see *emd & notebooks*); first host lands it |
| **Plot-quality charts**, ggplot2/plotly-grade statistical plots as components | telemetry notebooks, analytics | emd `plot` grows facets/scales; the renderer owns layout | `plot` exists (line/area/bar/point/step); quality bar tracked here |
| **Full-width devtools mini-app**, a network-timing inspector as one notebook block, scrollable, reusable | web development inside notebooks | emd component composed of `panel` + `datagrid` + `plot`, driven live by a document-version refresh | Demanded |
| **Half-width "modern top"**, a top-style live block that does NOT span the notebook | telemetry | emd `grid` composition of `sparkline` + `metric` | Demanded |
| **Full-width calendar**, Notion-class month component | knowledge app | emd component (new `calendar` kind, Tier C: admitted when its second consumer lands) | Demanded |
| **Axiom-class log/query views**, devtools-grade filtering over logs | log viewers, tracing | emd `query` + `datagrid` grow saved views | Demanded |

## Notebook review additions

The contents-page list mismatch is diagnosed in **List views**: Markdown and
document rows use independent width/margin rules, iconless rows reserve an
empty icon slot, and rail-density typography is applied to editorial content.
The required repair is a shared context-aware list layout, not a TOC offset.
The reusable Dioxus component ships as a staging target (`specimens/`,
deterministic fixtures, reset, critique draft); the renderer repair in
yggterm-shell and the specimen's inline mounting remain pending.

| Finding | Owner and acceptance | State |
|---|---|---|
| Detached ribbon panel, dark tab slab and weak command groups | Shared ribbon renderer; continuous shell tabs, explicit pinned/temporary modes, grouped commands, keyboard and pixel proof | Proposed anatomy in Ribbons; component change pending |
| Vault setup repeated across pages; generic credential bubbles obscure site identity | Browser sidebar content plus shared row renderer; contextual setup, site marks, distinct credential labels, short Fill/details paths | Measured row: 264 × 38, icon 15, bubble 26; proposal in Complex sidebars, not shipped |
| EMD panel controls rendered disabled | Document host's panel-action routing; demonstrate a real state-changing control before calling it interactive | Filed; current notebooks use supported schema-row exercises |
| Notebook images capped at 560 × 320 | Document image renderer; distinguish a reading illustration from an inspectable full-width design specimen | Measured in shadow; diagrams redrawn to this cap so labels stay readable. Full-width/zoom affordance remains pending |

The exercises model transitions, not production ribbon or vault components.
Passing their actions does not validate a repaired component. Private runtime
captures stay outside this public repository; published studies use invented data.

## The first filed defects

The design system opens its ledger with the two defects that motivated it.
Fixes land with pixel proof at the component layer, and the ledger rows move
when they do:

1. **Settings-rail top rows**, explanatory prose sat *beside* the toggles as
   multi-line paragraphs: control row proportions broken, toggle stranded in
   dead space. Fix = the **short-phrase rule** (one muted phrase, under the
   control, never a paragraph beside it). See *Forms & settings*.
   **FIXED 2026-08-28** (yggterm `lane/design/settings-short-phrases` → main
   `eb46e4dd`): all five rows carry the ≤9-word phrase under the control,
   explanations moved to row tooltips; the accepted before/after pair is the
   catalogue's forms-rail exhibit.
2. **A browser tab rail's header**, the profile pill and `+` rode the
   heading as solid accent fills, stacked under the nav row and omnibox:
   three bands of loud before any row. Prescription = the **header anatomy**
   standard (title row → tool row → section heading) with header actions at
   ICON weight, muted, accent on hover only
   (`session_row_action_button_style`); the accent budget belongs to the
   rows' own states. **PARTIALLY LANDED on yggterm main** (verified in
   `right_rail.rs` `WebTabsRailBody`, 2026-09-07): the floating solid-accent
   `+` is gone, group-scoped, icon-weight, on the group head row, and rows
   render from one arm of the shared renderer. The profile pill remains at
   badge accent BY RECORDED DESIGN (the owner specified profile switching as
   the rail's dropdown anchor). Owed: a header-anatomy pixel pair for the
   full rhythm, judged under the standard. See *Sidebars*.

## How this page grows

An agent that meets an ugly surface files the divergence here (or in the
app's own overlay notebook) *before* fixing it: what is wrong, which rule it
breaks, the screenshot pair. A defect with a name and a rule stops being a
thousand small tweaks, it becomes one row, one owner, one fix.
