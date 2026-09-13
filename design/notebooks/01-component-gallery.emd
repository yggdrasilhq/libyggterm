<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Component gallery

**edition 2026-09-07 · rev 1**

Every component an app may reach for, and the one-owner rule each encodes.
Below the prose, this page appends **live specimens**, real widgets painted
by the host, so the comparison against your own surface is a screenshot away.

The rule underneath the whole gallery: **one visual decision, one owner.**
Consumers consume. A surface that restyles a shared control is a second
encoding, and second encodings drift.

## yggui components (libyggterm · L1)

| Component | Purpose | The rule it owns |
|---|---|---|
| `conversation::*` | The agent-transcript-as-document: `ConversationColumn`, `UserTurn`, `AssistantTurn`, `WorkGroup`, `WorkRow`, `DiffStat`, `ChangedFileChips` | The only chat surface. A second hand-rolled transcript in any app is the bug this set exists to prevent |
| `prose::*` | The type system for rendered markdown: `ProseTokens::{document,conversation,rail}` | No host spells a face, size, leading or tracking of its own, change `prose.rs` first, then docs |
| `chat_input` | The composer: one rounded box, context upper-left, send lower-right | Host owns value/state; the box never self-clears |
| `chrome::TitlebarChrome` | Three-slot mirrored titlebar + window controls | Told a `SidebarEdge`; never decides a side itself, `ChromeOrientation` answers that |
| `rails::SideRailShell` | Docked or auto-hide side rail | Host owns the reveal state machine; geometry stamps are contract |
| `drag_tree` + `drag_visuals` | Headless tree drag-reorder + ghost/drop visuals | One drag grammar per window (`RowDragGesture`); thresholds are constants, not choices |
| `dpad::ScrollDpad` | 3×3 glass scroll pad for any scrolling surface | Host owns what the actions mean |
| `notifications::*` | Toast viewport, cards, tones, anchors | The anchor belongs to the viewport, not the toast |
| `pill_toolbar` | Floating translucent toolbar (find, steppers) costing no layout | Owns shape/material only, no state, ever |
| `split_button` | One sticky primary action + caret menu | Menu surface opaque; per-item accent allowed |
| `otp` | 6-cell code entry with native paste bridge |, |
| `icons` (`yggui-icons`) | lucide ISC path constants at stroke 1.5 + the `Icon` wrapper | The only icon source. A new inline `<svg>` in an app layer, or a unicode `↑↓→←×` in a control, is the defect; see *Icons* |
| `motion` | Easing curves + durations | Motion is functional; desktop-fast, never rubbery |
| `theme` | Gradient-theme plumbing over `YgguiThemeSpec` | Stable path: no compositor blur, no alpha-only chrome |

## Schema widgets (a Tier A app declares; the host paints)

The chrome of every libyggterm app is a **wire schema**, not app-drawn DOM.
The vocabulary grows only when a second app needs a kind.

| Widget | Notes |
|---|---|
| `section` | Opt-in `card: true` wraps a **form** in the Settings card. A list is never a card |
| `label` | `muted` for notices and status lines |
| `list-row` | The shared row engine: `depth`/`expanded`/`expand_action` (trees), `reorder_action` (drag), `status` (`durable`/`transient` dot, the app names a class, the host owns the colour), `menu`, `rename`, `actions` |
| `tabs` | Renders on the one segmented control. Every mode switch is this, never a hand-rolled pill |
| `search-box`, `text-input`, `number-input` | Fields wear the ONE field skin. `text-input` adds `multiline`, `line_numbers`, `word_wrap`, `secret`, `stored`, `actions[]`, `value_key` |
| `toggle`, `button` | `footer` buttons honour `primary: true`, a form's Save is pinned, not scrolled |
| `markdown` | A page of emd, rendered through `yggui::prose` typography |

⭐ **Body vs bar on a document surface.** In the viewport, `markdown`,
`list-row` and editor widgets render in the BODY flow; every other kind
(`section`, `label`, `toggle`, `search-box`, `tabs`, `button`) paints into
the TOP BAR strip. This was caught by ydesign's own pixel proof, its first
exhibition page appended rail-style controls after the prose and they
squeezed into a horizontal bar above the page. A control that wants to sit
in a flow belongs in a rail pane, or as prose in a document.

## emd components (a ```emd fence carries JSON; the host renders)

`grid`, `panel`, `plot`, `sparkline`, `metric`, `query`, `datagrid`,
`agentfinding`, the analytical vocabulary. Contracts and authoring rules in
the **emd & notebooks** page. One taste rule here: a component carries an
`evidence` block because a number without provenance is decoration, not data.

## What belongs where, deciding in one minute

- Is it **prose, a table, a reading**? Markdown + emd components.
- Is it a **control or a status**? A schema widget.
- Is it the **app's own domain object** (a tab, a file, a notebook)? A
  `list-row` wearing the row engine.
- Is it a **new interaction no widget can answer**? First grow the schema
  vocabulary (Tier C: one new host widget, two consumers), and only when a
  foreign engine is truly required consider Tier B, and pay its tax
  knowingly (no faithful screenshots, no dom-eval, no inherited theme).

## The specimens below this prose

The appended block is *live*: real rows with their status slots. Rail-only
cards and toggles are not rendered in this document appendix. Screenshot this
page and compare your surface against it, same fonts, same tint, same row
anatomy. If yours disagrees, the fix belongs in the shared component, not in
your stylesheet.
