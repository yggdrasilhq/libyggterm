<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# The design/ layout and the emd-renderer extensibility contract (1.0.0)

Owner specification, 2026-09-13. Two decisions land together because one
enables the other: every repository that ships UI carries the `design/`
layout (root `DESIGN.md` pointer page + `design/` tree — the contract is
`yggdrasilhq/ydesign`, `docs/design-layout.md`), and **the emd-renderer is
extensible by design**: `.emd` is markdown plus typed fence extensions, and
the renderer is the registry where extensions live.

## The renderer is a base, not a ceiling

`emd-renderer` exists so that every pipeline app renders ONE markdown
superset instead of inventing N. Extensibility is the mechanism that keeps
it one engine while the grammar grows:

- **Base extensions ship in libyggterm.** Anything the whole fleet renders —
  the analytical components of `spec-emd-components.md` (plot, sparkline,
  metric, query, datagrid, agentfinding) and the interactive notebook embeds
  ydesign uses to mount mini yggui/Dioxus apps inside a notebook page — is a
  base extension: typed contract, parser fold, bounded error card, in this
  repository, released by tag with everything else.
- **Private extensions stay in the product's pipeline repo.** A product that
  needs a renderer extension it cannot publish grows it in its own modular
  extension checkout (the `yggui-priv-modules` pattern: one repo of renderer
  extensions composed by that product's pipeline — practice-rs runs one).
  The extension names its base-version dependency; the notebook that needs
  it says so in its front matter. Private extensions NEVER fork the parser:
  they consume the crate's extension point the way base extensions do.
- **The renderer applies to any file that builds over markdown.** `.emd` is
  the conventional extension for such files. Degradation is bounded: a fence
  rendered without its extension produces the bounded error card with the
  source preserved — never a crash, never silently dropped content.

## What this means for ydesign notebooks

The ydesign interactive notebooks are `.emd` files at
`design/notebooks/*.emd` in each repository; the assets they cite are
organized by kind under `design/assets/{components,fonts,icons,img,...}`.
The base notebooks ship in the ydesign repository; every product repository
carries its own notebooks in the same shape and registers with
`ydesign init` so they appear on the shelf namespaced. Nobody needs the
ydesign app to work in a repository — only `design/notebooks/*.emd` need it
to render.
