# Pagenor development phases

Updated: 2026-10-09. Source: Appaji Ambarisha’s `Pagenor_Development_Phases.docx`.
This tracker records verified work against the supplied roadmap. Existing features
in `ROADMAP.md` are implementation history, not proof that a phase exit gate passes.

Owner workflow: make and test changes locally on macOS, keeping the shared code
compatible with Windows/Linux and WASM. Publish to GitHub and run the native
macOS/Windows/Linux matrix at a completed milestone; routine task commits stay
local. A task completion does not by itself complete a roadmap milestone.

| Phase | Priority | Work | Status / evidence |
|---|---|---|---|
| 00 | P0 | Repository, security and provenance | In progress. Local quality baseline and CI setup: [report](docs/pagenor/PHASE_00_REPORT.md). Own repository and local remote are configured. Remote CI passed macOS, Linux, Rust 1.95, packaging lint and FreeBSD. Windows exposed a path-separator assertion in the data-merge test; the assertion now compares paths and awaits its rerun. Private-repository branch protection and release supply-chain checks remain open. |
| 01 | P0 | Font discovery and catalogue | In progress. Core Text paths, all SFNT extensions, lazy named instances and duplicate handling implemented. Host comparison covered by tests; Diagnostics/rescan commands and automatic native refresh are implemented and pass the local gate. Windows registry and Linux/FreeBSD Fontconfig adapters are implemented (P01.2); cross-target compile and strict Clippy checks pass. Native Linux adapter tests passed. P01.3 removes stale loaded families/styles from the picker while preserving rendering/document handles. P01.4 fixes newly discovered style/instance resolution and supplements Core Text collections with PostScript-name discovery; hidden-path/extensionless-file coverage passes locally. P01.5 adds lazy Core Text table access for native fonts without readable files, with withdrawal/reactivation cache tests. P01.6 fixes lazy replacement of loaded installed files, including collection/style changes, identical-byte reuse and stale loader rejection. Actual Adobe/dongle lifecycle and Windows/Linux memory-font adapters remain open. Cross-platform native validation is deferred to the next milestone; Font Book activation lifecycle remains open. |
| 02 | P0 | Clipboard, input and document reliability | Pending exit-gate validation. Unicode paste fixes exist; external application, undo, autosave and recovery matrix still required. |
| 03 | P0 | Indic corpus and font presets | Planned. Requires stable font discovery, versioned fixtures and font licence/size review. |
| 04 | P0 | Script-aware fallback and composite fonts | Planned. Reuse existing composition and document architecture. |
| 05 | P0 | Indic editing and shaping | Planned. Cluster-safe editing and independent shaping comparisons required. |
| 06 | P1 | Indic composition and language support | Planned. |
| 07 | P1 | Professional typography and page tools | Planned. Validate existing features before adding or replacing them. |
| 08 | P1 | Import, export and print | Planned. |
| 09 | P1 | UI, accessibility and performance | Planned. |
| 10 | P1 | Cross-platform distribution | Planned. Native macOS, Windows and Linux CI starts in Phase 00; Windows ARM64 unsigned MSI build/install/architecture/CLI-version/uninstall smoke passed in run 37776034034; packaging breadth, GUI and signing gates remain here. |
| 11 | P1 | Public beta and 1.0 | Planned. |
| 12 | P2 | Post-1.0 improvements | Planned. |

Milestones: M0 = 00–02; Indic Alpha = 03–05; Publishing Alpha = 06–08;
Beta = 09–10; 1.0 = 11. No milestone is complete yet.

Next: complete local macOS activation evidence, validate host deactivation
behaviour, and finish Phase 00 release checks. Run the native platform matrix
when publishing the next completed milestone. Then validate the
Phase 02 reliability matrix before starting the Indic corpus.
Open findings and their completion evidence belong in [the issue register](docs/pagenor/ISSUES.md).

All platforms retain the shared Rust core and `unsafe_code = "forbid"`.
New platform discovery belongs behind native adapters; document font scopes,
family/style grouping and lazy font data must remain intact. WASM remains a build gate.
