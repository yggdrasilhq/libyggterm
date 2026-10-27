<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# The 6-scenario comparison — plain fixture vs emd host (runbook)

The deciding experiment named by the ydesign verdict (chain 0029, 2026-10-27):
**the executable SCENARIO is the primitive** — implementation + fixture +
interaction sequence + expected observations. The question is whether the
**emd arm** (em component pages hosted by `emd-page-host`) earns its place
over the **plain arm** (an ordinary fixture app mounted through the runner
command contract, `fixture-host::plain`), measured on:

1. **time-to-accepted-state** — a fresh seat's cost from "I know the
   component case" to "the falsify bundle is accepted",
2. **reset reliability** — the declared fixture reproducing across
   fresh-reset rounds (two clean passes close acceptance),
3. **defect detection** — declared plants detected through the ONE
   comparison contract,
4. **maintenance** — the diff-size cost of a component change reaching
   both the fixture and the expected observations.

The contract (`fixture-host`, one manifest model, one `Mounted`
abstraction, one judge, one bundle schema) exists precisely so this
experiment runs against ONE contract instead of two. **Everything below
is UNVERIFIED until driven on real fixture apps by the reviewer** — the
scaffold and both CLIs are the deliverable, not the numbers.

## The six scenarios

Each scenario is defined by fixture identity, interaction sequence, and
expected observations; the interaction sequence REPLAYS identically
through both arms.

| # | Scenario | Fixture (synthetic t3section shapes) | Interaction | Expected observation |
|---|----------|--------------------------------------|-------------|----------------------|
| S1 | Long content | A badge section where the label + detail text exceeds one row | render at the row height | scene digest stable; the detail wraps, geometry handles 2× content |
| S2 | Keyboard focus | Buttons section, 6 buttons | Tab/Shift-Tab across all + Enter on one | focus ring digest changes exactly once per focus move |
| S3 | Scrolling | List section with 40 rows | PgDn 3×, Home | row-block digests unchanged; the scroll offset is the only delta |
| S4 | Async failure | An input section whose load simulates absence | trigger the failing path twice | the same actionable error card twice — never a partial state |
| S5 | Resize | Grid section at 2 columns | grow height/width | reflow produces an equivalent scene digest through the declared values |
| S6 | Theme override | The same section under a second pinned
 profile | re-render under profile switch | ONLY the token digests change; scene structure digests identical |

## How the arms run the same scenario

- **emd arm**: `emd-page check|render|run|selftest <page.emd>` — a page
  that mounts the shapes; the scenario interaction is replayed through
  the page's declared fixture surface (`run --rounds 2`). Interactions
  beyond fixture_set (focus/scroll/resize) need the emd-ui harness — the
  honest verdict for S2/S3/S5 on the emd arm is UNVERIFIED until that
  surface adapter exists.
- **plain arm**: `fixture-scenario check|run|selftest <scenario.dir>` —
  the launcher speaks capabilities|reset_fixture|observe. The SAME
  event vocabulary carries interactions: the runner contract's `tap`,
  `type`, `swipe` (shared with the mobile/web/TUI adapters) map directly;
  the synthetic corpus app implements observe-only verbs, so S2/S3/S5
  interactions likewise need the interaction adapter — with the SAME UNVERIFIED semantics.

**Scaffold status (this repo, 2026-10-27):** S1/S4/S6 are RUNNABLE on
both arms through the declared-fixture path; S2/S3/S5 are declared, both
arms report UNVERIFIED rather than guessing (the contract's honest
failure semantics, deliberately — a scaffold silence would BE the
vanity).

## The measurement protocol

1. **ref-vs-ref CLEAN first** (determinism gate) on the SAME launcher.
2. **Two fresh-reset rounds must agree** — the same-run acceptance at
   scenario level.
3. **Declare plants before inspection**; every declared plant must be
   DETECTED through the common contract (`MISMATCH` with `axis=`-named
   findings).
4. Time/maintenance numbers are recorded by the reviewer from a FRESH
   seat performing the task, not by the scaffold itself.

Record outcomes as ux-falsify bundles under the env-exported evidence
root (`UX_FALSIFY_EVIDENCE`) — hashed retention, replay recipes inside.
