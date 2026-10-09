# Table direction and control contracts

Tables inserted or converted from text inherit the resolved paragraph direction,
including its paragraph-style chain. The Table panel offers Left to Right and
Right to Left column ordering through `table.options`; undo restores the previous
ordering. New text stories use document binding direction as their default.

Control/MCP typing uses an explicit valid text caret independently of the active
selection tool. Focused dialog fields keep the synthetic keyboard path. Ordinary
keyboard input retains the existing tool and clipboard behavior.

Rendering/export accepts an omitted page as the current desktop page (page zero
headlessly). Explicit null, negative, fractional, nonnumeric and out-of-range page
values return errors before writing. Missing story queries return errors rather
than empty summaries. Control quit continues through Pagenor's recovery and
unsaved-document handling.

PDF/A exports disable raster interpolation while preserving placed images and
Pagenor's damaged-raster prevalidation. Ordinary direct PNG/JPEG/GIF/WebP paths
retain interpolation; decoded RGBA fallback images retain krilla's default false.

Adapted upstream #128 and #118 under their original MIT OR Apache-2.0 licences;
see `PAGENOR_UPSTREAM.md`. Native platform/browser execution remains a separate
milestone check.
