# Master pages — P11.3, local Pagenor 0.5.2

Owner request: common objects, page numbers, headers, footers and guides repeat
throughout a document, with separate left and right masters for books.

## Using master pages

1. Open **Layout → Pages → Master Pages**, or the **Pages** panel.
2. The default **A-Master** is already assigned to every page of a new document.
   Facing-page documents have **Left Master** and **Right Master** buttons and
   live thumbnails. Click either side to edit it on the canvas with ordinary tools.
3. Place text, images or other objects there. They appear on every matching
   document page. Edit the master again to change all its unmodified instances.
4. While editing, use **Header…**, **Footer…**, or **Page Numbers** in the panel.
   Header/footer text and number frames are added to both sides; the dialog can
   target one side. Number frames contain the existing current-page-number
   field, not literal digits. Each page resolves its own section/page number.
   The frames are editable with normal formatting and positioning tools.
5. Use **Settings…** for shared margins and columns; choose Both, Left or Right.
   Alternatively, Layout → Margins and Columns acts on the master side in view.
   Drag guides from rulers or use Layout → Create Guides on the master. Inherited
   guides appear and snap on matching pages; change them on their source master.
6. **Back to document** returns to the document's preserved canvas view.
7. To create another master, click **New…**. Applying it throughout the document
   and using it for newly inserted pages is enabled by default. Its **⋯** menu
   also offers **Apply to all pages (default)** and **Apply to current page**.
   Drag a master label onto a document thumbnail to assign it to that page.
   Drag **[None]** to exclude a page from shared content.

Master content follows each page's geometric side, including right-to-left book
binding, even starting numbers and changes in page order. Single-page documents
use a single master. Page size, bleed and global grids remain document settings.
Explicit page-level margin/column edits remain local overrides until its master
is reapplied. Older saved documents retain their existing page settings: use
**Apply to all pages (default)** to adopt the shared master settings throughout.
Existing object override/detach commands remain available in the page context menu.

## Implementation

- Reuses the existing parent-spread, story composition, layer and override model;
  compatible `layout.parents.*` identifiers remain. No new subsystem, dependency
  or unsafe code. App/model changes are shared Rust for macOS, Windows/Linux and WASM.
- Master navigation selects the requested spread and side. Separate master/document
  views retain the document position and zoom. Only the selected master is highlighted.
  The bottom bar identifies master editing and offers a return action.
- Masters have live thumbnails and direct left/right controls. Layout guide commands
  and guide deletion target the master in view, instead of hard-coding master A.
- New/apply-master pages inherit margins and complete column definitions. A serialized
  flag preserves explicit local settings. Repagination resynchronizes the matching side.
  Applying a master throughout stores the default for insertion and document growth.
- Inherited guides stay on their masters, are translated to document spread coordinates
  and share the same resolution logic for drawing and snapping. Based-on master chains
  use cycle detection and a 32-master limit. Layer guide visibility is respected.
  A master spread guide is inherited only within the corresponding page bounds.
- Native `.pgnr` stores masters, default master, settings inheritance and guides.
  IDML retains master content and custom default/settings-inheritance attributes;
  existing IDML guide interchange limitations remain outside this change.
- New command parameters reject unknown masters, duplicate prefixes, invalid page
  indices, side values, column counts, margins and gutter values.

## Validation

All seven `cargo xtask ci` gates pass: **1,875 tests passed, 6 ignored**, with
20 WASM packages checked. All 12 packaging contracts pass. The separate macOS
arm64 0.5.2 bundle reports the expected version/build and passes strict/deep
ad-hoc code-signature verification. Actual egui output was rendered and visually
inspected for the master pair, settings dialog, inherited guide and numbered
pages 2–3. Regression coverage includes side matching after insertion/binding changes, undo, explicit
local overrides, default-master persistence, native/IDML master-content round trips,
per-page composed numbers, bounded inherited guides, guide layer visibility and
actual egui master selection/dialog routing. Your open development session is kept
running; the new bundle is built separately. No GitHub publication is part of this
local update, and native Windows/Linux runtime validation remains milestone work.
