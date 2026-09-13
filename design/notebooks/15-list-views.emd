<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# List views, one reading grid, distinct purposes

**edition 2026-09-07 · rev 1**

Status: a proposed Dioxus component and working demo now live in
`specimens/src/lib.rs` and `specimens/src/main.rs`. The current book contents
is the live failure example. The demo is a standalone staging target; this
chapter is not complete until it mounts that component inline and passes
interaction and pixel review. It is not yet an extracted yggui component.

## Begin with the contents page

Open the book header. Follow the left edge of its title and introduction down
to the first chapter. In the reported view, the chapter list jumps left of
the reading column. Its titles and descriptions are much smaller than the
introduction, yet it occupies a large blank canvas. The rows read as a
sidebar accidentally placed in a book.

Source inspection on 2026-09-07 explains the mismatch. The host document
Markdown wrapper has a 980px maximum width, full width, auto side margins and
28px horizontal padding. The ordinary document list row instead has fixed
20px side margins, an 860px maximum width, 12px horizontal padding, and a
32px icon slot even when no icon exists. Row titles are 13px; subtitles are
11.5px. These are observed implementation values, not the desired tokens.

The defect is not that all text should be centered. **Center the reading
column; left-align its text.** Introduction and list need one parent grid.
Do not fix an independent row's offset with a screenshot-specific margin.

## Three lists, three jobs

| Context | Human job | Appropriate behavior |
| --- | --- | --- |
| Navigation rail | Switch location among many destinations | Compact rows, selected/current state, deliberate disclosure |
| Editorial contents | Understand the book and choose what to read | Readable chapter titles, quieter descriptions, ordered rhythm |
| Operational list | Recognize an item and act on it | Distinct identity, metadata and action regions; explicit outcomes |

One component family may expose these variants. They should share interaction
semantics and tokens, not force the rail's density onto every surface.
Increasing every rail row to book size would be the opposite mistake.

## Proposed editorial anatomy

The heading, introduction and contents share the same centered, responsive
content container. Width and side padding belong to that container, not to
each child row. A row fills the available column and measures border and
padding consistently. The contents begins on the same grid as its heading.

Use a narrow ordinal column and a flexible text column. Align title and
description with each other; align all ordinals consistently. Either align
the ordinal edge with the section heading, or deliberately hang the ordinals
in a documented gutter. Never get the gutter accidentally from a missing icon.
An absent icon has no reserved slot unless the entire list requires alignment
with other rows that genuinely have icons.

The editorial tokens are the consultant's, not the wiring seat's
(gpt-6-astra via codex exec at LOW effort, consulted 2026-09-08; chain entry
`lores/chain-of-thought/0002-ydesign-toc-list-astra.md`): title 18px/700,
description 15px, ordinal 13px tabular and right-aligned, list gap 18px,
row padding 14px block, ordinal column 2.25ch, column gap 16px, accent
#265f63, hover #f3f6f5. The current chapter reads by accent ink and a 2px
underline rule, never a card. They live as custom properties in
`specimens/assets/book.css` and remain starting values for live pixel
critique, not constants. Long titles and descriptions wrap; row height
follows content. Do not use ellipsis to hide the words that distinguish
chapters.

Keep the resting page quiet. Avoid a stack of cards, persistent arrows,
decorative badges and heavy separators. A subtle hover treatment and a clear
focus-visible outline make the full navigation target apparent. Whitespace
separates entries; it must not separate the list from its own introduction.

## The interaction is part of the typography

Each chapter destination is a real semantic link, or an equivalent accessible
navigation control with the platform's expected keyboard behavior. The entire
row is the target, not only its tiny title. Focus order follows reading order.
Announce the current chapter as current, not as an unrelated selected option.

Secondary actions, if present in an operational variant, have separate names
and hit regions and must not also invoke the row destination. A contents list
normally needs no row action menu. Return to contents restores the reader's
place and focus. Disclosure on the sidebar book header is a different action
from opening the book; do not conflate them.

## The required Dioxus mini-app

Use twelve invented chapters, including a long title, a two-line description,
and a title-only entry. The same component and fixtures run in three contexts:
rail, editorial contents and operational list. Context is an explicit prop,
not a guessed pathname or app title.

The editorial example must navigate to a small chapter view and return to the
same focused entry. Inspect controls can switch current/proposed layout,
viewport width and fixture length; Reset restores all state. A rejected
variant reproduces the independent-container mismatch to teach its cause.
Do not satisfy this with a list-shaped image or a click counter.

## Review gates

- Compare the container/heading/list edges at wide and narrow widths. The
  declared grid relationship must hold without per-width correction offsets.
- At 200% zoom, preserve content, order and usable targets; wrap rather than
  shrink. Check the longest title and description, not just the first row.
- Navigate every entry by keyboard, open one, then return. Verify focus and
  reading position, not merely the accepted event.
- Test no-icon and mixed-icon operational fixtures. Empty space must have an
  explicit alignment purpose.
- Record component revision, variant, viewport, scale and fixture when a
  human critiques the result. Accepted changes update the shared component
  and this chapter together.

The current owner is the host's document list-row renderer in
`yggterm-shell/src/shell/right_rail.rs`, alongside its Markdown wrapper.
The generic geometry, density and semantics belong there or in its extracted
shared component, not in ydesign-specific offsets. This chapter does not claim
that the renderer repair or interactive specimen has shipped.
