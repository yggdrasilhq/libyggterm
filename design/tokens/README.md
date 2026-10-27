# design/tokens — the L1 design-token SSOT (seed, 2026-10-27)

Machine-readable design tokens for the fleet's component legos. This is the
home the ux-falsify campaign decided (Sol consults 0028/0029): ONE schema,
REFERENCE PROFILES over it (T3 Code look, ZCode look, …), platform bindings
GENERATED from these files.

## Layers (Sol ruling 0028 Q4)

1. **primitives** — raw scales (zinc ramp, spacing steps, type sizes). Never
   consumed directly by components.
2. **semantic roles** — canvas, chrome, surface, text hierarchy, border,
   accent, selection, focus, disabled, status, trajectory. What components
   consume. Defined once in `roles.toml`.
3. **component aliases** — composer, message, code-block, thread-row,
   task-row, sidebar-group. Named convenience aliases bound per profile.

## Profiles

`profiles/<name>.toml` binds MEASURED values to roles. Every value carries
provenance (source repo, file, extraction date). Profiles may be visually
very different over the one schema — that is the point (chain 0028: "one
SSOT does not require one universal appearance").

- `profiles/t3code.toml` — MIT reference (`~/git/t3codereversed`, upstream
  commit e60821f0e). Light zinc-25 / dark pure-#000, glass + grain recipes,
  geometry tokens (topbar 52px, sidebar insets).
- `profiles/zcode.toml` — PATTERN-LEVEL reference only (proprietary;
  `~/git/zcodereversed`, shipped arm `.theme-zai-dark`): canvas #161616,
  chrome #202020, cards #2b2b2b, white-alpha surfaces, trajectory hexes,
  ANSI-16. No copied assets; values are measured colors only.

## Laws

- Bindings (rust/CSS/TUI theme maps) are GENERATED from these files.
  Hand-editing a generated value is prohibited — change the profile, ripe
  the generator.
- TUI consumes explicit color/cell mappings from the same roles — CSS pixels
  do not translate into cells (Sol 0028 Q4).
- A profile revision is immutable: changes create a new revision with a
  migration note (ux-falsify reference-pin law).
- Components consume ROLES, never originating-brand values (the inherit
  spec; hybrid verdict 0029).

## Status

Seed landed 2026-10-27 by the ux-falsify campaign seat. Bindings generator
+ theme loading = the jyas-webapp skeleton unit (dreamer brief D5) and the
Wave-1 continuation. Until the generator exists, these files are the SSOT
by citation: consumers reference role paths, not hexes.
