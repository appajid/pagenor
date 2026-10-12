# P11.15 — PageMaker page completeness and terminal source bytes

Local Pagenor **0.7.1** correction, 13 October 2026. The owner confirms
`Nov 1-28.p65` has **28 pages**. The earlier five-page copy recovered only the
first referenced page container and must be treated as a partial import. The
separate PDF is not a document-wide oracle; only two earlier passages match it.

## Confirmed causes and changes

The verified MPL-2.0 libpagemaker 0.0.4 parser selects only the first PAGE
record container. The separate V2 helper traverses every referenced PAGE
container and counts those source records before converting any page content.
For single-sided documents it declares `pagenor:source-page-count` on its page
events. The Rust importer requires matching source and emitted page counts,
rejects conflicting/out-of-range declarations and returns `sourcePages`,
`pageCountVerified`, `pages` and `converterProfile`. Older helpers without a
source declaration explicitly warn that completeness is unverified. Page records
represent spreads in some facing-page documents; exact physical-page assessment
for that topology remains separate work and is not inferred from the PDF.

An additional bitmap uses a complete primary payload with no continuation
sequence. The stock parser unconditionally requires a continuation and aborts.
V2 accepts its absence; the existing Rust MIME/dimension/decoder checks still
validate the assembled raster. The real document includes an approximately
6.4 MiB graphic event. Only embedded-graphic events may now be up to 16 MiB;
ordinary/text events remain capped at 1 MiB, the whole trace at 64 MiB, pages at
1,000 and items at 10,000. Pixel/decoder limits remain unchanged. No whole font
catalogue is loaded eagerly.

The owner's example `P "Í_… J…ìöQ LO_…O>` should end with Telugu **ఉండాలంటే**.
The actual source contains byte **0xE8** after **0x3E**. It is lost when invalid
8-bit sequences pass through the helper's UTF-8 drawing-string interface, before
Pagenor selects a font. V2 carries printable source bytes as marked ASCII hex,
then Rust strictly decodes them into the existing document-scoped glyph slots.
This restores the terminal vowel without substituting Unicode text or guessing
a paragraph-specific glyph mapping. Unknown control-byte semantics remain outside
this profile; tab/CR handling stays explicit. Native source codes remain legacy
codes, not standard Unicode Telugu.

One source text frame on page 17 has zero height. Its finite geometry and text
are retained unchanged for overset inspection rather than aborting the whole
publication or resizing it. A synthetic native-reopen/layout-query regression
proves zero-width and zero-height frames do not crash.

## Platform and ownership

V2 is installed separately as `helpers/pmd2raw-byte-v2` (`.exe` on Windows),
selected before V1 in Pagenor's existing platform-specific user-data directory.
The old helper and all already running app bundles/sessions are preserved.
Explicit external helpers require `PAGENOR_PAGEMAKER_BYTE_OUTPUT=2` for the hex
profile; V1 retains `=1`, Unicode helpers omit the flag. No filename in a
publication can select an executable. No foreign implementation is linked/copied
into Rust and `unsafe_code = "forbid"` remains.

The original upstream archive URI, SHA-256 and MPL verification are in
[P11.14](PAGEMAKER_LEGACY_BYTES.md). Corresponding source archive, V2 patch,
COPYING and build instructions accompany the local helper. The patch adds page
container traversal/counting, original story IDs, optional image continuation
and UTF-8-safe byte transport. Private diagnostics used during investigation
were removed; successful helper execution produces no stderr.

Twenty-two original Type 1 families are converted into a new private
`Document Fonts` folder, retaining source encoding, advances and notices.
Source composite outlines are decomposed before CFF conversion; verification
accounts for equivalent straight curves and zero-area collinear retraces in
filled glyphs. 5,024 glyphs are checked. No currently loaded private fonts or
system fonts are overwritten. Proprietary fonts/publication files stay out of
Git and public release assets. Other missing Latin/symbol families still require
the owner's matching fonts; no substitute mapping is fabricated.

## Validation

The complete helper reports **28 pages, 169 text frames and 53 embedded TIFF
placements**, with zero stderr. The new native copy retains **110 stories** with
shared frames threaded rather than repeated content. Normal V2 discovery reports
`sourcePages:28`, `pages:28`, `pageCountVerified:true`, `converterProfile:"hex-v2"`
and twenty-two document font families. First and final page renders were inspected;
the contents line visibly reads **ఆ వేడి అలాగే ఉండాలంటే**, including the final
vowel. All 28 pages render after native save/reopen. The opt-in genuine-file
round-trip test passes. All seven `cargo xtask ci` gates pass: 2,586 tests pass,
none fail, seven are ignored; twenty crates pass the WASM check. The ad hoc
signed macOS ARM64 0.7.1 app passes signature verification and opens the scoped
P65 through the normal native file-open route with 28 verified source/imported
pages, 53 images and 22 document-font families.

The native preflight snapshot contains 107 overset reports, six unavailable
font families, 29 low-resolution warnings and two RGB warnings. Completeness
of page recovery does not establish original layout fidelity. The six fonts
are Helv, Myriad Condensed, Rupee Foradian, Times New Roman TUR, ZapfDingbats
and ZapfDingbats BT; the font-repair dialog is left available for owner review.
Existing open sessions and the original source publication remain unchanged.

Synthetic regressions cover every high source byte and terminal 0xE8 through hex
transport/native serialization, malformed/missing hex markers, source-page count
mismatches and limits, a bounded graphic larger than 1 MiB, and zero-sized frame
layout. The prior font scope, palette TIFF, rotation, resource identity/order,
text grouping, malformed-input and session-preservation tests remain.

Exact original typography/spacing/crops, layers, masters, nonprintable legacy
codes and original chain pointers are still incomplete. Frame chain ordering
uses the disclosed page/column inference. Low-resolution helper previews and
overset/missing-font preflight results must still be reviewed. This is a local
0.7.1 build; native Windows/Linux helper execution and publication are pending.
Published 0.7.0 assets are unchanged.
