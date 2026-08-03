# Changelog

Consumers pin this library by **tag**, so a tag is the release unit. Entries
are written from the git record, not from memory.

## v0.6.0 — 2026-08-03

**Breaking:** a transcript's headings are no longer the document's, and
`ProseBody::CONVERSATION_ANSWER` is no longer a serif. A host that pinned to the
old look has to say so itself.

- **The chat surface takes t3code's type system, measured rather than felt.**
  Body at 14px/1.625 in `DM Sans` (`CHAT_SANS_STACK`), both sides of the
  conversation at ONE size; headings at weight 600 with 20px above and 8px
  below; blocks 10px apart, lists indented 20px with 4px between items. Sources:
  their `apps/web/src/index.css` `.chat-markdown` rules and
  `ChatMarkdown.tsx:1600` (`text-sm leading-relaxed`).

  The serif answer was this library's own call, asked for and then withdrawn by
  the user once the two surfaces sat side by side: *"their design language of
  the chat interface is superior and I have changed my mind."* A transcript is
  not an article — it is threaded with paths, commands and tool output, and a
  serif fights every one of them. `PROSE_SERIF_STACK` remains exported for a
  host that genuinely wants an article face.

- **`Inter Variable` is spliced into the chat stack ahead of the generics.**
  `DM Sans` is not installed on the fleet's desktop host (`fc-match "DM Sans"` →
  Noto Sans), and a face nobody chose is worse than a second choice that was.

- **Headings are no longer shared across surfaces**, and that is now the
  documented rule: a document's headings open a chapter, a transcript's label a
  paragraph inside a turn that is already a boundary. Code treatment, table
  rules and the mono face stay shared, and a test holds each half.

## v0.5.0 — 2026-08-03

**Breaking:** `ConversationTokens` loses `prose_font`, `ui_font`, `mono_font`
and `column_px`, and gains `prose: ProseTokens`. A consumer renames
`tokens.ui_font` to `tokens.prose.ui_font`; nothing else moves.

- **`yggui::prose` — the type system every rendered-markdown surface reads.**
  `emd-renderer` answers what a document IS; it has no opinion about faces, and
  so every host had to invent one. Three of them did, separately: a terminal's
  Web View, a document reader and a chat app each spelled their own heading
  scale, code face and paragraph rhythm. That is not a shared design language,
  it is three languages that happen to agree for a while — the Web View was
  drawing code in `ui-monospace` while `ConversationTokens` beside it named
  JetBrains Mono, and neither knew about the other.

  `ProseTokens` now owns every face, size and rhythm; `ProseInk` carries the
  five colours a host is expected to override, and nothing else is a host's to
  decide. `ProseTokens::document()`, `::conversation()` and `::rail()` name the
  three surfaces; the rhythm below body copy is identical across all three,
  because a heading is a heading.

- **A surface that inherits must SAY `inherit`.** `ProseBody`'s fields are
  optional and `None` means inherit, not unset. A transcript sits inside a turn
  that has already chosen the face and size — the person's ask is sans at 15px,
  the machine's answer serif at 16px/1.72 — and a markdown root that re-chooses
  silently wins over it. That shipped: answers rendered at line-height 1.55 for
  as long as the reading surface shared a `compact` flag with a 300px rail pane.
  `ProseBody::CONVERSATION_ASK` and `::CONVERSATION_ANSWER` are now the tokens
  the turns themselves apply, so every type decision in the library is in one
  file.

- Every `*_style` helper emits a FIXED property-key set and varies only values,
  `inherit` included — held by a test, because Dioxus never clears a style key a
  later render omits.

## v0.4.3 — 2026-08-03

- **The conversation type scale**, measured against t3code rather than felt: the
  answer at 16px/1.72 with a hair of negative tracking, the ask at 15px rather
  than a full step below it, the footer at 11px so a timestamp reads as a number
  and not as a label.

## v0.4.2 — 2026-08-03

- **`MdInline::Image` is a typed node**, not a `🖼` glyph plus a link. A host
  that CAN display an image (a transcript full of pasted screenshots) had no way
  to tell one from a link to one. The crate stays platform-neutral and never
  assumes a `src` is fetchable — the host decides how to draw it.
- **The work seam is a spine, not ninety-six cards.** A run of consecutive tool
  calls collects under one hairline rule rather than each row carrying its own
  boxed surface.

## v0.4.1 — 2026-08-03

- **The library builds for web and Android now, and this is what was stopping
  it.** Seven declared dependencies — `base64`, `include_dir`, `png`, `libc`,
  `tokio`, `tao`, `time` — had **no reference in any source file**, left behind
  by earlier extractions, and several of them cannot build for a wasm or
  Android target at all. They are gone. `chrome` (the only module that touches
  `dioxus::desktop`) moves behind a `desktop-shell` feature, on by default, and
  `web` / `mobile` features join it. Proven: `cargo check -p yggui
  --no-default-features --features web --target wasm32-unknown-unknown`.

  A dependency nothing imports is not free in a library apps must link — it is
  what stops an app consuming the library at all.

- **`yggui::chat_input` — the composer.** `YggChatInputBox`,
  `ChatContextOption`, `ComposerSendShortcut`, `CHAT_INPUT_CSS`. One rounded box
  with both controls inside it — context at the upper left, send at the lower
  right — so the box stays one shape at any height, plus a searchable context
  menu that opens upward because the composer sits at the foot of the page. The
  send shortcut is a PROP: Enter and Shift+Enter are exact opposites and both
  conventions are defensible, so hardcoding either makes the component unusable
  for the other half of its consumers. The hint printed beside the button names
  the key that actually sends, and a test holds those two together.

  A transcript surface without a composer is a reader. This is the other half
  of `conversation`.

- **`yggui::otp` — the six-cell login code entry.** `OtpCodeEntry`,
  `complete_otp`, `digits_for_otp`, `YGGUI_OTP_CODE_LEN`, `YGGUI_OTP_CSS`,
  `install_otp_paste_bridge_script`, `otp_paste_from_native_script`. It exists
  as a shared component because pasting a code is genuinely hard on Android —
  the long-press paste menu is suppressed on a near-invisible input and
  `navigator.clipboard.readText()` is blocked in a WebView — so the widget
  ships with a document-level paste listener and a native-bridge button rather
  than each app rediscovering that. A pasted `"Your code is 481920 — do not
  share"` yields `481920`, and a full-length paste lands at cell ONE whichever
  cell had focus.

- **`ConversationTokens::from_css_variables` + `CONVERSATION_THEME_CSS`.** An
  app offering a **System** theme has delegated the answer to
  `prefers-color-scheme`, which Rust cannot see — so `is_dark` is a question it
  genuinely cannot answer, and the design language must not require it. The
  same tokens are now addressable as CSS custom properties, with a sheet
  carrying both themes. A test asserts every referenced variable is defined in
  the light root AND both dark arms, because a variable the sheet never sets
  resolves to nothing and drops the property — an invisible hairline with no
  error anywhere.

## v0.4.0 — 2026-08-03

- **`yggui::conversation` — the agent-transcript design language, as a shared
  component set.** `ConversationColumn`, `UserTurn`, `AssistantTurn`,
  `SystemTurn`, `WorkGroup`, `WorkRow`, `DiffStat`, `ChangedFileChips`,
  `TurnDivider`, `WorkingIndicator`, `ConversationEmptyState` and
  `QuietButton`, all reading from one `ConversationTokens` derived from the
  host's own palette.

  The module owns the SHAPE of a conversation — the 720px reading column, the
  asymmetry between what a person asked (a bounded card) and what the machine
  answered (the page itself), the quiet seam consecutive tool calls collect
  into, the metadata type scale — and deliberately not the message BODY, which
  each host hands in as an `Element` from its own content pipeline. That is the
  seam that actually needs sharing: one design language over two different
  content models, with neither importing the other.

  **Platform-neutral by construction.** No `dioxus::desktop`, no `tao`, no
  filesystem, so it compiles for web, desktop and mobile alike — which is the
  point, because the first two consumers are a desktop terminal and a
  web+Android chat app.

  Two invariants are locked and mutation-proven: a work row's style emits the
  same property-key set in every state (Dioxus never clears a key a later
  render omits, and these rows are recycled by a virtual window, so a failed
  row would otherwise keep its ink on an unrelated call), and the hover
  stylesheet's custom-property names match the ones the column emits.

- **`scripts/gallery-shot.sh` + the `conversation_gallery` example.** Renders
  every component in both themes under a private headless compositor and writes
  a PNG. A component library that can only be reviewed by rebuilding a host
  application is a component library nobody reviews.

## v0.3.1 — 2026-08-02

- **`search_field_shell_style` goes box-only.** It used to emit an inline
  `background`, hairline `box-shadow` and `transition`; an inline fill
  out-specifies a host stylesheet, so the titlebar search field stayed flat and
  inert while every other field in the host learned hover, focus and a themed
  fill. The function now emits layout only, and the host skins the element
  through its field stylesheet (`data-yggui-field`). The `dark_surface`
  parameter is kept for API stability and no longer read.

## v0.3.0 — 2026-08-02

- **`emd-renderer` joins the library, under MPL-2.0.** emd (extended markdown)
  is the markdown-superset document model and parser — one typed block tree
  that every document surface renders, edits, and splices back into the SOURCE
  file, with lossless round-trip as the invariant. It lived in yggterm as a
  GPL-3.0-or-later workspace crate; it is a platform organ of the app pipeline
  (yedit and ztlkasten's document surfaces, breezed, jyas-webapp all consume
  it), and by the licence-by-role rule a library apps must LINK is MPL, not
  GPL. Relicensed MPL-2.0 at the move, on sole copyright.

  It is pure — no Dioxus, no theme, one dependency (`pulldown-cmark`) — so a
  server-side consumer such as a zettelkasten graph indexer can use it without
  a UI stack. Raw HTML is dropped by construction rather than escaped, so
  note-derived content cannot reach a host's JS context.

  The spec moved with it: [`docs/spec-emd-renderer.md`](docs/spec-emd-renderer.md)
  is now the one owner of how this engine is supposed to behave, and yggterm
  points at it rather than keeping a copy.

- **The workspace version now tracks the tag.** It read `0.1.0` while the tree
  was tagged `v0.2.0`, because nothing consumes the number — yggterm pins by
  tag. A version nobody reads is a version that lies; it is `0.3.0` here.

## v0.2.0 — 2026-08-02

- The document split gutter enters the contract, with one place that clamps the
  ratio so neither half can be dragged away entirely.

## v0.1.1 — 2026-08-02

- The side-rail stamps become named constants, so an app and its host name the
  same thing instead of agreeing by coincidence on a string literal.

## v0.1.0 — 2026-08-02

- **First release: `yggui` and `yggui-contract` extracted from yggterm and
  relicensed MPL-2.0** on sole copyright. Plain MPL, deliberately without the
  "Incompatible With Secondary Licenses" notice, so a GPL application linking
  this library — which is exactly what yggterm is — stays permitted.

- **Fixed on the way out: three dependencies `yggui` declared and never used.**
  `dioxus-desktop`, `wry` and `webkit2gtk` resolved inside yggterm only through
  its `[patch.crates-io]` redirect to a vendored `wry` fork. Standing alone they
  were fatal — upstream `dioxus-desktop` 0.7.9 wants `wry ^0.53.5` and therefore
  `webkit2gtk =2.0.1`, against the `=2.0.2` pinned here, with no resolution.
  Nobody outside yggterm could have built the library whose whole purpose is
  being linked by other people. CI now builds this workspace alone, against
  upstream, so the property cannot rot again unseen.
