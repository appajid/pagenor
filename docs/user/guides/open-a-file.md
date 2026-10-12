---
title: Open a document
description: Native Pagenor, IDML, INDD and supported PageMaker imports.
order: 20
verified: 0.7.0
---

Choose **File → Open…** (Ctrl+O; Cmd+O on macOS), or Open on the Welcome screen.
Select a `.pgnr` document, a legacy `.pagenor` document, or a supported import.
Pagenor opens documents in tabs. Recent files are available through Welcome thumbnails
and **File → Open Recent**.

## Layout imports

IDML and INDD are supported import paths. IDML is preferred interchange when available;
INDD conversion can still omit unsupported source structures. Supported PageMaker
6.5/7 extensions include `.p65`, `.pmd` and `.pm7` (plus supported template aliases).
Legacy PageMaker ASCII text needs the right font/encoding. These readers do not imply
complete fidelity with every source application or version.

Check the page count, layer order, text, images and import diagnostics. The bottom
status indicator opens the issue list, where available diagnostics can be inspected.
Missing source fonts or linked files may change the result. Install/activate licensed
source fonts, or use the supplied Document Fonts folder for supported scoped imports.
See [InDesign files](../faq/indesign-files.md) for details.

## Save and export

Use **File → Save** (Ctrl+S; Cmd+S on macOS) to save an editable `.pgnr` copy.
Native format 5 retains the new blank-paragraph typing state. Pagenor reads previous
native versions; older app versions may refuse format-5 saves.

Use **File → Export IDML…** for supported interchange, or
[Export PDF](export-a-pdf.md) for delivery. Use [Place](place-images-and-text.md)
to bring images, PDFs or text into an existing layout.
