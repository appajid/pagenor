# Indic regression corpus

This directory is the seed corpus for Pagenor's Indic Publishing Engine.

The baseline file is only a smoke-test corpus. Before changing composition behaviour,
expand each script into tests for:
- conjuncts / virama or halant behaviour
- pre-base, post-base, above and below signs
- combining marks
- ZWJ / ZWNJ
- language-specific OpenType behaviour
- cursor movement, selection, Backspace and Delete
- line breaking and invisible break opportunities where appropriate
- justification
- font fallback
- threaded frames, tables and footnotes
- PDF selectable-text round trips
- IDML round trips where applicable
