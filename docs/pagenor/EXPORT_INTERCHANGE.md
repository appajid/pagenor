# EPS, Links and editable PowerPoint export

## EPS and relinking

Placed EPS retains the original file, its MIME type and point bounding box. Screen,
PDF and raster export use the TIFF preview, or a crossed placeholder when there is
no preview. Pagenor does not execute arbitrary PostScript. PDF keeps CMYK TIFF
preview channels. HTML/EPUB convert previews to PNG and report unusable graphics.
IDML round trips preserve EPS source and its resource type.

The Links panel's Relink picker lists graphics. Relink, Update and folder relinking
prepare and validate replacements before editing; a failed batch preserves the
original document/history. A successful batch has one undo step. PDF page selection
is retained and invalid replacement pages return errors. Link status compares source
bytes through bounded chunks instead of allocating a second complete file.

Place/table/relink reads are limited to 256 MiB per source; actual reads are capped,
including a growing file. Prepared link batches are limited to 512 MiB. EPS sizes
must be 0.001–20,000 points. Placement rejects nonfinite resulting geometry.

New native saves use format 3 to declare EPS rendering capability. Versions 1 and 2
and legacy JSON remain readable. Version-1 paragraph-default migration remains;
version 2 keeps its newer defaults. Save never downgrades a new document to version
1 based on whether it contains EPS. `.pgnr` and legacy explicit paths remain.

## PowerPoint

File → Export PowerPoint opens page-range, line-layout, rasterization and embedding
choices, then the platform Save picker/download. Empty page range means all pages.
Keep preserves composed line breaks; Reflow lets PowerPoint compose native text
boxes. Pages become slides with editable shapes, text, pictures and tables. Parent
items are copied to their pages and layer stacking is flattened.

Unsupported effects, placed PDF/EPS and type on a path can be rasterized. Approximate
or omitted features produce warnings after export. Groups containing threaded text
retain their text flow and warn about effects that cannot be represented. Failed exports keep options open
with an error. Command `file.exportPptx` and CLI `.pptx` export use the same backend.

Only accessible static TrueType outlines with appropriate OS/2 metadata are embedded;
collections, variable/CFF fonts, restricted and preview/print-only faces are omitted
with warnings. No-subsetting faces are embedded whole. Bitmap-only permissions are
honoured according to OS/2 table version; flags remain unchanged. Interpretation:
[OpenType OS/2 fsType](https://learn.microsoft.com/en-us/typography/opentype/spec/os2#fstype).
Fonts remain usable for ordinary Pagenor editing regardless of export eligibility.

Input checks bound pages to 4096, item/story nodes to 100,000, nesting to 64,
source JSON to 64 MiB and source assets to 512 MiB. Slide XML/media and generated
package size are limited to 512 MiB; embedding is limited to 128 MiB. Rasterized
export assets have a cumulative 512 MiB limit. These are local export limits, not
claims of complete PowerPoint format equivalence or native PowerPoint validation.

Original MIT OR Apache-2.0 proposals #150/#170 are adapted with Pagenor identifiers.
Owner artwork and generated contributor metadata are retained; provenance is in
`PAGENOR_UPSTREAM.md`. Tests use synthetic graphics and package/XML inspection.
Native PowerPoint and cross-platform GUI execution remain milestone checks.
