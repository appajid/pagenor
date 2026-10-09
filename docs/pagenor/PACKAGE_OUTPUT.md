# Package output safety — P00.4

File › Package and Copy Links To share checked output handling on macOS,
Windows and Linux. Files are staged beside their destination and individually
replaced after successful writes. Failed staging removes its temporary file.
Existing regular output files can still be overwritten. Existing file redirects,
redirected directories (including existing ancestors) and non-regular destinations
are refused. Select a real folder rather than a symlink/Windows junction; command
callers using OS aliases such as macOS `/tmp` should supply the resolved real path.

Document titles and linked/font filenames become single portable components.
Slash and backslash paths, reserved punctuation, trailing spaces/dots and Windows
device names are handled. Names are shortened on UTF-8 character boundaries,
with room for suffixes; short file extensions are preserved. Link/font collisions
use Unicode canonical caseless comparison, including decomposed accents and
Greek final sigma. These names affect the package copies only; the original title
remains in the document and report. Native output remains `.pgnr`.

Required native/IDML/PDF exports are prepared and all required document, report
and link destinations are checked before file writes. Native serialization or PDF
preparation refusal preserves prior package bytes. Copy Links To relinks the live
document only after all requested writes succeed. Package keeps the live document
unchanged. Font copying retains document scopes, fallback faces, licence decisions
and the no-file report for memory/native fonts. Unix overwrites retain file mode;
copied fonts retain source mode. Windows replacements inherit the folder ACL:
the portable Rust API does not preserve a previous file's custom DACL.

This is individual-file atomic replacement, not a transaction for a whole package
or durable power-loss recovery. A later storage failure can leave earlier completed
outputs. Directory checks and immediate rechecks reject existing redirects; they
are not capability-based protection against a concurrent adversary renaming parent
directories between filesystem calls. General native Save still uses its existing
write path. IDML import/resource bounds and #155's guide/work-count changes remain
separate tasks. No vendor font activation validation is implied.

Source: selected package portion of upstream #155 by @skylartaylor at
`9809a9b386b9254c18e847a0771cf1eb9b970acf`; see PAGENOR_UPSTREAM.md.
