# Numbering & Section Options — P09.13

Adapted upstream #342 at `9c683f3ef864cb705db3ae92f480d10684acfce7`,
using the chapter-number foundation from #341. Version remains 0.3.0; work stays local.

Layout → Numbering & Section Options opens the section for the page in view.
Page 1 always starts a section. Other pages can start/remove sections and choose
automatic or explicit page numbering, prefix, marker, style and prefix inclusion.
The chapter group offers the eight styles, Automatic/User Defined/Same As Previous
sources, and the current book's name. Same As Previous is disabled outside an open
book. OK and Cancel remain alongside the scrolling options.

Pagenor applies both parts through `layout.numberingSection`, using the existing
section/chapter commands to prepare a document in an isolated session, then
committing once. Refused values keep the dialog open and leave live document/history
untouched. One Undo restores both parts. Opening reads chapter settings directly
rather than executing an edit command, preserving a Data Merge preview.

Three added UI tests cover open/apply/remove, invalid page/chapter inputs including
invalid second-part styles and numbers, one-step undo/redo, and visible/clickable
actions at normal/small viewport sizes in English and RTL layouts. Existing popup
Enter/Escape guards and all New Document/footer behaviors are retained. Shared
translations are adapted; Italian remains deferred at the owner's request.

All seven local quality gates pass: 1,510 tests, 3 ignored; 20 WASM packages.
The actual egui window was rasterized and inspected in English and an RTL layout.
The scratch renderer's Arabic font fallback does not provide Arabic glyphs; this
inspection establishes geometry and action placement, not Arabic typography.
Native macOS interaction will be checked with the rebuilt combined app. Native
Windows/Linux execution remains a milestone check.
