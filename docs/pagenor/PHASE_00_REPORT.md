# Phase 00 — local baseline and audit

Date: 2026-10-08. Status: **in progress; local quality gate passes**.
See [phase tracker](../../PHASE_STATUS.md) and [issue register](ISSUES.md).

## Source and environment

- Repository: local Pagenor working tree, based on
  `f04606900f85d0b9e9e25c4d9ba78f055d58f49f`.
- Validation includes the pending font-discovery and supplied-branding changes,
  plus the baseline corrections in this report. These changes are uncommitted.
- Roadmap source: `Pagenor_Development_Phases.docx`, supplied by Appaji Ambarisha;
  SHA-256 `575be177a2d5b189c1ddaf9c7bbb6c06b3b6bb5bfd69a962a79b52eab5c72743`.
- Host: macOS 27.0.1, build 26A434, Apple silicon.
- Compiler: `rustc 1.99.0 (b940084d7 2026-09-28)`.
- Lockfile SHA-256:
  `48977247d46cb86e3a3862fc8824d3196be243af0e78049bab5d6434593df2bc`.
- Workspace: 20 packages; existing dependency layers retained. All packages
  inherit `unsafe_code = "forbid"`. No unsafe exception was introduced.

## Changes

The first quality run failed at strict Clippy on two constant-size slice loops
in PSD decoding. Fixed-size array chunks preserve their previous remainder handling.
Two independently reproduced test failures were stale rename assumptions: the RTL
test selected the brand’s P instead of the PDF run, and the render test required
more outlined glyphs than its current sample text contains. Production bidi,
composition and rendering behaviour were retained.

The tooling crate now inherits workspace lints, closing a policy gap found during
the final audit; every workspace package enforces the unsafe-code prohibition.
The quality tool now checks all workspace targets, uses the lockfile for build,
Clippy, test and WASM checks, and includes the actual web app in the WASM set.
New CI runs the gate on macOS, Windows and Linux, with a separate Rust 1.95 job.
Rust 1.95 is the declared minimum because the locked egui 0.36.2 dependencies
declare that requirement; all 20 packages now inherit it. Local verification used
1.99.0, so minimum-version support still needs the new CI job’s result.

Windows metadata now credits Appaji Ambarisha and uses Pagenor identifiers.
The MSI upgrade/component UUIDs are independent of the inherited installer’s UUIDs.
They are fixed values derived from Pagenor’s app identity, not regenerated per build.
Previously built installers using inherited UUIDs need separate migration review.
Linux metadata describes publishing and omits unapproved upstream contact links.
macOS’s native document type declaration matches `.pagenor` and
`application/vnd.pagenor+zip`. Full native install/file-open checks remain pending.
README promotions were removed, and artwork policy now reflects the owner’s reserved
branding rights. Software licences and original notices were preserved.

The packaging-lint download now verifies the pinned actionlint 1.7.12 Linux archive
against SHA-256 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`
before extraction. Other release inputs still need review under PG-0003.

## Validation evidence

Commands ran from the repository root unless stated otherwise. Full stdout/stderr
logs were retained in the local review bundle, including failed baseline runs.

| Command | Result |
|---|---|
| `cargo xtask ci` (initial baseline) | Exit 1: Clippy rejected two constant-size `chunks_exact` loops. |
| `cargo test -p pagenor-ui-egui isolates_arabic_names_from_latin_brand_and_zoom_suffix --locked` (before correction) | Exit 101: RTL-order assertion selected the wrong P. |
| `cargo test -p pagenor-render renders_page_with_text_and_fill --locked` (before correction) | Exit 101: stale `glyphs >= 15` assertion. |
| `cargo run --locked --package xtask -- ci` (final gate) | Exit 0: all seven steps passed. 785 tests passed, 0 failed, 3 ignored. |
| `cargo test --locked -p pagenor-fonts host_registry_fonts_are_cataloged_without_loading -- --nocapture` | Exit 0: every readable registry family/style reaches the lazy catalogue on this host. Core Text reports 207 files and adds 18 PingFang families beyond directory-only scanning. |
| `actionlint -shellcheck=` using verified actionlint 1.7.12 | Exit 0: all workflows parse and validate. ShellCheck was unavailable locally and is disabled in this command; packaging CI still runs it. |
| `git diff --check` | Exit 0 after removing an extra README end-of-file blank line. |
| `python3 audit_checks.py` (local review script) | Exit 0: legal/SVG byte comparisons, plist/XML parsing, edited shell-script syntax and workspace lint/MSRV inheritance. |

The final gate expands to:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
asset-attribution check
crate-layer check
cargo check --locked --target wasm32-unknown-unknown -p <package>
```

Exact final output:

```text
OK: 20 crates, no layering violations.
CI summary: all 7 steps passed (fmt, check, clippy, test, assets, layers, wasm)
```

All 16 WASM packages pass, including `pagenor-ui-egui` and `pagenor-web`.
WASM checking emits an existing unused `path` warning in the script-loading panel;
native strict Clippy passes. The three intentionally ignored tests are the large
story performance benchmark, text-heavy render wall-clock budget and cached RTL
wrapping benchmark. This run does not claim their performance budgets pass.

`NOTICE`, `LICENSE-MIT`, `LICENSE-APACHE` and `PAGENOR_UPSTREAM.md` are byte-for-byte
unchanged from HEAD. All nine original SVGs match their supplied ZIP contents.
The dependency-metadata inventory contains 559 packages across all resolved
targets/features, including workspace packages; this is evidence for review,
not a completed licence audit or release SBOM.

## Remaining exit-gate work

Updated 2026-10-08: the owner repository `appajid/pagenor` is connected and the
source history is uploaded. Help/MCP links use it. CI run 37770434311 passed
macOS, Linux and Rust 1.95; packaging 37770434230 and FreeBSD 37770434210 passed.
Windows failed a native-path text assertion, corrected in P01.2 and awaiting a
rerun. Private-repository branch protection requires a plan/visibility decision.
Signing/release permissions, immutable executable inputs, complete notices/SBOM,
source revision in artifacts and reproducible release builds remain unverified.

Phase 01 diagnostics, rescan and automatic refresh are implemented. Native
Windows/Linux adapters are in P01.2; their native comparison reruns and macOS
activation lifecycle evidence remain required before the phase exit gate closes.
