# Tabs editing — P07.3

Adapted upstream #343 at `989373114be324319ff57d14596df0649a996cd0`.
Pagenor remains 0.3.0; changes are local.

Type → Tabs and Window → Type & Tables → Tabs open the floating panel; the
shortcut is Cmd/Ctrl+Shift+T. Select text or a text frame, then click the ruler to
add a stop, drag to move it or drag it off to delete it. Left/Center/Right/Align On,
position, leader and align-on character are editable. The panel menu offers Clear
All, Delete Tab, Repeat Tab and Reset Indents. The magnet positions it above a
selected horizontal text column at the current zoom. Drags commit on release.

Paragraph Style Options → Tabs uses the same ruler and controls, applying its
changes on OK. The command API is `type.tabs.get/add/move/change/remove/clear/repeat`,
with optional `style` for paragraph-style targets. Otherwise it edits selected
paragraphs, using the first selected paragraph's stops for multiple paragraphs.
Each command is one undo step. Repeat uses the previous stop or left indent and
stops at the column width less right indent.

Positions are finite, clamped to 0–15552 pt; lists have at most 100 stops, sorted
with one stop per position. Leaders allow eight printable Unicode scalar characters;
align-on allows one. Explicit malformed style/string/width values are refused.
All paragraph/style command writers validate lists. Native document and IDML
loading repair invalid/oversized lists to these limits, preserving valid lists.
The existing native document format version, safe saves, IME, document font scopes,
New Document window and font-size preset controls are retained. Italian is deferred;
Ukrainian updates are skipped because this fork has no configured Ukrainian locale.

Tests cover command edits/undo, repeat, right-tab composed alignment, style inheritance,
malformed/capped lists, stored return values, native/IDML load repair, the keyboard
shortcut, ruler clicks and the Paragraph Style Options ruler. Actual egui panel
and style-window renders were inspected. Vertical tab rulers and Shift-dragging
only the left indent remain outside this port, as upstream documents. Native
Windows/Linux execution remains a milestone check.

All seven local gates pass: 1,524 tests, 3 ignored; 20 WASM packages.
