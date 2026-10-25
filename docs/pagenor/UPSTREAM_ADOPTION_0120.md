# P11.22 — queued editor and master workflows / local Pagenor 0.12.0

Review snapshot: 25 October 2026 (Asia/Kolkata). Base: local 0.11.0,
`2aed502fbad36f1f878def2ba04a273924d6cc77`.
This continues the reviewed queue; it is not a new complete upstream audit.
Five proposals are adapted, including two older closed proposals and three queued
in the 0.11.0 review. All five immutable heads were rechecked unchanged.

## Visible changes

- Layers supports inline object renaming after a slow second click. A quick double
  click selects; Enter/focus loss commits and Escape cancels. Grouped listings,
  selection squares, visibility/locking and layer/object drag behavior remain.
  The editor belongs to the active document and uses the existing undoable
  `object.rename` command, including Unicode names.
- The status bar and Properties page selector list document pages and masters.
  Facing masters have explicit Left Master and Right Master entries, ordered by
  physical side in both binding directions. Stable page/master IDs protect a menu
  against reordering; a menu from another document is refused. The enlarged status
  bar, separate master view and Back to document action are retained.
- Layout → Pages → Insert Pages and the Pages context menus open a dedicated
  count/placement/master dialog. It supports before, after, start and end insertion;
  master choices use IDs. Direct command callers retain chosen document defaults
  and past-end append behavior. Invalid/ambiguous masters and malformed placement
  requests fail before editing; a stale dialog cannot insert into another document.
- A master context menu can delete that master with one undo step. Local overridden
  items survive as ordinary objects; source items leave text threads, inheritance
  and override references are cleaned up, and a deleted default is cleared. Other
  document views remain intact. Deleting the last master returns to document mode.
- Paragraph border/shading sides and corners have link controls. Linked edits set
  all four values; unlinked edits change one, and relinking copies Top/Top Left.
  Corner sizes and shapes link together. The link choice is transient dialog state;
  existing paragraph/style preview and transactional confirmation are retained.

## Compatibility and validation

Preserve `unsafe_code = "forbid"`, Rust layering, Pagenor branding/owner rights,
`.pgnr` and legacy loading, distinct app/document icons, lazy/live/scoped fonts,
inline Indic input, EOF/final-LF/paragraph defaults, explicit book masters,
Welcome/recents, grouped vertical/dual-column panels and Links color/ICC data.
No private documents, fonts or mapping data, new assets or translations are added.
Vectnor and its chat are untouched. No publication or owner-session restart occurs.

The application advances 0.11.0 → 0.12.0 for this feature group. Native document
capability stays **6** and external dependency pins remain unchanged. Published
preview artifacts remain 0.7.0. Native Windows/Linux execution and packaging are
pending; shared Rust/WASM checks do not substitute for those builds.

All seven workspace gates pass: 2,907 tests, zero failures, 7 existing skips; twenty WASM packages. 8 synthetic offscreen views are inspected.

A separate
optimized local Mac GUI/CLI bundle and artifact-side report follow the source commit.

## Source pins and adaptation decisions

Source remains MIT OR Apache-2.0; upstream notices and authorship are preserved.

| PR | Author | Reviewed head | Adaptation |
|---|---|---|---|
| [#1083](https://github.com/storytold/designcraft/pull/1083) | @moppeeer | `5aed65ff238109960cc0b583d2e54f8b1529b338` | Slow second-click behavior ported to Pagenor’s grouped listing and existing `object.rename`; retain drag/drop and document-scoped editor state. |
| [#1136](https://github.com/storytold/designcraft/pull/1136) | @harrisonoconnorhover | `f78bf721e86d2357180c2578fc76ad400906a526` | Master deletion and reference cleanup; additionally clear the deleted chosen default and preserve other document views. |
| [#1505](https://github.com/storytold/designcraft/pull/1505) | @moppeeer | `7a68cc6e6e88eccd77a08f585726b60d21d8ac7d` | Shared navigation extended with explicit left/right master entries and stable page IDs; retain enlarged status bar and master view. |
| [#1508](https://github.com/storytold/designcraft/pull/1508) | @moppeeer | `946dd60733c52db0cb73ce85e1895b6ac848b9ba` | Dedicated insertion dialog and before/start backend; retain chosen master defaults and append semantics, validate references, and guard dialog document ownership. |
| [#1521](https://github.com/storytold/designcraft/pull/1521) | @moppeeer | `3085c7a8fdb70cf61cc1121d9860e2420da8ae73` | Link border/shading sides and corner size/shape through existing transactional paragraph/style dialogs; no translation catalogues imported. |

## Remaining queue

Six scopes from the 0.11.0 review remain pending: #1444 background link checks;
#1496/#1497/#1498 formatting Find/Change; #1531 Links redesign; #1537 date/EPUB
refactor. Earlier deferred rendering/import work also remains pending unless
explicitly closed by an adoption note. This checkpoint does not claim all upstream
PRs are implemented, nor does it change the excluded CJK/Italian UI scope.
