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

## Hosted-component contract — the page as a reproducible executable test host (2026-10-27)

A design page is an EXECUTABLE HOST of cases, not a static mockup (chain
0028 Q4). The runtime layer that makes this a reproducible test surface is
`crates/emd-page-host` (this repo), implementing:

1. **Versioned validation.** Every ` ```emd ` fence validates through the
   documented parse path. Wire version is checked FIRST (floor and
   ceiling): an unsupported version is reported as
   `"unsupported EMD component version N; this renderer supports 1"`, a
   pre-floor version requires an explicit migration decision — never a
   silent reinterpretation. Source/bounds checks are preserved
   (256 KiB source, 64 components, series/points/rows/height bounds).

2. **The page-host manifest (`page.host.json`**, contract rev 1, beside
   the page). The page's executable behavior is DECLARED before
   inspection: implementation dependencies (pinned component crate +
   revision), the design dependency (resolved token-profile digest), the
   synthetic fixture (the ONLY initial state `reset` restores) with a
   typed `mutable_keys` surface, digest masks WITH reasons, and the
   selftest's declared plant vocabulary. A page without a manifest is a
   legal read-only prose host.

3. **Dependency resolution with actionable failures.** Unresolved deps
   are reported before any render, naming the manifest key and the
   expected shape (`workspace:<semver>` or `component:<wire-version>` —
   a typo is an unresolved dependency, never a silently-prose page).
   The resolved token digest (page content + pinned profile) joins
   evidence provenance.

4. **Deterministic render evidence.** Every component reduces to a
   scene digest over its UI-neutral scene (the renderer's own
   geometry): identical input renders identical digests. The fixture
   state joins the page digest — altered fixture state can never look
   like an unaltered page.

5. **Fixture/reset/readiness.** `fixture_set` only through declared
   mutable keys; `reset` reproduces EXACTLY the manifest-declared
   initial values; `ready` = validated page + resolved deps + digests
   stable on recompute. Two fresh-reset rounds MUST agree — that is the
   runtime shape of the ux-falsify fixed point.

6. **Controlled lifecycle.** load → host → reset → unload. Each fresh
   load is a fresh generation; NOTHING survives an unload: re-executing
   a case starts from the declared fixture (repeated navigation/reload
   leaves no residual fixture state and no duplicate host registration —
   a re-run MUST assert digest equality, which the runner does).

7. **Error presentation with actionable locations.** Fence failures
   carry fence index, byte range, line:col of the opener, tree path
   (`root.grid.children[2].plot.series`), and the violated rule; bounds
   and evidence failures name the offending field
   (`evidence.units must not be empty`). Malformed fences stay VISIBLE —
   error cards, never a silently-dropped graph.

8. **The common evidence contract (ux-falsify).** `emd-page
   check|render|run|selftest` emit hashed bundles in the ux-falsify
   manifest schema (identity/subjects/environment/fixture/replay/
   capture/evaluation/provenance). `selftest` plants defects in a
   SYNTHETIC candidate and must detect ALL declared plants — a red
   selftest suspends page verdicts. PIXEL capture stays UNVERIFIED at
   this layer: scene-truth is the structural capture; pixel proof is a
   host-surface adapter's job and is never faked.

**Fixture corpus rule:** pages shipped in the repo hold only SYNTHETIC
data; the real corpus for CI is the shipped design/notebooks pages plus a
synthetic page derived from a consumer miniapp's SECTION SHAPES
(`crates/emd-page-host/corpus/`). No private data enters the repo.

## Migration behavior

- **Grammar version up (1 → 2)**: a host MAY add kinds; it must keep v1
  pages rendering constructor-identically and must name the floor it
  executes (see 1). A publisher crossing versions declares the migration
  in the manifest (`contract_rev` bump + page revision), and old pages
  keep their declared pins — never silently widened.
- **Notebook/cache pages WITHOUT a manifest**: they keep working as
  read-only hosts (validation-only). Adding a `page.host.json` is the
  migration path for a page that wants fixture behavior, a declared
  selftest, or a dependency pin; the contract rev in the manifest is
  the version the TWO sides agree on.
- **Token digest drift**: a page that consumed a pinned profile whose
  digest changed must re-declare the new digest under a NEW page
  revision (or reproduce the per-page digest) — a digest mismatch is an
  explicit failure, never an auto-refresh.

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
