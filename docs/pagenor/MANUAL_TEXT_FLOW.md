# Manual text flow — P09.8

Verified locally on macOS, 2026-10-10. Pagenor remains **0.3.0**.

The canvas lost quick clicks because egui clears `press_origin` when a press and
release arrive in one paint frame. A complete quick drag also has no `dragged`
flag. This affected Type-tool frame creation, caret placement and text-port
clicks. The shared frontend now recovers the press position from the input batch
and delivers the final drag position before release. The tools, command engine,
composition, document format and unsafe-code policy remain unchanged.

Four UI regressions pass: ordinary frame drawing, a complete drag delivered in
one frame, quick caret placement/paste, and quick out-port linking to an existing
frame on page two with text preservation, composed continuation and undo.
All seven local quality gates pass: **1,447 tests passed, 3 existing tests ignored,
20 WASM packages checked**. Native Windows/Linux interaction remains unverified.
An existing WASM-only unused-variable warning in `panels/library.rs` remains.
The macOS development bundle builds and passes plist and ad-hoc signature checks.
Build metadata: `uncommitted-P09.8`. Changes stay local; no GitHub release update.

## Actual desktop verification

Using the native Pagenor UI, created a two-page document with Facing Pages and
Primary Text Frame disabled. Drew a text frame using the Type tool and pasted ten
paragraphs. Selected it with Escape, clicked the red + in its out port, navigated
with Next Spread and clicked inside the second page. Pagenor created the linked
continuation frame, showed the remainder through paragraph 10, and changed the
preflight indicator from one overset error to No errors. Saved the result through
the macOS Save panel as `Manual text flow.pgnr`. The fixed app is left open on it.
The test used the rebuilt `target/macos-dev/Pagenor.app`; the older release-audit
copy remains separate.

## Manual steps

1. Draw a frame with the Type tool and type or paste text.
2. Press Escape to select the frame with the Selection tool.
3. Click the small out-port square on its right edge, just above the bottom-right
   corner. A red + means text is overflowing.
4. Navigate to the next page with the bottom Next Spread button or the Pages panel.
5. Click inside an existing empty text frame, or click blank page space to create
   a continuation frame. You can also drag to choose the new frame's size.

Repeat steps 2–5 if more text remains. The out port is separate from the corner
resize handle. No Smart Text Reflow preference is needed for manual linking.
