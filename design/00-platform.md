<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# The platform's standing rules

The design laws this repository owns. [DESIGN.md](../DESIGN.md) routes here;
these are the rules every higher layer inherits unless it explicitly
overrides a named one.

- **One visual decision, one owner.** Token structs and style functions are
  the only sources of faces, sizes, spacing and state styling. A host spells
  none of them.
- **Fixed property-key sets.** Every style function emits the same property
  keys in every state — values differ, keys never (Dioxus never clears a
  dropped key).
- **Typed superset grammar.** emd grows as typed `MdBlock`/`MdInline`
  variants with source ranges; the round-trip stays byte-faithful; raw HTML
  is dropped by construction.
- **Contracts name WHAT, never WHERE.** `ChromeSlot` vs `SidebarEdge`; an
  identifier that fuses them is a bug waiting for the first mirror flip.
- **MPL-2.0, plain.** Never add the Exhibit-B secondary-licences notice —
  the GPL-app-links-MPL-lib combination is load-bearing for the whole app
  ecosystem.

Per-file Exhibit A headers state the licence; `NOTICE` and the IP register
row carry the licensing facts.
