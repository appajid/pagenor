# Editor usability — Pagenor 0.5.1

Owner request, 2026-10-11: visible canvas scrollbars; Arrow selection of tables
and cell border properties; functional panel groups, removable panels and vertical
stacking; a slightly taller page-navigation/error status bar.

## Behaviour

- Both canvas scrollbar tracks are 16 logical pixels wide, with persistent,
  higher-contrast thumbs. Drag a thumb or click the track to navigate a zoomed
  page; wheel/trackpad, Hand/Alt-pan and split-view behaviour remain.
- With the Arrow tool, click a cell, drag across cells, or Shift-click to extend
  the rectangular selection. The Table panel's Table/Row/Column controls select
  those ranges. Cmd-click on Mac (Ctrl on Windows/Linux) selects the enclosing
  frame; double-click still enters text editing.
- In Table > Cell Borders, click proxy lines to choose outer sides or interior
  horizontal/vertical rules. Weight, colour, tint, stroke type and gap colour
  affect only those chosen edges. Cell Fill applies to selected cells. The Stroke
  panel and Cell Options retain the same engine implementation and undo history.
- Window > Panels groups Document, Typography & Tables, Color & Appearance,
  Objects & Layout, Publishing, Interactive and Utilities. The right icon rail
  uses the same groups, active indicators, and scrolls to all panels. The AI
  Assistant is part of Utilities and uses the same dock, retaining its existing
  independent floating-window preference.
- Newly opened panels use vertical stack entries with collapsible headings.
  Visible × controls remove panels; panel menu/icon choices toggle visibility.
  Reopening restores the saved placement; newly opened panels are scrolled into
  view. Window > Panels > Arrange Vertically
  converts existing docked arrangements to a functional stack and retains floating
  windows. Long stacks have a separate vertical scrollbar.
- The bottom bar is 28 logical pixels high (previously 17), with 22-pixel page
  controls and 12-point zoom/error labels.

## Implementation and compatibility

Reuse the pinned safe craft-ui docking model; no shared dependency rewrite or
unsafe code. UI commands remain in UI_COMMANDS; tables emit table.select and
existing table.setCell engine commands. A validated optional frame ID keeps
selection and caret geometry attached to the correct table story. Workspace
serialization and closed-panel return locations remain compatible. No document
format or asset changes. Shared Rust UI/tool code serves all platforms.

The local ignored plan/ instructions referenced by AGENTS.md were unavailable in
this checkout; AGENTS.md, CLAUDE.md, Cargo workspace, source/tests, tracked roadmap
and PAGENOR_UPSTREAM.md were reviewed before implementation.

## Validation

All seven `cargo xtask ci` steps pass: fmt, check, Clippy with warnings denied,
1,863 tests passed / 6 ignored / no failures, asset attribution, layering and all
20 WASM packages. The 12 packaging contracts pass. The ignored cases retain the
existing Linux private-D-Bus and manual timing/performance prerequisites.

Regression coverage includes transformed-frame Arrow cell selection, actual canvas
cell dragging with frame retention and undoable horizontal/vertical colour/weight
edits, invalid-frame atomic rejection, visible larger navigation controls, panel
close/reopen placement, complete functional registry coverage, long-stack scrolling,
new-panel reveal, and the AI Assistant's shared dock / retained floating behaviour.

Actual egui CPU renders show populated table cells and the new border controls,
visible canvas scrollbars at 200%, the taller bottom bar and vertical panel stacks.
Physical Windows/Linux runtime validation remains milestone work.

## Native Mac inspection and local installation

The final arm64 development app and CLI report 0.5.1. The bundle passes plist
validation and strict/deep ad-hoc signature verification. In the native app,
File > Open loaded the synthetic four-row table document. Arrow dragging selected
the first three rows; the Table panel changed their rule colour to magenta and
weight to 4 pt. The fourth row retained its original rules, and the document was
saved successfully. The larger bottom bar, visible canvas scrollbar tracks,
vertical Table/AI/Properties stack, automatic reveal of newly opened panels,
scrolling through the Table controls, and visible close controls were inspected.
Closing Properties and AI removed them from the dock.

The verified app was installed at /Applications/Pagenor.app on 2026-10-11 after
closing the test build. The previous 0.5.0 app and original bundle Contents were
backed up under the task's work/editor-usability directory. Installed executable
bytes match the tested build, and both owner-supplied app/document icon resources
match the build and remain distinct. The local 0.5.1 checkpoint includes an
installation receipt, source hashes, rendered UI images, successful check logs,
the synthetic document, app and CLI. No GitHub publication was performed.
