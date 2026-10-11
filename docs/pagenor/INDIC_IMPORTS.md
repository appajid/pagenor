# Indic text in IDML and INDD — P11.5

Local changes on 2026-10-11, based on Pagenor 0.5.2. This is an import correction,
not a claim of full INDD compatibility or completion of the Indic Publishing Engine.

The existing Rust importers and shared font/composition architecture are retained.
No plugin runtime, unsafe code, new font assets or upstream branding is introduced.

## Changes

- Shared language resolution recognizes Telugu, Tamil, Kannada, Malayalam, Hindi,
  Bengali/Bangla, Gujarati, Odia/Oriya, Punjabi/Panjabi, Marathi, Sanskrit, Assamese,
  Nepali, Konkani, Kashmiri, Sindhi and Sinhala/Sinhalese names, including IDML `$ID/`
  labels. Existing locale-code support remains. Unknown labels are not guessed.
- When glyph fallback is enabled (or an unavailable family invokes the existing
  substitute behavior), Indic clusters use one face that covers all their visible
  characters, including vowel signs and viramas. Joiners and variation selectors
  do not require standalone glyphs. Virama-linked continuations are kept together
  even where extended Unicode grapheme boundaries split them, such as Kannada.
- Native fallback checks whole-cluster coverage; a file with only partial coverage
  is not retained as the selected fallback. Small misses are cached within bounds
  and invalidated on font refresh. Loaded fonts, document scopes and the lazy
  catalogue remain in use; displaying a menu does not load every font's data.
- INDD wide text is decoded without silently replacing malformed UTF-16. Consecutive
  wide chunks are decoded together so a surrogate pair can span a chunk boundary.
  Character counts are bounded by their chunk records. Invalid/truncated text and
  font-name strings return errors instead of silently substituting content/fonts.
- INDD style-run lengths are mapped from UTF-16 code units to scalar positions for
  writing IDML, preventing supplementary characters from shifting later Indic styles.
  A boundary inside a surrogate pair returns an error. Reader allocations are capped
  before accumulating wide story text; the existing model/output budgets remain.

## Regression coverage

Eleven new tests cover language labels; partial-versus-complete font coverage;
native fallback refresh; Telugu/Hindi/Kannada/Malayalam composition with fallback
enabled and disabled; independent IDML text/font/style/language round trips across
nine Indic languages; wide INDD strings and every chunk split (including inside an
emoji surrogate pair); truncation; corrupt font metadata; UTF-16 style boundaries;
and engine open, native save/reopen and PDF ToUnicode export for both formats.

The engine PDF regression uses distinct synthetic glyphs and a ToUnicode-only
extractor. Joiners are separately checked in import round trips; invisible glyph
substitutions can require PDF ActualText, which that extractor does not interpret.
Synthetic fonts test routing and preservation, not the visual design of Indic glyphs.

## Validation and remaining limits

All seven `cargo xtask ci` gates pass: formatting, workspace/all-target checks,
strict Clippy, tests, assets, layers and WASM. **1,889 tests pass, six existing
ignores, zero failures; all 20 WASM packages check successfully.** `git diff --check`
also passes. Eleven new regressions are included in that total.

A separate local Rust probe opens synthetic IDML and INDD files using the installed
Kohinoor Telugu Regular face. Both preserve the exact Unicode source text through
native save/reopen, compose without missing glyphs or overset, and produce PNG/PDF
output. Both PNGs were inspected: Telugu vowel signs/conjuncts and joiner variants
have matching text appearance (the INDD fixture additionally contains its sample
red rectangle). `pdftotext -layout` extracts the exact original three-line Unicode
text from both PDFs, including ZWJ/ZWNJ. No font files were added or redistributed.
This native probe is separate from the synthetic-glyph regression tests.

The fonts, composition, IDML and INDD crates also pass cross-target checks for
`x86_64-pc-windows-gnu` and `x86_64-unknown-linux-gnu`. These are compile checks;
native GUI/file-opening/font-provider execution on those hosts remains unverified.
A separate macOS ARM64 release build was staged with build identity
`local-0.5.2-indic-imports`, verified with strict/deep ad-hoc code-signature checks
and `--version`. App/document artwork and document associations are preserved.
It is available for review without installing over or launching into the current
session. Version remains 0.5.2 until the next combined milestone; nothing was pushed.

Actual INDD files from multiple source versions, localized binary
metadata, Indic styling across differing font/style runs, document-font packages
and broader script-specific typography remain corpus validation work. INDD retains
its existing subset warning, including incomplete threading, master items, tables,
effects and other documented options. The binary reader does not yet resolve all
source language records; scripts are detected by the shared shaper, not by guessing
a language from the characters.

PageMaker/Anu legacy encoding work is separate and awaits the owner's font/sample.
Affinity and QuarkXPress readers are not implemented by this correction. Existing
documents and the owner's running application session are not replaced or closed.
