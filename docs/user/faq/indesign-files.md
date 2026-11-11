---
title: Can Pagenor open InDesign files?
description: Pagenor opens IDML and imports supported INDD content.
order: 30
verified: 0.7.0
---

Pagenor opens InDesign Markup (`.idml`) and imports supported content from
InDesign (`.indd`) files through **File → Open…**. It exports IDML through
**File → Export IDML…**. Save your editable Pagenor copy as `.pgnr`.

IDML is the preferred interchange route when available. INDD import recovers pages,
text, fonts, images and layer order from supported structures, but conversion can
still differ from the source layout. Check every page and the import diagnostics.
Unavailable fonts, linked assets and unsupported source features can affect fidelity.
Install or activate the original fonts where their licence allows it; verify Indic
text and font styles in the imported document before publishing.

Saved image clipping paths, custom wrap boundaries and explicit paragraph-spacing
overrides are retained by the supported IDML paths. This does not guarantee complete
InDesign parity. Pagenor also has supported PageMaker 6.5/7 import paths; legacy ASCII
text requires the correct font/encoding rather than automatic Unicode guessing.

See [Open a file](../guides/open-a-file.md) and [Export a PDF](../guides/export-a-pdf.md).
