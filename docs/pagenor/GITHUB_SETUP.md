# GitHub repository setup

Updated: 2026-10-08.

Pagenor’s owner-supplied repository is
[appajid/pagenor](https://github.com/appajid/pagenor). It is private.
The authenticated account has administrator access.

- Local `origin` uses `https://github.com/appajid/pagenor.git`.
- The former source remote is retained as `upstream` for provenance, with its push
  URL disabled. Local application history is retained.
- GitHub CLI 2.102.0 was downloaded to the local tooling directory and verified
  against the published SHA-256 checksum. Its existing `appajid` account login
  supplies credentials through a repository-local Git helper. No new browser
  authorisation was needed and no token was added to repository files.
- The two existing GitHub README commits are preserved when importing the local
  application history. The complete application is required before CI can run.
- Workspace packages inherit the owner repository URL. Help, CLI and MCP use its
  source and issue links. Unconfigured community links return an error rather
  than opening a different project’s community. Project Help works without a document.

Local validation: `cargo run --locked --package xtask -- ci` passes all seven
steps, with 786 tests passed, none failed and three performance tests ignored.
A CPU-rendered view of the actual Help menu was inspected because the headless
GPU screenshot adapter is unavailable on this host.

[Quality CI](https://github.com/appajid/pagenor/actions/workflows/ci.yml) runs
macOS, Windows and Linux checks, including WASM, plus Rust 1.95 on Linux.
Remote results are pending until the source upload and initial runs complete.
Release verification remains a separate Phase 00/10 gate.

`main` is currently unprotected. GitHub’s ruleset endpoint returned HTTP 403
requiring GitHub Pro or public visibility for this private repository; its
branch-protection endpoint confirms no protection is installed. Visibility was
preserved. This remains PG-0010 in [the issue register](ISSUES.md).

The original Phase 00 report records the earlier local baseline. This follow-up
records the newly supplied repository and ownership/link configuration.
