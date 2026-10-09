# Advanced data merge

P09.3 adapts upstream #109/#125 into the existing Rust command/document architecture.
Source selection accepts CSV/TSV, JSON arrays of records, XLSX and BIFF8 XLS.
Each source retains its own enable flag, filter and sort. The panel inserts fields,
previews records and configures multiple records or drawn grids. Merge creates an
ordinary editable document, with optional PDF output; the template remains available.
Joins, prefixes/postfixes and combined sorting are command interfaces at present.

Limits inherited from the Office importer still apply: bounded ZIP preflight,
64 MiB text, 100,000 items and padded-cell accounting. Grid planning is capped at
1,000,000 cell slots. Formula evaluation supports the documented small subset,
uses a shared 1,000,000-step workbook budget, bounds unary nesting to 64 and cell
references to 32. Cached results take precedence; unsupported or unevaluated
formulas produce warnings. Number/date formatting is intentionally limited.
BIFF8 CONTINUE records split inside a character are not fully reconstructed;
XLSM and general spreadsheet calculation are unsupported.

Data/image files are limited to 256 MiB including actual reads. HTTPS images use
the host curl executable with protocol, redirect, transfer-size and timeout limits;
missing curl or network failures produce merge warnings. WASM does not fetch remote
images. HTTP uses the bounded native reader. Remote image requests happen during
preview/merge when the template references them. Do not confuse these with automatic
font downloads. Redirects and network availability may vary between platforms.

Local checks cover source persistence/update, joins and first-match duplicates,
filters/sorts/affixes, grid geometry/adoption/merge, PDF output, image handling,
unsupported formulas, input budgets and UI command/undo delivery. CPU-rendered UI
inspection covers sources, grid drawing and scrolling to the lower controls at
1440 × 900. Native Windows/Linux execution and browser interaction remain milestone
validation tasks. Adobe/dongle font activation remains pending separately.
