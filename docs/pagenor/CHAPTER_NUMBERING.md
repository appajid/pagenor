# Document and book chapter numbering — P07.2

Adapted upstream #341 at `498e3d401c8214d3f617c59af0893a2eaf62dfd1`.
Pagenor remains 0.3.0; changes are local. This adds the command/model foundation;
the dedicated Numbering & Section Options window follows in #342.

`layout.chapterNumbering {}` reports the chapter number, formatted label, style,
source and the name of the open book containing the document (or null). Edits
support Arabic, Roman, alphabetic and two/three/four-digit Arabic styles, and
Automatic, User Defined or Same As Previous sources. A start alone selects User
Defined. Changes are undoable. Chapter Number variables and chapter cross-reference
values use the formatted label. New fields have defaults when reading older files;
no native format-version bump is required.

`book.paginate {}` resolves chapter numbers in book order and continues page
numbering. Same As Previous uses the preceding number in the current document's
style; the first document starts at 1 unless explicitly numbered. Save any modified
open book documents first. Successful updates synchronize clean open documents,
preserving their font scopes, selections and undo history. This prevents a later
Save from restoring stale numbers. All inputs are checked before the first write;
duplicate paths/aliases, missing documents, excessive chapter numbers and IDML
inputs are refused. Save IDML documents as .pgnr first: chapter-number IDML mapping
is unverified. Individual file replacement uses the existing metadata-preserving
safe writer. A later OS write failure can still leave earlier book documents
updated; multi-file transactions are not implemented.

Section prefixes allow eight Unicode scalar characters and exclude plus and comma.
Symbols cannot number pages or chapters. Partial section updates retain existing
settings, and invalid edits leave the document and undo history unchanged. Command
numbers are bounded before conversion; loaded chapter labels are bounded as well.

Regression coverage includes all chapter styles and native serialization, legacy
field defaults, variable/cross-reference labels, rejected inputs and undo, the
five-document sequence I/1/2/g/8, clean open-document synchronization, dirty-document
and missing-file refusal, aliases, and preservation of partial section settings.
Native Windows/Linux execution remains a milestone check.

Local gate: all seven steps pass; 1,507 tests passed, 3 ignored; 20 WASM packages.
