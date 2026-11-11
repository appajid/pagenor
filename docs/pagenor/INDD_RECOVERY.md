# INDD story recovery — P11.6

Local correction on Pagenor 0.5.2, 2026-10-11. Owner's current application session
is preserved; the separately staged build must be used to exercise this change.
No GitHub upload, system-font changes or source-document changes are made.

## Problems corrected

- Story indexes contain a leaf count and ordered `(text length, data UID)` pairs.
  The importer formerly followed only the first pair. It now reads every text,
  paragraph-style and character-style leaf in order, within existing limits.
  Missing, truncated or repeated leaf references return an error instead of
  silently importing a shortened story.
- A text record can alternate multiple byte and UTF-16 segments. Formerly a
  record's first segment determined decoding for the entire record, causing
  segment headers to become unrelated Unicode characters and font boxes.
  The reader now consumes each segment's tagged length and encoding, checks
  their total against the record's UTF-16 character count, and preserves
  surrogate pairs across segment/record boundaries. The older untagged wide
  layout remains supported.
- Fonts with typographic style names can also advertise a legacy `Regular`
  subfamily alias. Canonical style names now win over another weight's alias;
  a loaded alias cannot hide a lazy installed canonical face. Document-font
  precedence and lazy menu/catalogue behavior are retained.
- Paragraph space before/after is written into the intermediate IDML, including
  inherited spacing. Source justification value 4 now preserves a left-aligned
  last line, as observed in the owner's INDD/PDF pair. Other unverified binary
  justification values remain in the existing subset, rather than claiming
  complete enum coverage.
- Default page labels continue across spreads instead of resetting to 1/2 in
  each spread. Source section-numbering metadata is still outside this reader.

Shared Rust changes support macOS, Windows, Linux and WASM without new dependencies
or unsafe code. The production IDML parser and text-composition engine are unchanged.

## Owner-file validation

The owner explicitly supplied Mondicheyyi.indd. Its container reports version
21.3, sequence 23. The source file and its companion PDF were read only; neither
source document nor proprietary font binaries are copied into the repository.
Synthetic regressions reconstruct the relevant structures in Rust.

- Original model: only the first leaf of the main story, 2,377 wrongly decoded
  characters. Corrected model: all 11 text leaves and 54,806 UTF-16 units, including
  45,112 Telugu code points; all eight character-style leaves are retained.
- All 18 spreads, 34 page records and 33 linked text frames are retained.
  The companion source PDF also contains 34 pages; its final page is blank.
- Native imported text exactly matches the recovered source after the IDML
  importer's existing CR-to-LF and final paragraph-terminator normalization
  (54,805 scalar characters). Save/reopen preserves text, frame order and page count.
- Installed Mandali Regular, Mandali Bold and Ramsha Regular are used in the
  native headless composition. There are zero missing-glyph IDs and no overset.
  Mandali Regular resolves to `/Library/Fonts/Mandali-Regular.otf` rather than
  the unrelated weight that happened to share a Regular alias.
- Native page renders and PDF extraction were checked. Removing only whitespace,
  the exported PDF retains all 46,980 source text code points exactly.

## Remaining fidelity difference

Text recovery is fixed; exact source pagination is not complete. After preserving
paragraph spacing, Pagenor currently fills 31 of the 33 source text frames. The
source PDF fills 33 pages. Two later retained text frames are consequently blank
following reflow; no source story content is lost. Do not describe this as exact
33-page layout reproduction. Further measurements are needed for source composition
metrics and unsupported paragraph/frame settings. Master items, frame insets and
baseline settings, complex threading, tracking/kerning, tables, effects and wrap
remain covered by the existing INDD subset warning. IDML remains the broader
compatibility path.

## Validation

All seven `cargo xtask ci` gates pass: 1,898 tests passed, six existing ignores,
zero failures; 20 WASM packages. Focused synthetic coverage includes mixed segment
encodings, surrogate boundaries, multi-leaf text/styles, damaged indexes, canonical
font-style selection, lazy lookup, document scope precedence, paragraph spacing
and 33 sequential page labels. Windows/Linux shared font/import compile checks
pass; their native desktop execution remains a milestone check. A separate macOS
arm64 release bundle keeps the owner app/document artwork and 0.5.2 version,
with build identity `local-0.5.2-indd-recovery`.
