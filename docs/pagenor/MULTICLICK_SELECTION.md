# Canvas multiple-click selection

With the Type tool active, click twice to select a word, three times for a composed
line, four times for a paragraph and five times for the story. Small pointer
movement keeps the selected unit. A double click with the Selection tool enters
text editing; its next click selects the line. Double-clicking a text-thread port
still breaks that link.

`text.selectLine {frame, point}` is a non-undoable selection command, backed by
Pagenor's existing table/footnote-aware `text.selectAt` implementation. The point
uses spread coordinates. The control channel and MCP accept `tripleclick` or
`triclick`; `PointerEvent::TripleClick` counts at least three presses. Existing
`down` events with a `clicks` count still work.

## Adaptation

[Upstream #347](https://github.com/storytold/designcraft/pull/347) is pinned to
`67f33cf279381d41f4c09099215d8ecff0ef6442`. Its older canvas rewrite is superseded
by this fork's existing press counter and batched-input handling (P09.7/P09.8).
Those preserve four/five-click selection and quick drawing/threading gestures.
Release-time double/triple upgrades also recover multiple complete clicks in one
paint, without repeating an already-handled press or downgrading higher counts.
The new ruler corner also recovers its press position when a complete native
press/release arrives between paints, allowing quick double-click reset.

Version remains 0.3.0. No format, dependency, branding or unsafe-code changes.
Validation covers the engine alias, explicit tool/control/MCP events, real egui
rapid word/line/paragraph/story selection and batched ruler drag/reset. Full
integration evidence is recorded in PAGENOR_UPSTREAM.md and PHASE_STATUS.md.
