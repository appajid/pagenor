# Clickable Preflight status — P11.8

Local Pagenor 0.5.2 correction, 2026-10-11.

The bottom-bar Preflight count previously used a hover-only label and its
indicator used hover-only sensing. Clicking either now activates the live
Preflight panel through the existing `window.panel.activate` command. Opening
it repeatedly keeps it visible; opening it after Close restores it normally.
The panel already lists errors/warnings and lets users click an issue to select
its frame and go to its page. No document edits result from opening the panel.

Window → Output → Preflight now opens the same live panel. The headless
`preflight.run` query remains available to clients. The existing grouped route
Window → Panels → Publishing → Preflight and right panel icon still work.

Shared Rust UI change for macOS, Windows, Linux and WASM; no dependencies,
unsafe code, importer/font/composition changes or version bump. Owner documents
and the current running app are not closed or replaced by the local test build.

## Validation

Three UI interaction regressions pass: no-error status opens/reopens without
toggling closed, an error opens a clickable issue that selects its frame/page,
and the native Output menu opens the live panel. All seven `cargo xtask ci`
steps pass: formatting, check, Clippy, tests (1,905 passed / 6 ignored), assets,
layers and WASM (20 packages). The offline locked macOS release build succeeds.
The ad hoc signed bundle verifies and reports 0.5.2; application and document
icons are retained.

An isolated native test bundle starts with an empty recent-document list.
Native visual inspection shows the portfolio and its Preflight panel with
issue descriptions and page labels. Subsequent action attempts were rejected
by computer-use state guards because the app state changed concurrently;
no native click success is claimed. The automated interaction tests establish
the count-to-panel and issue-to-frame/page behavior. No existing app session
was quit, replaced or installed over.

The owner-source portfolio probe reports four errors: overset on pages 2, 10
and 22 plus missing Liberation Serif. Page 10 reports 17 characters; the other
two report zero characters at the story end. These diagnostics are shown as
reported, not suppressed. Clarifying zero-character report wording remains
follow-up work. The missing-font count may vary by document font context.
34 warnings, including cached-image resolution, are listed separately.
