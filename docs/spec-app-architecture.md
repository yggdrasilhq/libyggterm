# Spec: app architecture — the three content tiers

**Status:** RECORDED 2026-08-07, user-directed. His framing, which is the whole
question this answers:

> *"How do we make ychrome and later how will we make cellulose? This is a spec
> discussion first. If all libyggterm apps can be made like yedit then that is
> the way to do, if some apps cannot be made like that then we need to design
> the layers."*

**This spec lives here, not in the host.** The host is one consumer of the
contract; the apps are the others, and the thing being specified is what an app
IS. libyggterm is also the repo that currently fails to draw the distinction —
its README says *"a library apps must LINK is MPL; an application is GPL"* and
stops, which does not tell an app author whether they are writing a Dioxus
window or a headless daemon. That silence is the defect this closes.

**Owner surfaces:** this file (the tiers and the rules), the host's widget
vocabulary (`AppPaneWidget`), `yggui` (the window-owning component library),
`yggui-contract` (types both sides share), and the host skill
`libyggterm-surfaces` (the build recipe for one app).

## 1. The evidence: three consumers, three architectures, and nothing said so

Measured 2026-08-07, from the manifests rather than from memory:

| app | content painted by | chrome painted by |
|---|---|---|
| **yedit** | the host, from a JSON widget schema | the same schema |
| **ychrome** | its OWN engine (GTK + WebKitGTK + wry) hosted as a native surface | **the schema** |
| **cellulose** | undecided — *"the implementation stack is still open"* | undecided |

Neither yedit nor ychrome links `yggui`. Neither is a Dioxus app. yedit is a
headless daemon plus a thin CLI that speaks JSON over loopback HTTP and declares
itself with an OSC escape; ychrome is a browser that additionally declares two
rail panes.

⇒ **The finding that resolves the question: every app is ALREADY a schema app
for its chrome. They differ only in who paints the CONTENT.** So "can all apps
be made like yedit" has two answers, and separating them is the design:

- **for chrome — yes, always, and it is a law (§2);**
- **for content — no, and the layers are §3.**

## 2. LAW — chrome is always the schema

Every app declares its rails, panes, toolbars, settings and row lists as
widgets. No app draws its own chrome, including an app that owns a native
content surface.

**Why it is a law and not a preference.** The schema plane is the only one where
theme, row drag/reorder, in-place rename, notifications, context menus, faithful
screenshots, `dom-eval` and app-control are free, uniform, and identical across
every app. Every pixel an app draws for itself is a pixel outside all of that.
ychrome already proves a foreign-engine app can honour this — it ships an entire
web engine and still declares its vault and settings panes as widgets.

⛔ **A corollary with teeth:** an app that finds itself wanting to draw a
*button* natively has mis-tiered its chrome, not discovered a gap. Grow the
vocabulary (§3, Tier C) or fix the app.

## 3. The three content tiers, and the one question that picks between them

> **Who must paint the pixels, and why?**

### Tier A — the host paints it, with widgets it already has

The app emits `section` / `label` / `tabs` / `search-box` / `text-input` /
`number-input` / `toggle` / `button` / `list-row` / `toolbar` / `markdown`, and
ships **no UI code at all**.

Fits: document readers, list-and-detail corpora readers, form-shaped tools,
anything whose content is prose, rows, tables and todos. This is the cheapest
app that can exist on the platform and it should be the default assumption.

### Tier C — the host paints it, once it learns ONE new widget kind

The content is not expressible today, but does not need a foreign engine. The
answer is to grow the vocabulary by one declarative widget, which the host then
renders natively — and **every app gets it**.

⭐ **This is the tier that is easy to miss, and missing it is expensive.** The
instinct when a widget is absent is to reach for Tier B and draw the thing
inside a native surface. That serves one app and pays the whole Tier B tax
(below) forever, for everyone.

**Admission rules for a new widget kind** — both must hold:

1. **At least two apps want it.** One app's need is a feature request; two is a
   vocabulary gap. A graph view qualifies immediately: every document-corpus
   reader wants one, and a corpus with thousands of links is unreadable without.
2. **It is declarative — data in, events out.** A widget is a description the
   host renders and reports interactions on. An imperative drawing API handed to
   the app is Tier B wearing Tier C's clothes, and it defeats the point.

### Tier B — only a foreign engine can paint it

A web engine, a remote-desktop client, a video decoder. The host hosts a real
native surface; the app owns everything inside it.

⛔ **Tier B is a COST, not a choice**, and the costs are known rather than
theoretical:

- the host cannot faithfully screenshot into the surface,
- it cannot `dom-eval` it, so agent automation stops at the boundary,
- it does not inherit the theme,
- it composites and z-orders by different rules from the rest of the chrome,
- and every viewer of it is a second engine instance with its own lifecycle.

Take Tier B **only when nothing else can render the content**, and never to
work around a missing widget.

## 4. LAW — `yggui` and the schema are different products

The distinction libyggterm has not been drawing:

- **`yggui`** (Dioxus components) is for an app that owns **its own window** —
  one that runs OUTSIDE the host.
- **The schema protocol** is for an app that lives **inside the host's
  viewport**.

They are not two ways of doing one thing, and an app author must choose
knowingly. An app that wants both — standalone *and* embedded — is **one core
with two front-ends**, not one app with a mode flag.

⇒ **That is only affordable if the RENDERER is shared between them**, which
means a renderer that both a window app and the host need belongs **here**, in
libyggterm, not in the host's shell crate. This is the structural reason the
render extraction matters beyond tidiness; `spec-emd-renderer.md` §step 4 is the
first instance of the same move.

## 5. The layering that falls out

```
libyggterm (MPL — apps and the host both LINK it)
  ├── the widget SCHEMA types            ← an app builds typed widgets, not JSON
  ├── the app SCAFFOLDING                ← daemon-ensure, OSC declare, manifest,
  │                                         the loopback control server
  ├── the RENDERERS as they stabilise    ← emd model+parse (done), markdown
  │                                         render, graph render, grid render
  └── yggui                              ← components for window-owning apps
        ↑ links                                    ↑ links
  the HOST (GPL)                          STANDALONE apps (own window)
    renders widgets                         e.g. a spreadsheet run on its own
    hosts native surfaces
        ↑ speaks the schema over OSC declare + loopback HTTP
  Tier A / C apps  ·  Tier B apps (native content, schema chrome)
```

## 6. What this obliges — the extraction seams, in order

⛔ **Not "extract the pilot app as a library".** Measured: the pilot's reusable
surface is roughly 600 lines and the seam is in the wrong place. The leverage is
the protocol, not the app's code.

1. **The widget schema as typed Rust**, into `yggui-contract`. An app then
   depends on typed widgets instead of hand-rolling JSON, and an unknown widget
   fails at COMPILE time rather than failing the pane at runtime.
2. **The app scaffolding** — ensure-the-daemon, the OSC emitter, the manifest
   writer, the loopback HTTP shell. Roughly 200 lines that every app needs
   verbatim and that the pilot already proved.
3. **Analytical EMD components** (Tier C). Version 1 is now admitted and
   implemented: typed grids/panels, plots, sparklines, metrics, queries, data
   grids, and agent findings. They travel inside a Markdown body because their
   source must remain human- and agent-readable; the host paints their shared
   scene rather than an app painting private graph pixels.
4. **The renderers move out of the host as they stabilise**, markdown first.

## 7. The open measurement that decides one app

Whether a spreadsheet grid — virtualised, tens of thousands of cells,
selection-heavy, 60fps — survives the host's shell-DOM path is an **empirical
question, not a taste one**. It decides Tier C versus Tier B for that app and
nothing else does.

**Run it before writing the app**: a synthetic grid widget at realistic cell
counts, measured for scroll latency, selection latency and paint cost, on the
host's real rendering path. If it holds, the grid is a widget and the standalone
and embedded front-ends share one renderer. If it does not, the app is Tier B
and pays the tax knowingly.

⚠ Recording the honest state: this has not been run. No tier is assigned to a
spreadsheet by this spec.

## 8. Migration order

0. ✅ This spec.
1. Widget schema types → `yggui-contract`.
2. App scaffolding → a crate here; the pilot adopts it and stays behaviourally
   identical (that is the test).
3. The grid measurement (§7), because its answer changes what §4's "two
   front-ends" costs.
4. ✅ EMD analytical components: model, bounded parser, deterministic scene,
   host render, then Ytop as the first live producer. App-routed component
   controls and export remain follow-on work.
5. Markdown render extraction (`spec-emd-renderer.md` step 4), which unblocks a
   standalone document app.

⚖ **The rule to apply when this spec is silent:** ask §3's question — *who must
paint the pixels, and why* — and prefer the lowest tier that can answer it. The
tiers are ordered by cost, and the cost is paid by every app, not just the one
choosing.
