# P11.14 — PageMaker legacy character preservation

Local 0.7.0 investigation/correction, 13 October 2026. Anu 7 is the owner-confirmed
version of both the supplied font and the source publication, created on Windows. This task separates
missing source bytes from glyph-encoding fidelity. The owner now reports text
**99% accurate** after the original Type 1 source encoding/outlines were retained
in private document-scoped fonts. This is user feedback, not a measured whole-document accuracy rate. The owner confirms that only two passages (the credits
and Demand Drafts paragraph) match the separate reference PDF. Its remaining text,
page count and layout are not an oracle for this publication. No released package is replaced by this investigation.

## Reproduced cause

The installed libpagemaker 0.0.4 uses a signed `char` for a control-character
comparison. On this Mac, every text byte 128–255 compares below 32 and is skipped.
The raw, plain-text and SVG tools all lose these bytes. Selecting a different font
cannot recover bytes absent from the helper output.

An unmodified, separately built copy of libpagemaker 0.0.4 with
`CXXFLAGS='-O2 -funsigned-char'` preserves those text bytes. On the owner-supplied
publication it recovers 25,865 high bytes, with the same five reported pages and
31 text frames. Its stream is 8-bit text, **not UTF-8**; simply removing the signed
comparison without adding explicit decoding would break Pagenor's strict decoder.

## Rust import changes

Pagenor now supports an explicit byte-output helper profile. Only `insertText`
payloads are decoded as source bytes; event structure and attributes must remain
UTF-8. Source slots are carried in U+F000–F0FF until document font scopes and styles
have resolved. Each run then uses an available ordinary Windows-1252 glyph or its
private symbol slot. Undefined Windows slots retain their private byte rather than
being removed or replaced by U+FFFD. English stays ordinary text in its own runs.
This is lossless code preservation, **not verified Anu 7-to-Unicode conversion**.
The owner-supplied Anu 7 TTF and original Type 1 PFB collections arrange glyph slots
differently despite shared family/version labels. Twelve original PFB families were
converted privately to CFF OpenType using their original Encoding, outlines and
advances, with Windows-1252 aliases and U+F000 byte slots. Geometry was verified
for all 2,777 converted glyphs; no guessed Unicode or approximate outline mapping
was adopted. These fonts stay beside the private native copy in `Document Fonts`,
retain original notices and are excluded from Git and public redistribution.

The existing strict structure, process timeout, bounded pipes/events/frames, document
font scope, original-file protection and save-as-native behavior remain. Only the
fonts used in imported runs are loaded. Character run byte lengths are rebuilt and
validated, with paragraph formatting retained through native save/reopen.

An explicit `PAGENOR_PAGEMAKER_CONVERTER` override keeps precedence. Pair it with
`PAGENOR_PAGEMAKER_BYTE_OUTPUT=1` **only** for a byte-preserving build. Do not set that
flag for a Unicode-emitting converter. Otherwise a managed `helpers/pmd2raw-byte-v1`
(`.exe` on Windows) is selected from:

- macOS: `~/Library/Application Support/Pagenor/`
- Windows: `%APPDATA%/Pagenor/`
- Linux: `$XDG_DATA_HOME/pagenor/`, or `~/.local/share/pagenor/`

The standard installed helper remains available and adds a specific byte-loss warning.
WASM continues to require a desktop helper. Shared Rust behavior is portable; the
new helper build has been executed only on this ARM Mac. Native Windows/Linux
helper execution remains pending. Some non-Telugu source fonts and precise
typographic metrics remain unresolved.

## Helper build provenance

The helper is an optional external process; no foreign code is linked/copied into
Pagenor's Rust implementation and `unsafe_code = "forbid"` remains. libpagemaker
0.0.4 is **MPL-2.0**, as verified from its distributed COPYING and source headers;
it is not the GPL Apscribe/Scribus importer. Its licensed text-decoding implementation
was inspected to diagnose the signed-character bug. Earlier notes saying no C++
implementation had been read describe the initial importer, before this investigation.
No GPL/AGPL/LGPL implementation was read or copied for this correction.

Unmodified source:
https://dev-www.libreoffice.org/src/libpagemaker/libpagemaker-0.0.4.tar.xz

SHA-256:
`66adacd705a7d19895e08eac46d1e851332adf2e736c566bef1164e7a442519d`

Build outside the repository with Boost/librevenge development dependencies.
The separate source-story patch preserves the parsed text-block ID through the
geometry/collector and emits `pagenor:story-id` on each text object. It changes
no glyph mapping and stays MPL-2.0 outside the Rust codebase; corresponding
source archive, patch and COPYING accompany the local helper. Apply that patch
with `patch -p1 < /path/to/pmd2raw-story-ids.patch` before configuring for column
recovery. The unsigned-char flag alone fixes byte loss but does not expose story IDs.

```sh
./configure --without-docs --disable-shared --disable-werror \
  CXXFLAGS='-O2 -funsigned-char'
make -j4
```

On Homebrew, supply `CPPFLAGS=-I/opt/homebrew/include` and
`LDFLAGS=-L/opt/homebrew/lib` as needed. The resulting raw tool still depends on
librevenge. Install it as `pmd2raw-byte-v1` with its MPL notice and corresponding
source. This is an explicit build profile, not a claim that stock upstream binaries
already contain the correction. GCC/Clang's unsigned-char flag applies on other hosts;
Windows toolchain/build/dependency setup must be validated before distribution.

## Validation

Three new synthetic regressions preserve every high byte through native round trips,
keep Latin/symbol runs separate, retain formatting and reject malformed controls or
non-UTF-8 metadata. The existing Unicode/import-failure/session-preservation tests
remain. Focused tests: 11 pass, one opt-in real-file test ignored. Full quality and
real-file/native checks are recorded when completed below. Owner documents/fonts and
reference PDF subsets stay outside Git; no original publication or system font is modified.

Final byte-preservation checks: all seven `cargo xtask ci` gates pass, **2,577 tests,
zero failures, seven ignores**, twenty WASM package checks. The opt-in real-file
import/native-reopen test passes separately. The independently saved native copy
has five pages, 31 frames and 31 stories. Reversing the explicit byte transport for
all 31 saved stories reproduces **50,285 helper text bytes**, including all **25,865**
formerly omitted high bytes, with identical tab/space/line/paragraph order.
The source SHA-256 remains unchanged.

The original-Type-1-font native preview was inspected, then the owner confirmed
that the text displays 99% accurately. Native reopen retained all text/style runs
and loaded twelve document-scoped font families. Myriad Condensed, Times New
Roman TUR, ZapfDingbats and ZapfDingbats BT were still missing in that test. Only
the two owner-confirmed reference passages are used for visual comparison; the
separate PDF's other text/layout/page count are not validation targets.

## Columns and embedded rasters

The owner then reported repeated content in multiple columns and missing embedded
images. The separate MPL helper now exposes original text-block IDs. Frames with
the same nonzero source ID and identical text/formatting become one native story,
including same-page columns. Different IDs never merge just because text matches;
conflicts stay separate with a warning. Frame flow order is inferred by page,
left-to-right column position, then vertical position because the helper does not
expose original chain pointers. Item paint order is preserved independently.
`recoverRepeatedStories:false` disables recovery; the prior conservative inference
is retained only when the trace has no source IDs.

Embedded raster events are stored as native assets and editable graphic frames,
retaining bytes, helper geometry and rotation in item paint order. Matching assets
are reused. Base64 is validated by canonical round trip; MIME must match sniffed
raster bytes. Dimensions/decoded pixels are bounded to four million per image and
sixteen million per import, with a 64 MiB decoder allocation limit. Malformed or
unsupported embedded graphic events fail clearly rather than silently vanish.
Other unsupported vector events are still counted in warnings.

Three additional synthetic regressions cover column flow/identity/conflicts/disable,
image placement/rotation/paint order/resource reuse/native persistence, and corrupt
base64/MIME/geometry/pixel budgets. A further regression covers the helper
text angle values mislabeled `in` (still interpreted as degrees). The real file
also contains an 8-bit palette TIFF rejected by the locked decoder. A bounded
private grayscale view decodes its indices, then applies the original 16-bit RGB
ColorMap without changing embedded asset bytes. A synthetic exact-color/native
rendering test covers palette handling, pixel limits and malformed interpretation
tags. Palette decoding is limited to classic single-channel 8-bit TIFF, sixteen
MiB encoded bytes and four million pixels. Other TIFF paths remain as before.
Focused importer suite: fifteen pass, one opt-in sample ignored; the palette
regression passes. Final full-gate and new genuine-file/native results are recorded below.

No owner session is closed, no system font is installed, no released package is
overwritten and no GitHub publication occurs for this local correction. The test
session which received owner input is left open; further native checks use a new
profile/bundle/port and a new document path.

## Completed local validation — 13 October 2026

All seven final `cargo xtask ci` gates pass: **2,582 tests, zero failures, seven
ignored; twenty WASM packages**. The opt-in genuine P65 import/native-round-trip
test also passes separately: five helper-reported pages, 31 text frames and twenty
stories. Four original shared-story groups (2, 5, 3 and 5 frames) are threaded once.
All five embedded TIFF placements/resources survive native save/reopen, including
the indexed TIFF and one rotated graphic. Normal managed-helper discovery succeeds
without environment overrides; its MPL source archive/patch/notice accompany the
helper in local Pagenor user data.

For every one of the 31 frames, story text, character runs and paragraph formats
match the earlier font-recovered native copy. The five native image byte hashes
match the helper payloads exactly, and text/image paint sequence is unchanged.
The original P65 SHA-256 is still
`959dd2c3c2c76569599625afca978d720ac8932ba76f59f3a492e1585733083f`.
All five page renders and the isolated native application were inspected. Native
open loads all twelve adjacent source-font families. The corrected copy is a new
file; earlier owner-edited test documents are not overwritten.

Preflight on this copy reports 23 errors: nineteen overset frames and four missing
non-Telugu font families. Four low-resolution-image and two RGB-swatch warnings
remain. Exact original spacing/metrics, original chain pointers, image crop/wrap,
layers and masters remain outside the helper's recovered subset. Do not mistake
owner-reported text accuracy for a complete layout/fidelity milestone.

A separate ad hoc signed and signature-verified macOS ARM64 local app is prepared
with both Pagenor app/document icons and font/branding licences. No running bundle
or `/Applications` installation is replaced. Version remains **0.7.0** locally;
published 0.7.0 Mac/Windows/Linux assets remain immutable. The shared Rust changes
are cross-platform, but native execution of this optional helper and this private
sample on Windows/Linux are not yet validated. Owner fonts/documents stay out of Git.
