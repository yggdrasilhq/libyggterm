<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# emd & notebooks

**edition 2026-09-07 · rev 1**

![A rendered notebook is a reading surface, not raw syntax](%ASSETS%/catalogue-emd-reader.png)

Author the smallest example that makes the decision visible. Pair an observed
or clearly labeled proposed image with the reason it matters, then a small
exercise with reset and an observable outcome. Current EMD panel controls are
disabled by the host; ydesign's interactive exercises use supported schema
actions. Do not paste JavaScript or HTML into Markdown and expect execution.

`emd-renderer` is the fleet's extended-markdown system: one general-purpose
wheel so every scrolling UI, editors, transcripts, telemetry notebooks,
readers, renders beautifully without reinventing it. **emd says what a
document IS; `yggui::prose` says how it reads.** A host parses with emd and
renders each block through the prose type system; neither imports the other.

## The model, in five rules

1. **Source-decorated, not block-model-at-rest.** The markdown SOURCE is the
   document; blocks are ranges over it. Editing splices the source and the
   round-trip stays byte-faithful outside the edited block, the trust
   invariant every grammar addition must keep.
2. **Superset grammar grows as TYPED VARIANTS** on `MdBlock`/`MdInline`.
   Unknown variants fail loudly at compile time for renderers, that is the
   feature, not a cost.
3. **Raw HTML is dropped by construction.** Note-derived content never
   reaches a JS context.
4. **Fluidity at block granularity**: click-a-block-to-edit, then caret-line
   reveal, never a leap to full WYSIWYG.
5. **The layering is fixed**: model + parse (pure, UI-free, server-safe) in
   the crate; the render lives with the host until it stabilises, then
   extracts.

## The emd component contracts (fenced JSON: ` ```emd `)

Fenced JSON is the interchange: an agent authors it, a person inspects it, a
renderer never executes notebook source. Version is `1`; bounds: 64
components per tree, 16 series, 2048 points, 500 grid rows. Any analytical
component without its `evidence` block is invalid, provenance is not
decoration.

`evidence` (required on plot/sparkline/metric/query/datagrid/agentfinding):

```json
{
  "question": "What is this number answering?",
  "source": "the probe, file, stream or query that produced it",
  "window": "last 15 min",
  "freshness": "4 s",
  "units": "percent",
  "state": "observed",
  "reproduction": "the command that regenerates it"
}
```

`state` is one of: `observed`, `collecting`, `silent`, `unavailable`,
`stale`, `uninstrumented`. A missing sample is a JSON `null`, a **gap**,
never a zero.

| Kind | Shape (abridged) |
|---|---|
| `grid` | `{columns: 1..4, gap_px, children[]}`, composition only |
| `panel` | `{title, subtitle?, controls?[{label, action?, value?, primary}], children[]}` |
| `plot` | `{title, mark: line\|area\|bar\|point\|step, height: 120..720, legend, series[{name, color?, units?, values[{x, y(null=gap), label?}]}], evidence}` |
| `sparkline` | `{label, values: [number\|null], value?, delta?, evidence}` |
| `metric` | `{label, value, detail?, delta?, tone: neutral\|good\|warning\|critical, evidence}` |
| `query` | `{title, language, source, status?, controls?, evidence}` |
| `datagrid` | `{title, columns[], rows[[]] (each row matches columns), compact?, evidence}` |
| `agentfinding` | `{title, summary, findings[], next_question?, status?, evidence}` |

A component that fails to parse renders a bounded error card naming the
problem, it never takes the page down with it.

### A worked example

```emd
{"version":1,"kind":"grid","spec":{"columns":3,"gap_px":12,"children":[
  {"kind":"metric","spec":{"label":"Components","value":"8","detail":"kinds in the vocabulary","tone":"neutral","evidence":{"question":"How large is the analytical vocabulary?","source":"emd-renderer components.rs","window":"this build","freshness":"static","units":"kinds","state":"observed","reproduction":"grep -c '^[a-z]*(' components.rs"}}},
  {"kind":"metric","spec":{"label":"Bounds","value":"2048","detail":"max plot points","tone":"neutral","evidence":{"question":"What bounds keep a notebook bounded?","source":"components.rs","window":"this build","freshness":"static","units":"points","state":"observed","reproduction":"grep MAX_POINTS components.rs"}}},
  {"kind":"metric","spec":{"label":"Version","value":"1","detail":"wire version","tone":"good","evidence":{"question":"Which wire version do these fences speak?","source":"COMPONENT_VERSION","window":"this build","freshness":"static","units":"version","state":"observed","reproduction":"grep COMPONENT_VERSION components.rs"}}}
]}}
```

## Notebooks as an app pattern

Base notebooks are **source-controlled and ship inside the app** (ydesign
embeds these files; ytop does the same for its shelf). Agent-composed
notebooks live as one JSON file each under the app's data dir and may never
shadow a shipped id. A notebook page is a reading: it can be printed by the
CLI (`ydesign --notebook <id>`), diffed in review, and checked in CI without
a GUI.

## Demanded: the left-ruler scroll component

One component, three consumers, all already named, this is the design
system's next extraction, and its shape is already settled by its users:

- **A transcript** puts one mark per user turn on a vertical ruler; click to
  jump. The ruler replaces hunting through a scrollback for where you asked
  something.
- **A photo library** marks months and years at different depths, a
  skeuomorphic ruler where a year is a long tick and a month a short one.
- **A long document** tabs its headings onto the ruler as bookmarks.

Shape: the component owns the *rail* (geometry, tick depths, hover, click)
and receives an array of `{offset, depth, label?}`; the host owns what a
mark means. It is one yggui component with three hosts, admitted the moment
its first host lands. See the Roadmap page for its place in the queue.
