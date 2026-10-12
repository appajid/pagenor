# Experimental PageMaker import — P11.10

Local Pagenor 0.5.2 addition, 2026-10-11. This is an initial text-recovery
compatibility importer, not a complete PageMaker reader or a fidelity milestone.

## Opening files

Desktop File → Open accepts PageMaker 6.5/7 document and template extensions
`.p65`, `.t65`, `.pmd`, `.pmt` and `.pm7`. Dropped documents use the same engine import.
The command is `file.open {path}`. `file.openBytes {name, base64}` also accepts
these files on desktop when `name` includes the extension. The original file
is read, never modified; the imported document starts without a save path.
Save it separately as `.pgnr`.

The optional external `pmd2raw` executable from libpagemaker must be installed.
It is already installed on the owner's test Mac (version 0.0.4). The application
checks the `PAGENOR_PAGEMAKER_CONVERTER` environment override first, then the
normal Homebrew locations on macOS, otherwise `pmd2raw` / `pmd2raw.exe` on PATH.
An unavailable helper gives a clear error without changing open documents.
The existing Apscribe build at `/Users/appajiambarishadarbha/apscribe` uses
the same Homebrew libpagemaker 0.0.4 backend (confirmed through its build
metadata and PageMaker QA notes). Pagenor reuses that installed backend
through its command-line helper; the Apscribe/Scribus GPL importer is not
copied into the MIT/Apache Rust codebase. No C++ implementation was read for this
initial P11.10 implementation; P11.14 later inspected verified MPL-2.0 helper source
and adds an external byte/source-story profile, described in its separate notes.
Apscribe's notes already establish editable-text native round-trip tests for
`.p65` and `.pm7`, while recording its own graphics/style/layout limitations.

No helper binary is bundled, no foreign library is linked and no helper source
is copied. The public [Document Liberation project listing](https://www.documentliberation.org/projects/)
identifies libpagemaker as a PageMaker reader with conversion utilities.

The process interface and native editable document construction are shared
Rust code for macOS, Windows and Linux. Helper availability and execution on
native Windows/Linux hosts remain unverified. Web builds retain the existing
architecture and return an explicit desktop-helper requirement for PageMaker.

## Recovered content and limitations

All pages reported by the helper become independent Pagenor pages. Text frames
retain their reported positions and sizes, text, tabs, spaces, forced line
breaks, paragraph boundaries, font names, font sizes, RGB text colours,
alignment, paragraph indents and paragraph spacing. Missing fonts use the
existing Find/Replace Font workflow. Separate one-face legacy families use
their sole catalogued style (e.g. Medium/Black), rather than inventing a
Regular face; missing or multi-style families are not guessed. A `Document Fonts` folder beside the
source file is scoped to that document through the existing loader.

Legacy Telugu text is retained as the characters emitted by the helper with
its original font names. **No ASCII-to-Unicode mapping is guessed.** Correct
appearance requires the matching original fonts; an unrelated Unicode font
cannot turn legacy codes into Telugu. Unicode conversion requires a verified
mapping/reference document for each encoding. The stock helper's diagnostic stream
is not a byte-preserving source-text interchange. In particular, its signed-char
build can omit bytes 128–255; see the explicit byte-preserving helper profile and
current Anu 7 validation in [P11.14](PAGEMAKER_LEGACY_BYTES.md).

P11.14's separate helper exposes original text-block IDs, so identical stories
with the same ID flow once through linked columns/pages. Distinct IDs remain
independent. Original chain pointers are not exposed: page/left-column/vertical
order is inferred and disclosed. The stock helper retains the previous conservative
inference for long, identical, equally formatted stories on at least three distinct
pages. `recoverRepeatedStories:false` keeps separate copies in either mode.

Embedded raster images exposed by the helper are retained as native assets/graphic
frames with geometry, rotation and relative paint order. Classic single-channel
8-bit palette TIFFs use bounded index/ColorMap rendering while retaining the
original embedded TIFF bytes. Other unsupported graphic
events are counted in warnings. Layers, masters, original crop/wrap/page settings,
colour management and exact typography remain incomplete. Mixed page sizes and
unrecognised nested/text structures are refused rather than silently flattened.
Every successful import shows an experimental-subset warning, including when
no graphics were reported. Check all pages, overset text and fonts before use. Import warnings remain
visible inside Find/Replace Font when missing-font repair also opens.
The source extension does not prove the authoring version: these formats share
compound-container conventions.

The SVG helper was evaluated and rejected for this importer: on Ramsha its
unpositioned text spans collapsed the multiline layout. This implementation
creates editable native text frames instead of presenting that SVG as a faithful
page preview.

## Bounds and validation

Input and stdout each have a 64 MiB limit, stderr 16 KiB, helper runtime 30
seconds, pages 1,000, frames 10,000 and events 200,000. Per-frame spans/paragraphs,
line length, colours, font-name length and measurements are bounded. The helper
runs without a shell against a temporary copy; the document cannot select an
executable, command switches or an output path. Both pipes are drained
concurrently. Malformed/truncated/ambiguous event structures, nonfinite geometry,
unknown units and unexpected diagnostics return errors before session mutation.

Synthetic tests cover editable Unicode/legacy text, formatting and native
round trips, frame geometry, multiple pages, omitted-art warnings, hostile
measurements/attributes, multiline/truncated records, file recognition and
failure without replacing the current document. The opt-in genuine-file test
uses `PAGENOR_PAGEMAKER_TEST_FILE`, `PAGENOR_PAGEMAKER_TEST_PAGES` and
`PAGENOR_PAGEMAKER_TEST_FRAMES` and optional `PAGENOR_PAGEMAKER_TEST_STORIES`; proprietary samples remain outside the repository.

The supplied **Ramsha.p65** passes with **five pages and five editable text
frames**, with one shared story rather than five duplicate copies, using GowthamiMedium, GowthamiExtraBold, GowthamiBlack and Arial.
The local **1 Mana Aaharapu Alavatlu.pmd** passes with **nine pages and 31 text
frames**; four polygon and two path events are explicitly omitted. Native
serialization and reopen retain the recovered text and character formats in
both samples. The initial installed GowthamiMedium/Medium and GowthamiBlack/Black
faces rendered legacy text as Telugu; GowthamiExtraBold was initially missing.
During the latest retest, all four source families resolve to installed Regular
faces, including GowthamiExtraBold. A source PDF is still needed to validate
complete appearance and typography.
The native composition retains all five pages/frames and one 5,145-byte story
without overset; text currently occupies three pages, leaving two frames empty.
Source line spacing/page distribution is not preserved, and was not guessed
to force text to fill five pages.

No GitHub publication, owner-session closure or /Applications replacement.

The final local quality run passes all seven `cargo xtask ci` gates: formatting,
workspace checks, strict Clippy, **1,920 tests** (seven ignores, including the
opt-in PageMaker corpus test), asset licensing, crate layering and 20 WASM
package checks. Both genuine-file tests pass separately on the final importer,
including one shared Ramsha story / five frames and PMD 31 stories / 31 frames.
The macOS release remains **0.5.2**, build **local-0.5.2-pagemaker**, with
unchanged app/document icons and an ad hoc signed bundle.

Initial native File → Open inspection in an isolated bundle/profile showed
Ramsha's five-page document and a font-repair dialog for GowthamiExtraBold.
The latest isolated retest visibly displays the import warnings, saves a new
.pgnr through the native Save dialog, returns to Welcome after closing only
the saved test copy, and reopens it from Recent Documents. Fresh source import
and the reopened native copy agree: five pages, five linked frames, one
5,145-byte story, 96 composed lines and no overset. All four source fonts now
match installed fonts; original pagination remains incomplete.
The original source hash remains unchanged. Existing owner sessions and the
first test session, which subsequently received user input, are left open.


## P11.15 update — local 0.7.1

The V2 helper reads every referenced page container and uses lossless printable
source-byte hex transport. Source page count is checked against emitted pages
when declared; otherwise completeness remains explicitly unverified. `file.open`
returns `sourcePages`, `pageCountVerified` and `converterProfile` with its imported
page count. The owner-confirmed Nov 1-28.p65 now retains all **28 pages**, 169 text
frames, 110 distinct stories and 53 embedded TIFF placements; the earlier five-page
copy was partial. See [complete-import notes](PAGEMAKER_COMPLETE_IMPORT_071.md) for
terminal vowel preservation, bounds, optional bitmap continuation and validation.
