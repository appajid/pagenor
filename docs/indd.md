# InDesign document (.indd) format notes

## Pages
- 4096-byte pages; last 12 bytes = u32 type, u32 link, u32 checksum
  (low 16 bits = byte sum of [0,4092) mod 65521; high 16 bits unknown).
- Types: 0 master, 2/4 roots (shadow pairs), 3 bitmap?, 5 page map, 6 B-tree node, 7 B-tree root,
  8 raw data page (physical, link 0), 9 object page (link = logical page number).

## Master page (pages 0..2, GUID 0606EDF5-D81D-46E5-BD31-EFE7FE74B71D + "DOCUMENT")
- byte 24 endian (1 = LE); u32 @29 major version, u32 @33 minor; u64 @264 save sequence.
- Pick the master with the highest sequence. u32 @0x160 -> type-2 root, u32 @0x3A8 -> type-4 root.
- type-4 root: u32 @128 -> page map (type 5). Page map: u32[32 + logical] = physical page.

## UID index (type-6 leaves: u32 count, u32 ?, count * 4 x u32)
- Location tree: (uid, slot << 16 | length, page, segment index).
  Segment with slot != 0: record on logical page. Slot 0: raw physical page.
  Object bytes = segments 2..n in order, then segment 1 (the tail).
- Class tree: (uid, class id, 0, 0).

## Object pages (type 9)
- Records from offset 0: u16 total length (incl. 4-byte header, padded to 4), u16 slot.
- Slot flag 0x8000: split record; body starts u16 0, u16 next slot, u32 next logical page.

## Objects
- Payload = sequence of (u32 implementation id, u32 length, data).
- Class 0x129 = binary blob (TIFF proxy, EPS, PDF, JPEG).
- 0x15b hierarchy: u32 spread, u32 parent, u32 n, n * u32 children.
- 0x151 item transform: 6 doubles (a b c d tx ty). 0x162b path geometry. 0x565 spread bounds.
- 0x501 spread, 0x1401 master spread, 0x3301 page, 0x6201 spline item, 0x401 group,
  0x263 multi-column text frame, 0x227 frame column, 0x2501/0x6601 placed graphics.
- Text-leaf 0x262: u32 kind, u32 owner strand, u16 record count, records.
  A record has u32 size (excluding that size word), u32 UTF-16 unit count,
  then one or more tagged text segments. A u16 tag's low 14 bits count this
  segment's units: 0x4000 is byte text, 0x8000 is UTF-16LE. Mixed segments
  share a record; their headers are not characters. An untagged wide single
  segment is also retained. Invalid lengths/counts/surrogates return errors.
  The writer maps style run lengths from UTF-16 units to scalar positions.
  P11.6 reads every ordered leaf of both text and style strands.

## Semantics found so far (2026-10-07)
- Doc uid 1: 0x301 = document layer order (u32 n, uids), back-to-front like IDML; retain empty user layers. The private page-storage layer has flag 1 at 0x304 offset 0 and the sentinel name `Internal_pages_layer_name`; it is not a user layer.
- Doc uid 1: 0x501 = spread order (u32 n, uids). Spread 0x503 = spread layers; spread layer (0x301): 0x303 kids back-to-front, 0x302 = (doc layer uid, u16 guides flag).
- Page 0x50F: 0x5DD bounds (l t r b), 0x5CC transform. Doc prefs uid 6 0x533 = page W/H.
- Spline 0x6201: 0x162B path (u32 nsub; u32 npts; per point u32 kind: 2 = anchor only, else left/anchor/right; u16 open flag),
  0x151 transform, 0x6E03 graphic attrs, 0x2C32 visible (0 = hidden), children via 0x15B.
- Group 0x401: transform in 0x40D; child transforms are relative to the group.
- Graphic attrs (u32 n; u32 id, u16 len, typed values): 0x6E68 fill, 0x6E69 fill tint, 0x6E64 stroke, 0x6E65 stroke weight, 0x6E66 stroke tint.
  Value types: 0x117 uid, 0x6E68 double, 0x6E69 point, 0x6E67 int, 0x6E65 bool, 0x6E63 meta.
- Swatch 0x1F05: 0x1F10 name, 0x1F01 (u32 space 5 RGB/6 CMYK/7 LAB, u16 n, n doubles 0..1 (LAB raw)).
- Story 0x201: 0x223 lists strands. Strand classes: 0x234 text, 0x236 paragraph runs, 0x235 character runs, 0x228 frame list.
  Strand 0x261 = u16 leaf count followed by `(u32 text length, u32 data UID)` pairs; data 0x262 = (u32 kind, u32 owner, u16 n, runs).
  Runs: u32 size, u32 length, u32 style uid, text attr list (u16 n; u32 id, u16 len, typed values).
- Style 0x205: 0x230 name + u32 based-on @4, 0x23F attrs. Chain root "[No paragraph style]" / "[No character style]".
- Text attrs: 0x1B01 colour, 0x1B02 font style (string), 0x1B03 size, 0x1B1B leading (-1 auto), 0x1B1A auto-leading,
  0x1B2B font family uid (0x3E03 objects, name in 0x3E05), 0x1B7E justification (0 left, 1 centre, 2 right, 3..6 justified).
- Multi-column object 0x263: preferences 0x2D1 = u32 column count at 0, f64 gutter at 4; columns bounded to 1..40 and finite gutter to 0..10,000 points.
- Frames: column 0x227 -> 0x220 frame-list strand -> story. Graphic 0x1702 (raster)/0x2501 (PDF)/0x6601 (EPS) -> 0x8CBC link 0x8C42 -> 0x8C9B
  resource 0x8C41 (0x8C92: file URI + embedded blob uid). Bounds 0x1633; placed PDF inner y = t + b - y_pdf.

## Embedded graphics
- Placed PDF/AI: the link resource embeds the PDF; it is written as IDML `<PDF>` with `Contents`.
- Placed EPS: DOS EPS (C5 D0 D3 C6) with a TIFF preview at header u32[5]/u32[6] (offset/length). The
  preview is usually an 8-bit palette TIFF with an alpha extra sample; it is converted to PNG.
- Links without embedded data: InDesign's proxy image (graphic 0x170D → 0x1708 → 0x119 → blob), TIFF or JPEG.

## Provenance
Clean-room: derived only from documents the author created, by comparing them with their own PDF exports.
No Adobe code, SDK headers or format documentation were used. No InDesign-produced file is committed;
tests build synthetic documents (`crates/indd/src/synthetic.rs`).


## Pagenor adaptation and validation

The original MIT OR Apache-2.0 #98 proposal is adapted with Pagenor names and
its existing document-font, recovery, native-format and asynchronous import flows.
Open and Place recognize `.indd`; imports start unsaved and show a subset warning.
The converter accepts at most 256 MiB; model/output budgets are at most 512 MiB.
Names are capped at 4096 bytes, story text at 4 MiB per story, combined text and
attributes at 64 MiB, style depth at 128, item depth at 32, items/runs at 100,000,
pages at 4096 and path points at 1,000,000. Generated story/style pieces are charged
as they are built. Style cycles and nonfinite page/item/path geometry return errors.

Tests use synthetic documents constructed in Rust. Real-file version breadth,
layout fidelity and native platform execution are not established by these tests.
Unsupported options are listed above and in the import warning. An IDML export
from the source application remains the broader interchange route.

P11.6 owner-authorized real-file observations extend the original clean-room
notes without committing source files or fonts. Paragraph spacing uses 0x1B26
(before) and 0x1B27 (after); justification 4 preserves a left-aligned final line.
[Measured recovery, tests and remaining pagination limits](pagenor/INDD_RECOVERY.md).

## Portfolio validation (2026-10-11)

The owner supplied `Appaji_Portfolio2.indd` and asked to preserve layer sequence.
Observed against its companion IDML/PDF and linked-image folder, read only:
version 21.6, sequence 11, 22 pages, 57 placed raster images, six user layers.
The prior IDML conversion reversed the layer list and omitted class 0x1702.
The local fix preserves layer order and empty layers, restores image proxies
and carries multi-column count/gutter through the existing IDML import.
No owner file, linked artwork or proprietary font is included in repository tests.
[Measured results and fidelity limits](pagenor/INDD_PORTFOLIO.md).
