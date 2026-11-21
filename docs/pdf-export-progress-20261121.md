# PDF export progress — P11.31 (21 November 2026)

Local work after the published 0.15.0 release; no installer or public release is replaced.

## Behavior

Every native interactive PDF export shows a progress window, including one-page
and small documents. Preparing, actual output-page counts, finishing and saving
are distinct stages. Page totals follow the selected range or spreads. Generation
and the production desktop atomic writer run off the interface thread. The
snapshot retains document fonts and shares immutable document/cache data; it does
not copy PDF bytes through base64 merely to save them.

The progress window closes after successful saving and is replaced by **PDF
export completed.**, with the path, page count and any original export warnings.
Only successful jobs persist PDF options. A failed export restores its options
and error; cancellation returns to its options without claiming success. Cancelling
the save chooser leaves options open and creates no job.

Cancel is cooperative between preprocessing stages and output pages. A page already
being drawn finishes first; subsetting/serialization may finish before cancellation
is observed. Cancellation and saving claim one atomic boundary: an accepted cancel
never calls the writer or overwrites an existing output. Saving cannot be cancelled.
A second interactive job is refused while the first remains pending. Normal Quit
and window close are refused with an explanatory status while exporting; the
existing recovery and unsaved-document path remains in place after the job ends.

A progress frame is painted before completion, with a 200 ms minimum window
visibility so fast exports also show feedback. This does not delay actual generation
or the native background save. Custom hosts without a background writer save their
completed bytes through the existing UI-thread writer. Browser downloads retain
the synchronous backend and receive completion feedback; this change does not
claim browser background generation. Explicit-path automation and engine/CLI
exports retain synchronous result fields unless the UI caller opts into
`background: true`. `ui.inspect.pdfExport` exposes pending/terminal state.

## Upstream review

Open and closed export proposals were checked online on 21 November 2026. Original
MIT OR Apache-2.0 notices/licences remain applicable. Selected behavior is adapted
to Pagenor's existing session, file chooser, fonts, writer and dialog architecture.

| PR | State checked | Author / inspected head | Result |
|---|---|---|---|
| [#1106](https://github.com/storytold/designcraft/pull/1106) | Open | @ForestParks / `608e10665afaceba656769caf136199aa62ddb6f` | Adapt snapshot generation, retained font scopes, raw shared engine report, background atomic saving and cancellation/write boundary. Do not take unrelated GPU, input, font-menu or app-wide command blocking changes. |
| [#2990](https://github.com/storytold/designcraft/pull/2990) | Open | @Pazaak / `1baa5ac4c1a0a484a2263e622980d963f653273f` | Adapt real page progress and cooperative cancellation checkpoints. The multi-job Exports panel and broad mutation-lock registration are outside this requested progress-window change. |
| [#1927](https://github.com/storytold/designcraft/pull/1927) | Merged | @Silvertree2010 / `1db1635a8831d2a890fd09f13fd4699b6eaf0c45` | Separate follow-up: reuse compositions in long-document PDF exports. No performance percentage adopted without local measurement. |
| [#2795](https://github.com/storytold/designcraft/pull/2795) | Open | @AlPaka1234 / `d49b223b677bf9330ff9e51ad9765a99dbfd21f1` | Separate preset/transparency follow-up. |
| [#2824](https://github.com/storytold/designcraft/pull/2824) | Open | @jayzed80 / `20422e8e41149aff85d9a2c4ea9e7514a3c59a3f` | Separate interactive PDF options and spreads dialog follow-up. |
| [#2989](https://github.com/storytold/designcraft/pull/2989) | Open | @MMexicann / `94d340a1227f1b5cb10f5dac0dc1db81c9cba078` | Separate Unicode batch-filename bounds follow-up. |
| [#3073](https://github.com/storytold/designcraft/pull/3073) | Merged | @dylanmtaylor / `402df07161cae8e440fa41174b3da65f3b9dff5c` | New since the previous review: JPEG output guards and image-probe tests; separate raster-export follow-up. |

## Verification

Regression coverage checks one-page/long export feedback, selected ranges/spreads,
PDF byte equality without progress, cancellation between sheets, cancellation
preserving existing output, actual native atomic writing on another thread,
refused late cancellation, source-document closure, retained options on failures
or worker panics, and explicit-path synchronous compatibility.

All seven workspace gates pass: **3,223 tests passed, zero failures, seven existing
ignores; twenty WASM packages**. Fifteen focused export checks cover the new PDF
progress and native UI lifecycle, including the existing options/persistence tests.
The native save-answer regression was also run with the original missing dialog
context restored temporarily: it failed because the answer waited behind its own
options window. Restoring the fix makes it pass. The exact fix was restored before
the final full gate.

Two offscreen views were rendered and visually inspected: actual progress-window
layout with a controlled seven-of-48-page state, and its replacement completion
alert with warnings. These controlled UI snapshots are not a document-fidelity or
performance benchmark. Real synthetic PDF generation, background atomic writing,
existing-file preservation and snapshot closure are covered separately by tests.

The current Mac app/session and published 0.15.0 installers remain unchanged.
This is local source work; no new release or installation is claimed. Windows/Linux
native runtime testing of this new UI change remains for the next build milestone.
