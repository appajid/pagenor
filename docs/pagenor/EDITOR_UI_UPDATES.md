# Editor UI updates — P09.6

Ten reviewed UI proposals were adapted locally, one feature at a time, after the
owner approved their supporting backends. Italian translation #218 is deferred.
No remote push is part of this task.

## Behavior

- PDF Export now opens options when invoked without parameters: preset, page
  range, spreads, image compression, bleed, marks, PDF standard, tagging and
  flattener. Successful exports remember options; pages/title are per job. Explicit
  parameters still export directly. Cancelled saving and failed export/writes keep
  the dialog and its values. Invalid mark measures are rejected, settings are
  parsed before any write, and a switched document cannot be exported accidentally.
  Browser download and desktop file delivery use the same exporter.
- Swatch chips and names share a clickable row in Properties, shared pickers and
  contextual task-bar menus. Fill/stroke chip double-click opens the existing Color
  Picker; single click still opens swatches. Confirming a colour remains undoable.
- Canvas right-click offers text, object or view actions. Selected text takes
  priority over its containing frame. Actions use the existing command/clipboard
  bridge and show unavailable commands disabled; failures retain normal status
  reporting. Right-click itself preserves selection and does not feed drawing tools.
- Paragraph Style Options has Paragraph Rules. Above/below rules edit weight,
  swatch or Text Color, tint, column/text width, offset and left/right indents. Edits
  retain the complete inherited/current rule, and scrolling reaches both rules.
  The upstream patch's snake_case names and unsupported `width`/`overprint` JSON
  members were replaced with supported Pagenor fields. There is no new per-rule
  overprint control; Pagenor's existing Rule model has no such attribute.
- The control-bar panel is always created with a stable ID; hidden state takes
  zero height and paints no frame. Existing two-row/scrolling controls remain.
  Interactive for PDF shows it and Buttons and Forms; saved custom workspaces
  keep their own stored settings.
- Cmd means the portable primary accelerator. Ctrl means that accelerator on
  Windows/Linux and physical Control on macOS. Extra Mac Control no longer matches
  an unrelated bare or Cmd-only binding. Conflict checks compare normalized
  bindings. Recording preserves Mac Control, waits through modifier-only/held-key
  events and consumes recorded Enter/Escape; Escape cancels recording only.
- Table > Distribute Rows Evenly and the Table panel share `table.distributeRows`.
  Selected rows get equal exact heights with the same composed total. Cell text,
  column spans, headers and selection survive one undo/redo. Repeated headers in
  threaded frames count once. Missing composed rows, row-spanning cells and bad
  indices fail before editing. Exact heights may overset cell text, as the panel
  tooltip states. Composed cells are inspected once rather than once per row.

## Validation

Each proposal has a separate focused local test result. New regressions cover
native/browser PDF delivery and error/cancel/UID preservation, chip clicks and
colour confirmation/undo, right-click duplication and Unicode paste/undo,
inherited rule editing/undo, first-frame bar geometry transitions, custom workspace
restoration, platform modifier semantics/dispatch/recording and table height,
content, selection, merge/overset refusal and repeated headers.

All seven final local gates pass: formatting, checks, strict Clippy, tests,
asset licences, architecture layers and WASM. There are **1,308 passed, zero failed,
3 existing ignored tests** (22 new regressions); all **20 WASM packages** compile.
The full-window fill/stroke regression also covers a recent unrelated click that
makes egui classify the next double-click as a triple-click.

Inspected CPU renders of the actual UI cover PDF preset/crop-mark pointer changes,
clickable swatch rows, the Color Picker, canvas context actions, both rule sections
and scrolling, Interactive PDF controls, and table-row distribution before/after
an actual panel-button click and undo. They use synthetic documents, not native
window screenshots. The table pointer check changes only rows 1/2 from 40/80 to
60/60; undo restores their heights. PDF tests write/read actual export bytes.
Native Windows/Linux and browser execution remain milestone checks. The running
macOS app bundle and user documents/recovery data are not replaced or modified.

Pinned source heads and authors are in PAGENOR_UPSTREAM.md. This completes this
selected UI group, not the broader Phase 09 exit criteria.
