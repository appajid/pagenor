# Composite fonts and replacement

P04.1 adapts pinned upstream #126 and the final #129 branch. This is shared Rust
work for desktop and WASM. It does not complete the Indic or font-activation gates.

Composite components resolve through the active document's font scope. Generated
footnote labels use the component selected for each character while retaining one
text anchor. Nested note/cell objects and font diagnostics are included in rendering,
PDF export and preflight. Placeholder fonts do not satisfy glyph coverage.

Family aliases are read from bounded catalogue metadata. Static style aliases are
also catalogued; variable named instances retain their named style entries. Menu
and availability checks keep installed font bytes lazy. Alias lists retain at most
64 distinct names and 64 KiB of decoded text, shared across named instances. File
and native instance metadata are bounded before expanding repeated names. The
current catalogue determines menu/style availability. Retained file faces and their
aliases stay resolvable this session, without restoring removed picker entries. Existing document-local precedence and newly installed exact styles stay.

Opening a document with missing families or styles offers Find/Replace Font. The
screen distinguishes the original request, currently resolved face and suggested
replacement. Replace All uses the command-backed font replacement and remains
open for additional replacements. Done keeps the document's original requests and
continues with rendering fallbacks. Failed replacement remains visible. Text,
paragraph/character style definitions and composite entries are updated by default;
replacement, undo, catalogue changes and manual rescans refresh the displayed list
and its suggestion. Suggestions consider characters
used by the document and do not claim an absent original font is installed.

The retained Mojikumi/kinsoku repairs preserve the final upstream scope, including
nested preflight and finite IDML values. Withdrawn speculative preset/CID changes
are excluded. See [Mojikumi repair scope](../mojikumi.md) for supported limits.

The existing native adapters, .pgnr names, fallible serialization, clipboard and
recovery behavior remain. Dot-prefixed families stay hidden from menus, available
to backend lookup. Real Adobe/dongle activation validation remains pending at the
owner's request. Windows/Linux and browser execution belong to the milestone matrix.
