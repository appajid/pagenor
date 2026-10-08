# Installed-font discovery and refresh

Updated: 2026-10-08. Task P01.1.

Open **Type › Font Discovery…** to see the latest installed catalogue, scanned
font folders, macOS registry file count, rejected files and duplicate-face
choices. **Rescan Installed Fonts** is available there and in the Type menu.
Neither command needs a document. Counts describe installed fonts, excluding
bundled, in-memory and document-local fonts.

On native macOS, Windows and Linux, the running UI checks discovery locations
and file size/modification metadata on a worker at most every three seconds.
Changed candidates rebuild the existing lazy catalogue. On macOS each check
also queries Core Text for currently available file-backed font descriptors,
including activation locations outside standard folders. New families can
appear without restarting the application or using Rescan. UI repaint timing
and a busy worker can extend the delay. WASM has no installed-font discovery.

The scan still reads only bounded SFNT metadata tables, including named variable
instances; full font data is loaded when a face is resolved. TTF, OTF, TTC and
OTC, family/style grouping, duplicate precedence and document font scopes use
the existing implementation. Unchanged automatic checks do not read font tables.
Diagnostic records stop at 256 while counters include omitted records. A file
with no readable family metadata reports an unsupported/damaged/limit reason;
this metadata scan does not validate every outline or shaping table.

Rescans serialize with background refreshes, replace the catalogue together,
clear previous glyph/chain misses and increment a generation counter. The UI
invalidates composition and canvas caches when that generation changes, without
editing documents or adding undo/history entries. Fonts supplied via
`FontDb::add_font` also update the generation. Document-local faces and their
source paths remain unchanged and continue to take precedence within their scope.

Commands are available through CLI, MCP and the control channel:

```json
{"id":1,"method":"engine.execute","params":{"command":"font.diagnostics","params":{}}}
{"id":2,"method":"engine.execute","params":{"command":"font.rescan","params":{}}}
```

Both return roots, hostRegisteredFiles, hostPathlessFaces, filesScanned, families,
faces, duplicateFaces, skippedFaces, rejectedFiles, unavailablePaths,
issuesOmitted and issues (path/reason). `app.fontDiscovery` opens the UI dialog.

Validation:

- A failing regression first reproduced a glyph that stayed missing after
  installing a covering font and rescanning; it passes after miss invalidation.
- Synthetic tests cover automatic candidate changes, unused-font removal,
  unchanged checks, in-memory fonts, bounded malformed-file diagnostics,
  duplicate records, concurrent rescans and unchanged document-local face handles.
- The host test compares every readable Core Text face/style with the lazy
  catalogue. The inspected UI reported 798 installed families, 3,772 faces,
  944 files and 52 duplicate faces on this Mac; these are host-specific counts.
- Full local `cargo xtask ci`: all seven steps passed, 792 tests passed,
  no failures, three performance tests ignored; all 16 WASM packages checked.
- The real egui dialog was rendered and visually inspected with the scratch CPU
  renderer because a headless GPU adapter is unavailable on this host.

Remaining Phase 01 evidence:

- A controlled native Core Text registration probe logged
  `NOT IMPLEMENTED - RegisterFonts`; its test font never became registered.
  This does not verify Font Book activation/deactivation. A direct Font Book
  and third-party font-manager session remains required (PG-0102).
- Descriptors with no readable file path cannot be fed to the existing Rust
  shaper; their count is visible in diagnostics. This includes fonts that macOS
  may expose without accessible font bytes.
- Already-loaded installed faces stay cached to preserve existing rendering;
  unused removed files leave the catalogue. Define picker behaviour for loaded
  deactivated faces before hiding or evicting them (PG-0104).
- Windows registry and Linux Fontconfig/XDG activation locations still need
  safe native adapters and native comparison tests (PG-0103). Standard font
  directories and automatic file-change discovery are already shared.
