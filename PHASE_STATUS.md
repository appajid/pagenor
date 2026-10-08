# Pagenor development phases

Updated: 2026-10-08. Source: Appaji Ambarisha’s `Pagenor_Development_Phases.docx`.
This tracker records verified work against the supplied roadmap. Existing features
in `ROADMAP.md` are implementation history, not proof that a phase exit gate passes.

| Phase | Priority | Work | Status / evidence |
|---|---|---|---|
| 00 | P0 | Repository, security and provenance | In progress. Local quality baseline and CI setup: [report](docs/pagenor/PHASE_00_REPORT.md). Own repository and local remote are configured. Initial CI passed macOS and Rust 1.95 on Linux; Windows quality remains pending; Linux hit a runner disk limit and requires a rerun with reduced debug symbols. Auxiliary workflow fixes await reruns. Private-repository branch protection and release supply-chain checks remain open. |
| 01 | P0 | Font discovery and catalogue | In progress. Core Text paths, all SFNT extensions, lazy named instances and duplicate handling implemented. Host comparison covered by tests; Diagnostics/rescan commands and automatic native refresh are implemented and pass the local gate. Font Book activation lifecycle and native Windows/Linux adapter validation remain open. |
| 02 | P0 | Clipboard, input and document reliability | Pending exit-gate validation. Unicode paste fixes exist; external application, undo, autosave and recovery matrix still required. |
| 03 | P0 | Indic corpus and font presets | Planned. Requires stable font discovery, versioned fixtures and font licence/size review. |
| 04 | P0 | Script-aware fallback and composite fonts | Planned. Reuse existing composition and document architecture. |
| 05 | P0 | Indic editing and shaping | Planned. Cluster-safe editing and independent shaping comparisons required. |
| 06 | P1 | Indic composition and language support | Planned. |
| 07 | P1 | Professional typography and page tools | Planned. Validate existing features before adding or replacing them. |
| 08 | P1 | Import, export and print | Planned. |
| 09 | P1 | UI, accessibility and performance | Planned. |
| 10 | P1 | Cross-platform distribution | Planned. Native macOS, Windows and Linux CI starts in Phase 00; packaging and signing gates remain here. |
| 11 | P1 | Public beta and 1.0 | Planned. |
| 12 | P2 | Post-1.0 improvements | Planned. |

Milestones: M0 = 00–02; Indic Alpha = 03–05; Publishing Alpha = 06–08;
Beta = 09–10; 1.0 = 11. No milestone is complete yet.

Next: finish Phase 00 ownership/release checks and Phase 01 diagnostic/rescan work,
then validate the Phase 02 reliability matrix before starting the Indic corpus.
Open findings and their completion evidence belong in [the issue register](docs/pagenor/ISSUES.md).

All platforms retain the shared Rust core and `unsafe_code = "forbid"`.
New platform discovery belongs behind native adapters; document font scopes,
family/style grouping and lazy font data must remain intact. WASM remains a build gate.
