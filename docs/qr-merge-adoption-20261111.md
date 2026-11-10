# Pagenor QR merge correction — 2026-11-11

Local follow-up **P11.26 / 0.14.1** to 0.14.0. Implemented and validated locally.

## Online review and source

The latest-created search of open and closed upstream pull requests finds one new
proposal number after the prior review cutoff, #2578: [#2581](https://github.com/storytold/designcraft/pull/2581).
The thirty most recently updated records were also screened; the newest number
remains #2581. The 11 November UTC update search returns no records at this check.
Older queued feature proposals remain separate from this focused correction.

Source: @autocraft-bot[bot], open/non-draft #2581, inspected head
`6414b19e5ffcf2211ebd92c58c3be8a2bd336ce1`, updated 10 November 2026 at 22:16:28 UTC.
The retained MIT OR Apache-2.0 licences and original copyright apply. This is an
adaptation to Pagenor's current commands, not a merge of the upstream branch.

## Behaviour and compatibility

A generated standalone QR code is a compound path. A QR merge fills frame content;
using a standalone QR path as the target retains the old path and therefore the
old visible code. Refuse that binding before recording a placeholder or changing
a label, with guidance to use an empty graphic frame.

Pagenor also binds fields through `object.qrCode`, independently of
`data.placeholder.add`. Apply the same target validation there, including calls
that create or edit a standalone code and bind a field in a single command. Its
existing transaction restores the document, selection, allocation and history on
failure. Empty-frame bindings and frames containing QR content remain supported;
hyperlink placeholders on QR paths remain supported.

No model/native format change, dependency update, font or icon change, interface
translation or CJK feature is needed. Native capability stays 8; `.pgnr` and
legacy `.pagenor` support stay intact. Shared Rust code covers native and web
frontends. This source-only change leaves the installed 0.14.0 app and current
user session untouched; no GitHub upload or release publication.

## Validation

Six generated-document regressions cover explicit/selected/inferred QR targets,
both command entry points, rejection atomicity, frame-content replacement for
three records, undo/redo and unchanged templates, empty-frame QR generation, and
hyperlink compatibility. The visible compound path of each merged code is also
compared with the generated path for that record's payload.

Before the fix: **four tests pass and two fail**, reproducing both command bugs.
After the fix: all six pass, including transaction, undo/redo and template checks.
Final `cargo xtask ci`: **all seven gates pass — 3,163 passed, zero failures,
seven existing skips; twenty WASM packages**. Formatting, all-target compilation,
Clippy with warnings denied, asset attribution and layer checks pass. No warnings in the workspace gates. Optional craft-fonts is unset, matching the previous source gates.

Windows GNU and Linux GNU compile checks pass for `pagenor-doc`, `pagenor-fonts`,
`pagenor-compose`, `pagenor-images`, `pagenor-format` and `pagenor-render`.
The full engine checks stop in the pre-existing `ring` C build because this Mac
lacks `x86_64-w64-mingw32-gcc` and `x86_64-linux-gnu-gcc`; this is a toolchain
limitation, not a successful full-platform build. The changed engine code is
compiled by the successful Mac and WASM gates. Native Windows/Linux execution
remains pending. No UI layout changes require new
screenshots; the tests exercise the same engine commands used by the UI/CLI/MCP.
Temporary target/profile files are removed after checks; compact before/after,
workspace and platform logs remain in the local Codex output report
`outputs/pagenor-pr-2581-20261111`. Original user documents/fonts and Vectnor are
untouched. The upstream head and newest proposal number were rechecked online and
are unchanged.
