# Installed-font discovery and refresh

Updated: 2026-10-08. Tasks P01.1 / P01.2 / P01.3 / P01.4.

Open **Type › Font Discovery…** to see the latest installed catalogue, scanned
font folders, host registry file count, rejected files and duplicate-face
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

Both return roots, hostSource, hostRegisteredFiles, hostPathlessFaces, filesScanned, families,
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
  A retry with unique family/PostScript names and repaired checksums produced
  the same host log. This does not verify Font Book activation/deactivation. A direct Font Book
  and third-party font-manager session remains required (PG-0102).
- Descriptors with no readable file path cannot be fed to the existing Rust
  shaper; their count is visible in diagnostics. This includes fonts that macOS
  may expose without accessible font bytes.
- Loaded installed faces stay cached to preserve existing rendering. P01.3 makes
  installed picker families/styles follow the current catalogue, so removed
  cached faces no longer restore deleted entries. Document scopes remain
  selectable and keep their existing handles. Host deactivation that leaves a
  readable file in a scanned directory still needs a separate policy/test.
- Windows and Linux/FreeBSD adapters are implemented as described below. Native
  CI and user font-manager/installation smoke tests remain PG-0103.

## Native location adapters (P01.2)

`with_system_fonts` combines standard platform folders with host discovery.
`with_font_dirs` remains isolated: it never consults host registries or configuration.
The new sources provide paths to the existing bounded SFNT metadata scanner;
family/style names and named instances continue to come from the font itself.
Canonical paths and existing family/style precedence suppress duplicates.

Windows reads the machine and current-user `CurrentVersion\Fonts` registry keys
through the safe `winreg` API, in both 32-bit and 64-bit views. Absolute paths can
point outside standard folders. Relative paths resolve against Windows Fonts or
the current user's local font folder. String decoding preserves UTF-16 paths;
REG_EXPAND_SZ expansion is bounded and performed once. Invalid value types,
malformed paths and unavailable keys are diagnostic records. Discovery never
writes installed-font keys. Tests write only a disposable, separate HKCU test key.
Fonts registered privately inside another process and memory-only GDI fonts are
not exposed by these installed-font keys; this adapter does not promise access
to them. Fonts passed to Pagenor's own `add_font` API already reach the picker.

Linux and FreeBSD load Fontconfig at runtime through the safe `fontconfig` API
and request file locations only. Missing Fontconfig is reported, while folder
scanning remains available. Fresh configuration reads follow FONTCONFIG_FILE /
FONTCONFIG_PATH, ordered numeric *.conf includes, relative/tilde/XDG prefixes,
remapped directories and reset-dirs. Cycles and hostile input are bounded: 256
configuration paths, 1 MiB per XML file, 8 MiB total, 4,096 entries/directories and
16,384 pending elements. XDG_DATA_HOME and absolute XDG_DATA_DIRS locations are
also included. New files and configuration directories are found on refresh,
even if Fontconfig's process-wide cache has not refreshed. This is a location
adapter; it does not reproduce Fontconfig aliases, matching or reject rules,
and standard font folders remain scanned.

Diagnostics include the source name (Core Text, Windows font registry or
Fontconfig). Paths with non-Unicode OS bytes remain intact in the catalogue;
only diagnostic presentation uses replacement characters, avoiding a JSON
serialization panic.

Additional validation:

- Full local `cargo xtask ci` passed all seven steps: 795 tests passed, none
  failed, three performance tests ignored; all 16 WASM packages checked.
- Windows and Linux font crate/tests compile for their actual Rust targets and
  pass Clippy with warnings denied from macOS. This is compile-time evidence;
  native execution remains required.
- Isolated tests cover Unicode/environment paths, expansion limits, registry
  type errors and external collection paths; XML include ordering, reset/cycles,
  XDG resolution and oversized/malformed configuration.
- Native tests compare all readable registered faces with the lazy catalogue.
  Linux/FreeBSD additionally compare against an independent `fc-list` process
  and run a fresh subprocess with external fonts and a changed configuration.
- The earlier CI rerun passed macOS, Linux, minimum Rust, packaging lint and
  FreeBSD. Windows exposed an unrelated path-text assertion; it now compares
  paths and retains its save/reopen validation. New adapters await native CI.

API references: [Fontconfig configuration](https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-user.html),
[Microsoft font installation](https://learn.microsoft.com/en-us/windows/win32/gdi/font-installation-and-deletion),
[`fontconfig` safe wrapper](https://docs.rs/fontconfig/0.11.0/fontconfig/),
[`winreg` safe wrapper](https://docs.rs/winreg/0.56.0/winreg/).

## Removed loaded fonts and menu availability (P01.3)

Installed family/style picker entries now come from the current lazy catalogue.
Loaded installed faces remain cached for resolution and rendering, but no longer
add stale families or styles to these menus. Bundled and in-memory fonts remain
selectable; document-font scopes retain their own entries and take precedence.
`has_family` continues to describe usable cached/scoped/installed faces, so a
removed but loaded face may still return true during the current process even
though it is absent from the shared picker. This preserves the existing renderer
and fallback behaviour rather than evicting font bytes used by layouts.

A regression failed before the fix when removed Bold remained in the style list.
It now covers partial-style removal, complete family removal from both menu APIs,
unchanged cached/document Arc handles, scoped styles, restored files and retained
bundled/memory entries. The complete local gate passed: 796 tests, zero failures,
three performance tests ignored, all seven steps and all 16 WASM packages.

Font Book validated the uniquely named QA font: one file passed with no problems.
The owner installed it as `/Library/Fonts/pagenor.ttf`; Font Book lists Pagenor
Test1. A native Rust UI probe confirmed `font_menu` contains Pagenor Test1 /
Regular and resolves it from that installed path (glyph A = 2). The inspected
diagnostics dialog reports 799 installed families, 3,773 faces and 945 files;
these counts are host-specific. The owner requested that it remain active. No activation setting was
changed by this test. Font Book’s Active filter showed the test family.
A separate Core Text family-list/matching probe did not expose it on this host;
Pagenor discovered its readable installed file through directory scanning. This
confirms installed-file/menu coverage, not external session registration. Direct
activation/deactivation remains unverified.

P01.2 remote CI passed macOS, Linux (including native Fontconfig comparisons),
minimum Rust and packaging lint. Windows and FreeBSD are still running at the
time of this update; the issue register records their final evidence. Windows
ARM64 packaging/run smoke passed in run 37776034034: cross-build MSI, native
install, executable architecture check, CLI version invocation and uninstall.
This does not validate ARM64 GUI rendering or font activation.

## Desktop availability and newly discovered styles (P01.4)

The catalogue is an index of local family/style names and font file paths, not
an internet font service. Native discovery reads host registrations and physical
font files. WASM alone has no installed-system-font access. Keeping full glyph
bytes lazy does not restrict the desktop family's menu to bundled/web fonts.

macOS now unions the available-font collection with any additional registered
PostScript names, resolving their descriptors through safe Core Text APIs.
Neither query filters names beginning with a dot. Both queries returned the same
516 names / 207 files on this host, so the complementary query adds no files in
this particular session; no improved Font Book activation coverage is claimed.
The owner’s Pagenor Test1 remains active. Registry-supplied direct paths need no
TTF/OTF extension; folder scans include hidden files/subfolders with supported
extensions. A synthetic hidden family, hidden paths and an extensionless direct
file test verifies menus, lazy loading, deduplication and glyph coverage.

A newly discovered Bold or named variable instance previously resolved as an
already-loaded Regular because nearest-style lookup happened before catalogue
loading. Exact loaded styles now retain precedence; if an exact style is present
only in the catalogue, its file is loaded before choosing a closest style.
Missing styles retain nearest-style fallback. Variable-axis/custom-instance
lookups follow the same rule. Two regressions failed before the fix and pass
now; they also verify preserved document/memory handles and normalized style
names. Font bytes remain lazy for menus; no document or OS activation is changed.

Full local `cargo xtask ci`: all seven steps passed, 799 tests passed, zero
failed, three performance tests ignored and all 16 WASM packages checked. Local
font-crate Clippy checks with warnings denied also passed for Windows GNU and
Linux GNU targets; this is compile-time evidence, not native execution.

Remaining limits still apply to descriptors without readable font data and to
external font-manager lifecycle coverage. New tasks are tested locally; GitHub
uploads and native Windows/Linux/macOS validation are deferred to a completed
roadmap milestone, per the owner's request.

Reference: [Apple's available PostScript-name API](https://developer.apple.com/documentation/coretext/ctfontmanagercopyavailablepostscriptnames()).
