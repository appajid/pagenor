# Batched composition lookup — P00.7

The performance idea from upstream #172 is adapted locally at the pinned head
in [provenance](../../PAGENOR_UPSTREAM.md). On 2026-10-10 GitHub reports the request
closed, unmerged and renamed “withdrawn”; the author says it was opened in error.
The retained three-file proposal is independently reviewed and adapted rather
than treated as an accepted upstream release.

Every composition-cache read validates the story, styles, document font scope,
frame allocations and wrapping items. Previously it located each threaded frame
with a separate document walk, including when returning an unchanged cached
layout. `Document::find_many` now resolves the story's resident frame-ID slice
in one pre-order walk, stopping once all distinct IDs have been located. Empty
requests return immediately; one distinct ID uses direct ID comparisons.

Document spreads precede parent spreads. Nested groups retain their full index
paths; repeated IDs resolve once and absent IDs are omitted. For malformed duplicate
item IDs, the first match agrees with `find`; document validation still rejects
duplicates. Embedded frames in stories, notes and table cells continue through
Pagenor's existing anchored-frame cache signature. No font publication, cache
ownership/eviction, undo, recovery, native format or UI behavior is changed.

The source's recursive batch walker is replaced with heap-held iterator levels,
shared with single-item lookup. Traversal storage is bounded by the depth of the
already resident, acyclic `Arc<Item>` tree; request storage by the resident ID
slice. Returned paths add their own storage. There is no silent depth cut-off or
truncation of results. Other recursive document operations and recursive teardown
are outside this change's scope. No dependency or platform-specific API is added;
the existing Rust layers and `unsafe_code = "forbid"` remain.

This reduces frame location work from O(items × frame IDs) to
O(items + frame IDs + returned path lengths) per story. Reading every story still
walks the document once per story. Cache misses still compose through the existing
frame-specification path; this change does not add a persistent document index.
Anchored-frame fallback and wrapping scans also retain their existing costs.

## Local verification — 2026-10-10

Four new regressions cover nested/document/parent lookup, empty/missing/repeated
requests, malformed duplicate-ID first-match behavior, 2,048 nested groups on a
128 KiB thread stack, and actual cache reuse/invalidation for nested/parent geometry,
wraps, page keys, previous compositions and document font scopes. The existing
embedded-note frame regression also passes. Focused document/composition tests:
**233 passed, 1 existing ignored**. An isolated child using the previous
recursive locator on the same 2,048-group/128 KiB-stack fixture aborts with stack
overflow; the new single/batch lookup regression passes. The child uses synthetic
in-memory items and disables core dumps, touching no user document or app session.

A local standalone harness compares the previous Pagenor cache (`ef43f49`) and
its original recursive single-item locator against the adapted cache. External
dependencies are pinned to the repository lockfile. Both controls use identical
synthetic documents and build settings (development harness optimization 1,
dependency optimization 2); composed output is compared before timing. Each row
is the median of 11 alternating warm-cache runs. This measures synthetic cache
reads, not total application or export time.

| Pages | Frames/story | Stories read | Previous | Batched | Speedup |
|---|---|---|---|---|---|
| 100 | 50 | 20 | 2.538 ms | 0.375 ms | 6.77× |
| 1,000 | 50 | 200 | 333.221 ms | 28.524 ms | 11.68× |
| 1,000 | 1 | Last 100 | 9.996 ms | 8.711 ms | 1.15× |

The single-frame row samples the last 100 stories to stay below the existing cache
eviction threshold. No elapsed-time assertion is added to the test suite.
Windows GNU and Linux GNU production-library strict Clippy checks pass for
`pagenor-doc` and `pagenor-compose`; these are cross-compilation checks, not native
OS execution or full desktop cross-builds. Native platform execution remains a
milestone task. The macOS desktop executable builds successfully and reports `pagenor 0.2.1`.
The running app bundle is not replaced. The first full gate passed all 1,286 unit
tests but lost an AI doctest dependency artifact when a separate desktop build
changed features in the same target directory during rustdoc. The desktop build
was completed and the full gate rerun sequentially; no source workaround or
suppressed test was used. The final `cargo xtask ci` passes all seven gates:
**1,286 tests passed, 0 failed, 3 existing ignored; 20 WASM packages checked**.
Work remains local, with no GitHub push or milestone completion.
