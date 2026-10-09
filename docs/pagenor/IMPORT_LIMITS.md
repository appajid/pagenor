# Native and Office import limits — P00.3

Pagenor checks ZIP end records and the central directory before asking the ZIP
reader to index native `.pgnr`/legacy `.pagenor`, DOCX or XLSX input. This check
does not allocate a directory. It understands classic ZIP and ZIP64 records,
rejects multi-disk/inconsistent directories, and bounds directory size/count.
The importer then checks declared entry sizes and reads relevant parts with caps.
Duplicate ZIP names and multiple files for the same native asset are errors.

These initial limits follow the reviewed upstream hardening policy. They are
explicit compatibility limits, not the native file format version: old valid
files within the limits remain readable. Large production jobs exceeding them
need a future configurable/streaming policy before the corresponding milestone.

| Resource | Native documents | DOCX/XLSX |
|---|---|---|
| Input archive | 512 MiB | 512 MiB |
| ZIP entries | 4,096 | 4,096 |
| Central directory | 16 MiB | 16 MiB |
| Declared single expanded entry | 256 MiB | 128 MiB |
| Declared total expanded entries | 512 MiB | 512 MiB |
| Relevant JSON/XML part | Document JSON: 64 MiB; metadata: 1 MiB | XML: 64 MiB each |
| Legacy single JSON input | 64 MiB (including legacy embedded asset text) | — |
| Additional output limits | Actual native part reads cumulatively limited to 512 MiB | Imported text: 64 MiB; items: 100,000 |

Office XML parsing caps nesting at 256 elements, one million events and 500,000
nodes per part. It rejects incomplete/multiple-root input. Word import charges
paragraphs/runs, note expansion, padded rectangular table cells and retained
styles against cumulative output budgets. Style-parent cycles use a visited set.
Excel shared-string expansion and materialized grids share output budgets.

Native saves check JSON/asset sizes, entry count, total expanded size and final
archive structure/size against matching reader limits. `cmd::to_bytes` and
`cmd::file_bytes` return `Result<Vec<u8>>`; callers must handle refusal rather
than writing an empty buffer. Save, Save As, Save a Copy, Serialize, snippets,
book updates and packages propagate serialization errors. Save commits relative
data-source paths and saved-state markers only after successful serialization
and writing. Recovery serialization failure preserves the last valid owned
checkpoint. Existing `.pgnr` suffix handling, explicit paths and legacy names
are retained.

This is a focused hardening batch. Hosts still read input bytes into memory
before invoking these parsers. Native JSON serialization occurs before its
size is checked. This is not streaming or a complete XML/ZIP security audit.
Atomic replacement on actual storage failure, package path/rollback hardening
(#155), IDML ZIP/XML/resource bounds (#156) and native platform execution are
separate tasks. IDML receives the quick-xml API compatibility change only.
Font discovery/loading and vendor activation behavior are unchanged.
