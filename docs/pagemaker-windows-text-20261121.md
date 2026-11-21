# Windows PageMaker text transport — P11.32 (21 November 2026)

Local repair after 0.15.0; no installer, release, original publication or running
Mac session is replaced. The owner reports both computers on the 0.14 series.
The same defect is present in the published 0.15.0 binary source.

## Source-preservation contract

The owner explicitly requires file-format conversion only. Read P65/PM7 directly;
retain the original legacy character-code values, text, font assignments, styles,
threading and recoverable graphics. Installed original fonts interpret those
legacy codes. This repair does not translate text or turn Anu 6/7 text into Indic
Unicode. Pagenor's internal UTF-8 representation and existing ordinary/symbol cmap
slot selection retain the same source-byte meaning as the previous Mac path.

No PDF is used to reconstruct text, images or layout. The supplied PDFs are
optional visual references for local diagnosis, not converter inputs, sidecars
or dependencies. The expected-JSON sidecar belongs only to the synthetic installer
test; normal P65/PM7 import does not need it either.

## Cause and repair

The byte-v2 helper deliberately sends legacy source bytes as ASCII hex inside
`insertText (@PAGENOR-BYTES-V2@...)` records. Windows text-mode output has CRLF
record endings. The Rust byte decoder split only at LF, then required the record
to end with `)`. The remaining CR made every Windows text record miss this test.
It passed through as UTF-8 metadata, and the later parser's CRLF-aware `lines()`
accepted the encoded marker as document text. This affects English and legacy
Telugu alike; changing fonts cannot fix transport text.

Normalize exactly one trailing record CR before recognizing events. Preserve
payload spaces/high bytes, reject malformed byte-text records instead of letting
them fall through, and retain the existing budgets and strict attribute checks.
LF, CRLF, mixed endings and no final newline produce the same normalized trace.

A separately configured helper named `pmd2raw-byte-v2[.exe]` now selects V2 when no
explicit profile is provided, including mixed-case Windows names. Byte-v1 names
select V1. Explicit `PAGENOR_PAGEMAKER_BYTE_OUTPUT` values retain precedence,
including 0. Generic helper names retain their UTF-8 default. This recognizes a
helper's transport protocol, not the document's Anu 6/7 font encoding; font byte
mapping, style grouping and Unicode-conversion policy are unchanged.

## Regression and packaging tests

Three original synthetic regressions fail before the repair: V2 CRLF/mixed input,
V1 CRLF with payload spaces, and malformed Windows records that previously passed
through. They pass after the repair. A fourth test covers named helper overrides
and explicit-profile precedence without mutating global environment variables.

The earlier packaged smoke fixture had three blank pages. It verified helper
availability, page traversal and native round trips, but could not detect broken
text decoding. It now contains one original English frame, the high byte E9 and
three pages in two PAGE containers. Its generated expected-text sidecar is ASCII
JSON with escaped Unicode. Both native installer verifiers require the exact
English/high-byte story text after native save/reopen, as well as page count.
Missing text, leaked markers and a changed accent must fail validation. No user
publication, proprietary font or binary fixture is committed.

All **24 packaging contracts** pass. All seven workspace gates pass: **3,227 tests
passed, zero failed, seven existing ignores; twenty WASM packages**.

## Private-file verification

The two supplied P65s and their PDF references remain local. With the fixed CLI,
each original was imported once through its real Mac helper and once through an
adapter that emits Windows CRLF. The adapter has a mixed-case V2 executable name
under a path containing spaces; the explicit byte-profile environment variable is
removed. Both complete native document graphs match, allowing only the fresh
native creation timestamp to differ. Every embedded asset has identical bytes.
All story text is checked for leaked markers and originals are hash-checked.

| Sample | Pages | Stories | Imported UTF-8 text bytes | Image assets |
|---|---:|---:|---:|---:|
| English equipment document | 5 | 2 | 6,897 | 0 |
| Telugu publication | 32 | 122 | 273,980 | 47 |

Both first-page renders were inspected against the supplied Mac references;
English/legacy Telugu text is visible and no transport markers remain. This
preserves the existing Mac interpretation; it is not a new full PageMaker layout
fidelity claim or Anu-to-Unicode conversion. Original fonts remain necessary.

This is real-file transport simulation on Mac, not execution of the new build on
Windows. The next x64/x86/ARM64 Windows installer milestone must run the strengthened
nonblank fixture natively. The published 0.15.0 binaries still need a new patch
build. Reimport originals after updating; previously saved bad conversions are not
automatically rewritten. The earlier PDF-progress change is retained on this branch.
