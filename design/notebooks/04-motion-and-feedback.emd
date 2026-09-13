<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Motion & feedback

**edition 2026-09-07 · rev 1**

Motion is functional, not decorative. The workspace should feel tighter and
more exact after a transition, not more playful. State changes are crisp;
nothing rubbers, bounces, or lags.

Read a transition as a sequence: ready → working → result, or working →
recoverable error. The same job occupies the same place throughout. Progress
must come from measured work; if the total is unknown, say “Working” without
inventing a percentage. A stable label is more useful than a lively spinner
that tells the user nothing.

## Durations and curves

- Desktop-fast durations with Material-3-style curves: **emphasized
  decelerate** when something enters or is revealed, **emphasized
  accelerate** when it exits, the **standard curve** for small state shifts.
- Hide/show motion must read as a *structure change*, not a flourish.
- Decoration must never move text: a live indicator changes **colour only**.
  (A streaming answer's rule once also changed its padding, and every answer
  slid sideways the moment it finished, a jump under the reader's eye, on
  every turn. Never again.)

## Toasts

- In-app toasts sit **horizontally centred near the top** by default, over a
  terminal, whose newest output lives at the bottom.
- Over a **document** the top of the viewport is the title being read, so the
  stack moves to the **bottom corner on the rail's edge** (directional chrome
  follows the mirror).
- ⭐ **The anchor belongs to the viewport, not to the toast.** Whatever the
  placement, every arm emits the same style keys; a bottom anchor reverses
  the stack so the newest toast stays nearest its edge.
- Tones are clear; stacks animate upward as items leave; job notifications
  **coalesce by task identity** instead of stacking duplicate progress cards.
- Long-running work is never silent: visible progress, or it did not happen.
- Offer Undo for reversible actions. Irreversible actions require appropriate
  confirmation; a toast cannot recover something the system did not retain.

## The stage-curtain loading rule

Loads look like a stage production: the audience never sees the mess. A
loading viewport may show, in order of preference:

1. the correct final frame immediately,
2. the previous faithful frame held perfectly still (a ghost),
3. a flat background-coloured veil.

Nothing else. No DOM leaks, no partial rows, no stale frames that later
"correct", no blink between a covering layer and the final frame. The curtain
comes down **before** teardown churn starts and lifts only when the final
frame is fully painted underneath. Latency is preferred over flicker.

## The shared blink clock

Every blinking indicator, live dots, loading tabs, working marks, flips on
the SAME tick: one timer, half of each **2400 ms** cycle, one stylesheet rule
that stamps every marked indicator invisible. Indicators join by carrying the
marker in their style; they never declare an animation of their own.

- The period is a **verdict, not a default**: a faster wave was measured as a
  strobe and slowed to 2400 ms on sight.
- ⛔ The clock may not be a CSS animation of a custom property: the style
  system can advance such an animation without repainting its consumers, so
  every dot freezes. Whatever drives the phase must be a change the paint
  path cannot ignore.
- A blink is proved with a **burst of screenshots**, never with a computed
  style read, the probe that reads computed style forces the very
  recalculation whose absence is the bug, so it can only answer "yes".

## Auto-hide chrome

- The revealed auto-hide titlebar floats on a soft drop shadow **alone**,
  never a hairline (a bright separator line is what a 1px border paints, and
  it shows worst over the lighter sidebar).
- Auto-hide reveal is **chrome, not layout**: it draws over the workspace
  with the same tint/gradient language and never resizes or shifts content.
- The revealed sidebar is a floating island: inset card, four-side margin,
  rounded corners matching the viewport radius, ambient shadow on every side.
  Docked stays the seamless in-flow panel it always was.

## Proving motion

Test reduced-motion preferences as well as the default. Reduced motion should
preserve status and focus without requiring blinking or movement to understand
the result. A steady working label is still informative.

Motion is proved in time, not in a single frame: burst screenshots across the
transition (enter, settle, exit), checked for the three forbidden things,
text that moved, frames that flicker, states that linger. If the transition
cannot survive a burst, it is decoration pretending to be function.
