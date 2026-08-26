# Spec: EMD analytical components

**Status:** version 1 model, parser, bounds, deterministic plot layout, and
yggterm host rendering are implemented. App-routed component controls, richer
statistical transforms, facets, export, and persistent scale domains are next.

## Purpose

Extended Markdown needs analytical objects that remain source text. A Ytop
trace plot, a finance comparison, and an Axiom-like query/results/agent
workbench should not invent three rendering systems. They should serialize one
typed tree that a person can inspect, an agent can reproduce, and every
libyggterm host can paint consistently.

The component source is inert JSON in a fenced `emd` block:

````markdown
```emd
{"version":1,"kind":"sparkline","spec":{"label":"CPU","values":[12.0,null,41.0],"value":"41%","evidence":{"question":"Is CPU rising?","source":"/proc/stat","window":"30 s","freshness":"2 s","units":"percent","state":"observed","reproduction":"ytop --json"}}}
```
````

`null` is a missing observation and draws a gap. It is never coerced to zero.
Unknown versions and malformed JSON render a visible local error with the
source preserved. No HTML, JavaScript, expression language, callback, or query
is executed by the parser.

## Version 1 vocabulary

`grid` and `panel` compose children. They are neutral layout primitives, not
dashboard semantics. Their children can be:

- `plot`: line, step, area, bar, or point series with axes, grid, legend,
  colourblind-safe defaults, gaps, and hover tooltips;
- `sparkline`: a compact bounded series plus exact value and optional delta;
- `metric`: a labelled exact value with neutral/good/warning/critical tone;
- `query`: a titled language/source block and status;
- `data-grid`: bounded columns and rows for precise evidence;
- `agent-finding`: summary, findings, next question, and evidence boundary.

This tree is sufficient to model a workbench with a dataset/query panel, a
results table or plot, and an agent finding beside it. A future finance book can
use the same arrangement for assumptions, prices, scenarios, and commentary.

Controls are declarative `{label, action, value, primary}` records. A document
host without an application action route renders them visibly disabled rather
than pretending a click worked. Routing them through the generic pane-action
channel is a later host capability; the source grammar does not change.

## Evidence is part of the component

Every analytical leaf carries:

```text
question, source, window, freshness, units, state, reproduction, observed_at?
```

`state` is one of `observed`, `collecting`, `silent`, `unavailable`, `stale`,
or `uninstrumented`. This is not decoration. A missing probe and a measured zero
must remain distinguishable in source, pixels, and agent reads.

## Plot ownership

`emd-renderer` owns data bounds, nice ticks, missing-value segmentation, x/y
placement, grouped-bar geometry, area baselines, path generation, and the
default Okabe-Ito-derived palette. It returns a UI-neutral scene. The host only
translates scene primitives into native SVG and supplies theme ink for grid,
axis, surface, and prose colours.

Version 1 uses bounded inline points and recomputes the scene when an app
publishes a new document version. That gives Ytop live plots without blocking a
pane GET or adding a renderer-private data fetch. A later evidence-reference
form may update data without reparsing the block, but it must preserve the same
typed evidence contract.

## Bounds and failure behavior

- source: 256 KiB per component block;
- tree: 64 components;
- plot: 16 series and 2,048 total points;
- sparkline: 2,048 points;
- data grid: 500 rows;
- plot height: 120–720 px;
- grid: 1–4 declared columns.

Violating a bound is a component error, not a truncated graph. Producers must
downsample and state their transform before crossing the document boundary.

## Typography and quotes

Analytical components inherit the host document palette and the shared prose
tokens. Markdown blockquotes use a one-pixel line in the current foreground ink
and italic text. Consequently the line is black/dark in a light palette and
inverts with foreground ink in dark mode; accent blue is reserved for actual
links and state, not quotation punctuation.

## Acceptance

1. valid and malformed fences each align to one top-level source range;
2. raw source is never executed or inserted with `innerHTML`;
3. null data produces a visual gap;
4. scenes are deterministic for identical input;
5. every numeric analytical leaf exposes evidence metadata in source and UI;
6. renderers fail compilation when a new component variant is unhandled;
7. visible work is inspected in both light and dark themes at a real viewport
   size, not approved from JSON alone.
