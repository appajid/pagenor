# Pagenor upstream provenance

Pagenor is an independent derivative of DesignCraft.

Initial upstream DesignCraft revision:
`f4a4c9709b7c552800f15ba3d1f0f68dd75fa165`

Original DesignCraft copyright:
Copyright (c) 2026 ArtCraft Team and the DesignCraft contributors.

The upstream software is available under MIT OR Apache-2.0. Pagenor retains
the applicable original copyright, licence, attribution, and third-party
licence notices.

The ArtCraft and DesignCraft names are referenced here only to identify the
origin of the upstream work. Pagenor does not use ArtCraft trademarks as its
product identity.

## Reviewed upstream adaptations — P00.2 (2026-10-09)

The following MIT OR Apache-2.0 upstream contributions were adapted locally.
Pagenor retains the original licences and copyright notices. Author logins identify
source contributions and do not alter the generated contributor registry. Changes
use Pagenor product identifiers and preserve its current font/recovery architecture
and `.pgnr` naming. These are selected patches, not a whole-branch merge.

| PR | Source author | Inspected head revision | Adaptation |
|---|---|---|---|
| [#168](https://github.com/storytold/designcraft/pull/168) | @DWAA1660 | `412962d9f8eb5bb913d07254bca6ea2035a20137` | Windows DX12 default and explicit backend overrides; Pagenor troubleshooting. |
| [#152](https://github.com/storytold/designcraft/pull/152) | @skylartaylor | `6a53d95895e5a1de1ac2d95b6d3bd77508cab61d` | Separate export text/attribute/story/CSS contexts and normalize language/MIME metadata. |
| [#149](https://github.com/storytold/designcraft/pull/149) | @jahala | `c63ca54227d72a5a4e030ad028de1beb3f2d512d` | UTF-16 surrogate pairs in RTF/Tagged Text and round trips. |
| [#147](https://github.com/storytold/designcraft/pull/147) | @jahala | `1ed206642f4a0a55592d355107a03abf53b93678` | Validate deferred raster decodes before writing PDF; skip damaged images. |
| [#146](https://github.com/storytold/designcraft/pull/146) | @jahala | `c84eb56258af9908957f38ee2f2cb67ba709e1aa` | Unsaved-close modal for menu/shortcut/tab; current UID/recovery flow and .pgnr test path. |
| [#143](https://github.com/storytold/designcraft/pull/143) | @aiongg | `7d064f268e6893f814c91988c41432287b44a10d` | Font-menu portion only: search/favourites/keyboard shared across fields; dot-family filter is UI only. Existing task-bar position behavior retained. |
| [#137](https://github.com/storytold/designcraft/pull/137) | @jahala | `7c739feb6f7ad903fa7bc6e2132a68c1cc11f107` | Bound guide/column counts and checked step-repeat multiplication. |
| [#127](https://github.com/storytold/designcraft/pull/127) | @aiongg | `67c05eb22c40af59fd07b76939a2ac5c41191406` | Reject unused export options; clarify byte-offset/MCP contracts; preserve Pagenor usage and links. |
| [#120](https://github.com/storytold/designcraft/pull/120) | @skylartaylor | `bae5a92dbbf2731504e9ce2b639a0511fdcbc7da` | Preserve typed JSON values in generic command dialogs. |
| [#117](https://github.com/storytold/designcraft/pull/117) | @skylartaylor | `8662526736d8ee0c08ac61c472fac6fd5c8b317b` | Browser-safe Print, decoded startup flags and canonical MIME; retain .pgnr and legacy naming. |
| [#115](https://github.com/storytold/designcraft/pull/115) | @IHAGI-c | `36000e75f558da674b6f593af29b79cf6e565b93` | No-break hyphenation guard and regression. |
| [#111](https://github.com/storytold/designcraft/pull/111) | @aiongg | `ed3e8ba8c3921ae104cf3b2731650fa9fb89a825` | Forward macOS native Edit commands to focused fields; preserve document commands otherwise. |

## Bounded import/save adaptation — P00.3 (2026-10-09)

Adapted [upstream #154](https://github.com/storytold/designcraft/pull/154), by
@skylartaylor, at inspected head `9cb07a99641c432653a386e5f3e0eb3b8e88b160`.
All 14 source-file blob identities match the pinned head. The MIT OR Apache-2.0
licences and original copyright remain applicable.

The port adds the standalone L0 `pagenor-archive` preflight, native format and
DOCX/XLSX input/output budgets, linear bounded style inheritance and the shared
quick-xml 0.41 compatibility change. Pagenor preserves `.pgnr`/legacy reading,
current MIME, document-font scopes and owned recovery checkpoints.

Local additions reject multiple XML roots, budget padded Word-table cells,
reject duplicate ZIP entries/ambiguous asset IDs, account for actual cumulative
native reads and compare future format versions without narrowing. The engine
now propagates serialization errors through save/copy/serialize/snippet/book/
package callers; failed save preparation leaves live relative paths and saved
state unchanged. No package-path rewrite or IDML archive/resource confinement
from #155/#156 is implied. See [limits and scope](docs/pagenor/IMPORT_LIMITS.md).

## Package output adaptation — P00.4 (2026-10-09)

Adapted the package-output portion of [upstream #155](https://github.com/storytold/designcraft/pull/155),
by @skylartaylor, at inspected head `9809a9b386b9254c18e847a0771cf1eb9b970acf`.
The live PR head was verified and all six changed-file blob identities match the
pinned snapshot. Original MIT OR Apache-2.0 licences/copyright remain applicable.
The guide and descendant-aware step-repeat portions are not adopted by this task.

The port uses portable Unicode filenames, redirect checks and staged individual
file replacement. Pagenor retains `.pgnr`, fallible serialization, document font
scopes and native/memory font reporting. Local additions preflight required output
paths, prepare exports before file writes, check existing redirected ancestors
iteratively, preserve short extensions when truncating names and reserve suffix
space. Folder-wide rollback, hostile concurrent directory renaming and general
Save atomicity are not claimed. See [behavior and limits](docs/pagenor/PACKAGE_OUTPUT.md).

## Editor UI adaptation — P09.2 (2026-10-09)

Reviewed both open and closed upstream requests: 53 open and 61 closed in the
pinned snapshot. Of 58 merged closed requests, 56 merge revisions are ancestors
of Pagenor's initial revision; #132/#135 concern release packaging. The unmerged
#74/#50 are superseded. The closed #54 AI proposal is included in the remaining
feature integration scope, alongside open feature backends, at the owner's request.

The following original MIT OR Apache-2.0 contributions are adapted without
changing their licences or generated contributor registry:

| PR | Source author | Inspected head | Adaptation |
|---|---|---|---|
| [#112](https://github.com/storytold/designcraft/pull/112) | @aiongg | `9fec15876a674bd5b5c38ae00bfb0a7ec52c8561` | Responsive app/control/tools bars and accessibility labels. |
| [#142](https://github.com/storytold/designcraft/pull/142) | @jahala | `20422c47256b21338f05de5ead111166f01368bf` | Second control row remains visible. |
| [#143](https://github.com/storytold/designcraft/pull/143) | @aiongg | `7d064f268e6893f814c91988c41432287b44a10d` | Remaining contextual task-bar drag, pin and reset; existing shared font search retained. |
| [#145](https://github.com/storytold/designcraft/pull/145) | @aiongg | `63a1f03498910e74d0eab9bf9c2073f79e6f6469` | Ruler unit context menus and Properties language. |
| [#144](https://github.com/storytold/designcraft/pull/144) | @aiongg | `ca0ab6e55179b5c504a9a8626048e65f1cd1ed78` | Rich character-style options, inherited previews, sparse attributes and rename/rebase. |
| [#171](https://github.com/storytold/designcraft/pull/171) | @zydvc26s67-web | `3018c56e0c8428fead12d0e72e1de9160579dbd4` | Style delete UI and character deletion combined with #144's richer editor. |
| [#80](https://github.com/storytold/designcraft/pull/80) | @aiongg | `1ccf4fbfbe719bdd52b656284e9a83f617dc28c3` | Open/Place/drop error alerts. |
| [#119](https://github.com/storytold/designcraft/pull/119) | @skylartaylor | `637d05e776cbc0b699c16141b09cfc34a6fd71c6` | Import intent and stable target document identity across asynchronous delivery; PDF options retain bytes. |
| [#138](https://github.com/storytold/designcraft/pull/138) | @jahala | `9ec7858dc959ac47275ec27d4bc08f25ea3a8559` | Browser named-key filtering, narrowed locally to explicit key names to preserve ordinary capitalized text. |

Changed-file blob identities match all nine pinned heads. Pagenor preserves owner
branding, product URLs, `.pgnr` and legacy names, shared font search/scopes,
clipboard services and recovery ownership. Serialization remains fallible.
Local additions validate style replacements/paragraph renames, reject inheritance
cycles and rewrite nested text, defaults, note options, running headers, TOC/XML
references. Existing text traversal now includes endnotes. No remote push or
milestone completion is implied. Larger feature branches are tracked separately.

## Advanced data merge adaptation — P09.3 (2026-10-09)

Adapted [#109](https://github.com/storytold/designcraft/pull/109) and
[#125](https://github.com/storytold/designcraft/pull/125), both by @CameronGilroy,
at inspected heads `aff5c9492e62a06f277f5a8eb9627b19cbc4f259` and
`805ac702d5ab2ae5bcce3431eb9ab7245f9a2449`. Changed-file blobs match the pinned
snapshots. Original MIT OR Apache-2.0 licences and copyright remain applicable.
The dependent second proposal was compared against the first to avoid replaying it.

Multiple linked sources, JSON, source filters/sorts, record grids and the grid tool,
skipping warned records, affixes, joins, virtual fields, workbook values/formulas,
inline/remote images and merged PDF output are adapted. The panel exposes source,
filter/sort, field, record, warning and grid controls; joins, affixes and combined
sorting currently use commands, rather than additional dedicated screens.
Pagenor retains native names, document fonts, recovery and fallible serialization.
The unrelated PSD chunk optimization is omitted. Imported design documents record
source proposals and are not development instructions.

Local additions retain archive/output budgets for XLSX, limit repeated XLS strings
and padded grids, bound formula recursion/evaluation and grid planning, cap actual
file/HTTPS reads and reject URL request-line injection. Join keys are indexed while
preserving the first matching record. Details: [data merge scope](docs/pagenor/DATA_MERGE.md).

## Typography and layout adaptation — P07.1 (2026-10-09)

Adapted the dependent final branch #96, including #83/#84/#81/#106. All changed
blob identities match the pinned source heads. The original MIT OR Apache-2.0
licences and copyright remain applicable; generated contributor files are untouched.

| PR | Source author | Inspected head |
|---|---|---|
| [#83](https://github.com/storytold/designcraft/pull/83) | @aiongg | `673039f5dc7dac7804e482531d30d7a3ea328baa` |
| [#84](https://github.com/storytold/designcraft/pull/84) | @aiongg | `141a5306bf5f36a7e964e91efed5f4ff4033df43` |
| [#81](https://github.com/storytold/designcraft/pull/81) | @aiongg | `dc18c104485d2797e4846569f714b3ab8d844bef` |
| [#106](https://github.com/storytold/designcraft/pull/106) | @skylartaylor | `bed9438612e85f7d28799a6dff679ab6859c76b9` |
| [#96](https://github.com/storytold/designcraft/pull/96) | @denizkoekden | `38ec9ce7e04a3e61dd6f8d9f9be07889df6fc952` |

Drop caps/selection, first baselines, wrap/defaults, text threading/ports, snapping,
word/paragraph selection, font style linking, text/frame colour targeting, kerning
across format runs, baseline-based horizontal frame fitting and empty paragraph
mark formatting are adapted once. The inherited clipboard/control isolation change
is omitted to retain Pagenor's tested menu/control bridge and recovery behavior.

Format version 2 pins the former paragraph defaults when loading version-1/legacy
files. Pagenor retains bounded native reads and fallible serialization, u64 future
version comparisons, .pgnr names and document font precedence. The migration uses
indexed bounded style walks and includes endnotes. Current selection style traits
use scoped fonts and custom variable-axis values. No Indic phase exit is implied.

## Composite fonts and missing-font UI — P04.1 (2026-10-09)

Adapted the dependent final #129 branch, including #126, once. Source blob
identities match the pinned heads. Original MIT OR Apache-2.0 licences and
copyright remain applicable; contributor registry/profile files are untouched.

| PR | Source author | Inspected head |
|---|---|---|
| [#126](https://github.com/storytold/designcraft/pull/126) | @ShaoWenbinSaleh | `af43dd88c9fbd77c07309a24ee4a9d6345dc726a` |
| [#129](https://github.com/storytold/designcraft/pull/129) | @ShaoWenbinSaleh | `9f655739fab096e6d809f4687d403dbf28cf4ac1` |

Composite resolution/generated labels, nested objects/preflight, localized aliases,
replacement suggestions and repeated Find/Replace Font controls are adapted to
Pagenor's scoped lazy catalogue and typography pipeline. Alias retention is bounded
and shared across instances; removed faces do not regain picker entries. The exact-style
refresh invariant and document font precedence remain. The final Mojikumi repair
scope is retained; withdrawn broad preset/CID changes are excluded. Imported scope
documents describe source proposals, not development instructions. Details:
[font replacement scope](docs/pagenor/FONT_REPLACEMENT.md). No Indic phase exit or
Adobe/dongle validation is implied.

## Table direction and control contracts — P09.4 (2026-10-09)

Adapted the following original MIT OR Apache-2.0 contributions locally, preserving
licences, copyright, Pagenor identifiers and generated contributor metadata:

| PR | Source author | Inspected head | Adaptation |
|---|---|---|---|
| [#128](https://github.com/storytold/designcraft/pull/128) | @aiongg | `0f52f3ba89b1d48e31a157c2b5b49d66e29b136b` | RTL table defaults/panel, new-story binding direction and PDF/A raster interpolation. |
| [#118](https://github.com/storytold/designcraft/pull/118) | @skylartaylor | `e9cc4b49eca854dd0101cca48f8a9017d27dfd1a` | Explicit-caret typing, missing-story errors and strict render/export page contracts. |

Pagenor retains recovery-aware control quit and damaged-raster prevalidation;
import targets, document font scopes and ordinary clipboard/keyboard routing stay.
See [behavior and limits](docs/pagenor/TABLE_CONTROL.md).

## EPS/Links and editable PowerPoint export — P08.1 (2026-10-09)

Adapted the following original MIT OR Apache-2.0 proposals locally, retaining
licences, copyright and Pagenor identifiers. Contributor metadata and owner
artwork remain unchanged. Source file identities match the pinned heads.

| PR | Source author | Inspected head | Adaptation |
|---|---|---|---|
| [#150](https://github.com/storytold/designcraft/pull/150) | @skylartaylor | `b050343a6ca790d7a3e0c7167827260c114affb9` | EPS source/previews, point sizes, graphic Relink filter and transactional replacement preparation. |
| [#170](https://github.com/storytold/designcraft/pull/170) | @PhilippKrueger | `a075efccd8ab1eccb9db12e48c9bbca2309b974d` | Editable PowerPoint backend, CLI route and export UI. |

Local additions retain format-version migration and document-scoped composition,
cap actual file reads and export structure/bytes, validate geometry before edits,
keep grouped threaded text editable, and check embedding eligibility against
OS/2 metadata. The options dialog preserves errors/cancellation and shows export
warnings. The source contributor-profile change is omitted. EPS uses previews
without interpreting PostScript. Native PowerPoint compatibility and cross-platform
GUI execution are separate milestone checks. [Behavior and limits](docs/pagenor/EXPORT_INTERCHANGE.md).

## INDD subset import — P08.2 (2026-10-09)

Adapted [#98](https://github.com/storytold/designcraft/pull/98), by @denizkoekden,
at inspected head `5593c8a61e8c9ec42b76177781635cb3ac0c8f69`. Source blob
identities match the pinned snapshot. Original MIT OR Apache-2.0 licences and
copyright remain applicable. Pagenor uses a standalone L0 reader and its existing
IDML importer; unrelated layer reclassification is omitted.

Open/Place/native/web filters preserve `.pgnr`, import target identities and
scoped document fonts. Subset warnings are visible, imported documents start
unsaved, and failed conversions do not add partial documents. Local additions
bound actual input/name reads, style inheritance, model traversal, text/attributes
and generated XML pieces. No Adobe application bundle or actual Adobe-produced
fixture is copied. Tests construct synthetic documents in Rust; real-file version
breadth/fidelity remains unverified. [Supported subset](docs/indd.md).

## AI assistant and review backend — P09.5 (2026-10-09)

Adapted closed [#54](https://github.com/storytold/designcraft/pull/54), by
@ShaoWenbinSaleh, at inspected head
`21681fb3379ca55ef1c7498ecd19a8f74b16178f`. Original MIT OR Apache-2.0 licences
and copyright remain applicable. Provider services are L2, persisted review data
L1 and the UI/worker bridge L6. Pagenor keeps its current format version 3,
document-scoped fonts, import targets, clipboard bridge, owner branding and
contributor metadata. Upstream packaging changes, live examples and claims are
omitted. Local additions bound persisted input, requests, models, workers and
reports; preserve transaction/history on errors; share rebased snapshots; and
retain provider model labels and plan-use disclosure. Actual provider and native
Windows/Linux credential execution remain unverified. [Scope](docs/ai-assistant.md).


## Authenticated desktop control adaptation — P00.5 (2026-10-09)

Adapted [upstream #151](https://github.com/storytold/designcraft/pull/151), by
@skylartaylor, at inspected head `c4ca2a9dbe8980614ae9d29bbc067fdf9f3f2491`,
based on `14e677b24216396e398823ff034d9882322362d1`.
The MIT OR Apache-2.0 licences and original copyright remain applicable.

The server/client source was compared with the pinned Git blobs before adaptation.
This ports token authentication, fallible listener/client threads, connection/request/
framing/reflection/timeout bounds, loopback client destinations and click-count limits.
Desktop, CLI/MCP and automation examples use Pagenor identifiers. Existing development
rules were retained; only their local control examples were updated for authentication.
Pagenor keeps automation off by default and requires `--control`; the documentation's
previous, unimplemented environment-port shortcut was removed. Document commands,
clipboard, recovery, lazy scoped fonts, branding and `.pgnr` behavior stay intact.

Local additions remove automatic request replay after a lost reply, validate reply
IDs/status, check ID exhaustion, bound serialization before allocation can grow without
limit, propagate socket-option failures and avoid exposing invalid token values in
errors. The UI queue holds at most 16 requests and rejects overflow before dispatch;
a reply timeout does not cancel an already queued command. Incoming tokens are bounded
before copying, CRLF requests work at the byte limit, and connection slots use a
minimum-Rust-compatible compare/exchange loop. Best-effort stderr writes avoid a
broken diagnostic pipe causing a panic; capability tokens are not passed to a logger.
Regression tests exercise actual loopback transport, auth-before-UI delivery, queue
pressure, lost replies/reconnection and pointer-event bounds. Validation and remaining
platform scope are recorded in [control hardening](docs/pagenor/CONTROL_HARDENING.md).


## Desktop logging adaptation — P00.6 (2026-10-09)

Adapted [upstream #122](https://github.com/storytold/designcraft/pull/122), by
@switch87, at verified head `36255f16979853d795370d1130be4c18affd4819`.
Its logging implementation originates in `4f2595500207de49382851b275da15e23f9aa99e`;
Git records Gert Pellin as the author of both commits. All three inspected source-file
blob identities match the pinned head. Original MIT OR Apache-2.0 licences and
copyright remain applicable; no contributor registry/profile changes are made.

The logger and selected desktop/README integration are adapted with Pagenor names
and existing settings paths. Timestamped records go to stderr and a current log,
with two previous launch logs. No new dependency is introduced. Pagenor keeps
control off by default, token announcements out of the file logger, its current
recovery/clipboard architecture, lazy document-scoped fonts, branding and `.pgnr`.
The upstream environment-port shortcut is omitted; invalid explicit control ports
are diagnosed without echoing supplied values. The existing engine panic hook is
retained, with the desktop logger installed before it.

Local additions cap individual record formatting and aggregate pending bytes,
include the cap notice within the file byte budget, stop buffering after file
failures, flush the sink and preserve UTF-8 boundaries. New Unix diagnostic files
are private; exclusive creation avoids truncating a file appearing after rotation.
Startup/open/final-recovery errors use the logger, while unavailable storage falls
back to safe stderr. Tests cover closed stderr, fresh-process rotation, the existing
panic hook and control-token exclusion, as well as deterministic file failures.
[Behavior and local evidence](docs/pagenor/DESKTOP_LOGGING.md) distinguish production
logger cross-compilation from deferred native platform execution.


## Batched composition lookup adaptation — P00.7 (2026-10-10)

Adapted the performance idea and three-file proposal in
[upstream #172](https://github.com/storytold/designcraft/pull/172), by
@ZuraGuerra (Git author Zura Guerra), at verified head
`85657c9b88a4ff520d41b8c0913678a72b34f8ed`. All three source blob identities
match GitHub's file metadata. On this date GitHub marks the request closed,
unmerged and “withdrawn”; its author says it was opened in error. Pagenor's local
adaptation is independently reviewed, not an upstream acceptance claim. Original
MIT OR Apache-2.0 licences and copyright remain applicable. Contributor registries
and profiles are unchanged.

The composition signature locates a story's frame IDs in one document walk.
Pagenor retains its anchored/embedded-frame fallback, font scope, allocation
retention and existing cache invalidation behavior. Local changes replace the
source's recursive helper with iterative pre-order traversal, shared with single
lookup, and accept a resident ID slice rather than an unrestricted iterator.
Missing/repeated IDs, parent spreads and first-match ordering are preserved.
There is no persistent document index or dependency addition. Regression tests,
local before/after measurements and platform limits are recorded in
[composition lookup](docs/pagenor/COMPOSITION_LOOKUP.md). This task stays local;
no remote publication or running macOS bundle replacement is performed.

## Reviewed upstream adaptations — P09.6 (2026-10-10)

Ten UI proposals were completed sequentially with focused interaction/command
tests. These are selected adaptations from the pinned MIT OR Apache-2.0 source
heads below, not merges of their stacked branches. Existing font scopes,
branding, native format, clipboard/recovery and Rust layers are retained.

| PR | Source author | Inspected head revision | Adaptation |
|---|---|---|---|
| [#196](https://github.com/storytold/designcraft/pull/196) | @vpkoskinen | `b6ca25cf660a80be6f5a1047f0aab794658c0334` | PDF options/settings; transactional cancel/error handling, document UID guard and desktop/browser delivery. |
| [#199](https://github.com/storytold/designcraft/pull/199) | @vpkoskinen | `aeaa9f981862ceb3bcabe057ed715b02fc162832` | Whole clickable chip/name rows shared by Properties, shared pickers and task bar. |
| [#221](https://github.com/storytold/designcraft/pull/221) | @kubuntugamer | `733c4ff52473196ac6b5e8fae5879149a65ca929` | Fill/stroke chip double-click opens the existing RGB picker; safe colour conversion and popup-anchor handling. |
| [#228](https://github.com/storytold/designcraft/pull/228) | @kubuntugamer | `97b6cd6aac85f97e006de74d53da648a70daaf2b` | Canvas contextual commands with text priority, enablement, shared clipboard bridge and command undo. |
| [#233](https://github.com/storytold/designcraft/pull/233) | @kubuntugamer | `4d3e584918eb637934034306b5ca345c37c03793` | Paragraph Rules using Pagenor camelCase Rule attributes, retaining inherited/edited values; scrollable rule controls. |
| [#226](https://github.com/storytold/designcraft/pull/226) | @kubuntugamer | `d8f5f1f31274fc271de7d99927cd6e3b12d2a371` | Stable zero-height hidden control panel, retaining the existing responsive two-row contents. |
| [#231](https://github.com/storytold/designcraft/pull/231) | @kubuntugamer | `e01810d731d65c189b09efc309522fc0f8254d2e` | Interactive for PDF opens the control bar and Buttons/Forms; custom workspace restoration retained. |
| [#217](https://github.com/storytold/designcraft/pull/217) | @kubuntugamer | `78be3ce189b52526f844b66957515899a99f2e25` | Platform-aware accelerator aliases, physical Mac Control matching, normalized conflict comparisons. |
| [#219](https://github.com/storytold/designcraft/pull/219) | @kubuntugamer | `c4990aa338e8fb0ec4a81a37703381dcfabcbdee` | Record physical Mac Control/portable primary combinations; ignore held repeats and consume recorder keys. |
| [#224](https://github.com/storytold/designcraft/pull/224) | @vbasky | `6aaa4f0b753aa008f3ca18e7910083f79a58a1ad` | Shared row-distribution command/menu/panel with composed-height checks, sparse one-pass height collection and undo. |

Every changed source-file Git blob matches the API snapshot for all ten heads.
Italian translation #218 is deferred by the owner. Previously handled stacked
clipboard, close-confirmation, style-deletion, toolbar and typed-dialog changes
are not overwritten. See [behavior and evidence](docs/pagenor/EDITOR_UI_UPDATES.md).


## Release-note audit — P00.8 (2026-10-10)

Audited all 34 changes in the [v0.4.0 release](https://github.com/storytold/designcraft/releases/tag/v0.4.0),
from v0.2.1 at published release head `14e677b24216396e398823ff034d9882322362d1`.
All 32 application/workflow PR merge revisions are ancestors of Pagenor's initial
source; current implementations were checked and retained. Distribution/docs
#132/#135 are selected Pagenor adaptations in P10.2. No upstream identifiers,
branding, contributor profiles or whole-branch merge were introduced.
[Individual evidence](docs/pagenor/UPSTREAM_040_AUDIT.md). Pagenor 0.3.0 is a local
application checkpoint, with independent versioning and unchanged format 3.


## Distribution and download documentation — P10.2 (2026-10-10)

Adapted the following MIT OR Apache-2.0 source snapshots locally. All ten
changed source-file blob identities (nine from #132, one from #135) match their
pinned API/Git records. Original copyright/licences remain applicable; generated
contributor registries and self-submitted profiles remain untouched.

| PR | Source author | Inspected head | Adaptation |
|---|---|---|---|
| [#132](https://github.com/storytold/designcraft/pull/132) | @echelon | `0f3f15df61547b0cae46941d954eedf84216bd1d` | FreeBSD tar packaging, x86_64/aarch64 Flatpak bundles, AppImage zsync and unsigned release-job separation. |
| [#135](https://github.com/storytold/designcraft/pull/135) | @echelon | `c4945ce8a1756f6caadf01f6459a3aefca073a4c` | Download/build-format documentation, Windows first, adapted to Pagenor’s own repository and actual local status. |

Pagenor keeps version **0.3.0** per the owner’s instruction, format 3, `.pgnr`,
Rust layers, scoped lazy fonts, clipboard/recovery, supplied artwork and owner
rights. Existing nfpm page-layout description/vendor are retained; the default
maintainer no longer refers to an upstream address. Notices and bundled-font
licences are retained across the adapted package formats. Flatpak allows X11
because Pagenor’s Linux host explicitly selects it; network remains opt-in.

Local additions validate and bound release-tar extraction and both ELF machine
headers, keep dry-run stubs out of release artifacts, remove stale zsync metadata,
route updates to the builder’s own repository, and bound FreeBSD VM resources to
match the existing private-runner budget. Source and bundle manifests agree.
Synthetic package checks and static workflow/script checks do not establish
native installation/execution. All 34 release-note items are accounted for in
[the audit](docs/pagenor/UPSTREAM_040_AUDIT.md); actual native package/signing
validation remains a milestone task. No GitHub push or publication is performed.


## Reviewed upstream adaptations — P09.7 (2026-10-10)

The new-PR pass adapts 23 selected proposals: eight editor UI changes followed by
six compatibility fixes and nine multipage/composition changes. Pagenor stays
0.3.0. The existing architecture, lazy scoped fonts, native extension, owner
branding, clipboard and recovery paths are retained. See the per-PR author/head
provenance, combined-branch adjustments and verification limits in
[editor and publishing updates](docs/pagenor/UPSTREAM_EDITOR_PUBLISHING.md).


## New Document dialog adaptation — P09.12 (2026-10-11)

Adapted open [upstream #325](https://github.com/storytold/designcraft/pull/325),
by @moppeeer, at pinned head `e78407027e08e1c80613e83533fbe916f508a42e`.
The seven reused Rust source-file Git blob identities match the fetched API
snapshot. Original MIT OR Apache-2.0 licences and copyright remain applicable.
Contributor registries/profiles are unchanged. This is a selected adaptation,
not a merge of the upstream branch.

New Document has Recent/Saved/Print/Web/Mobile tabs, proportioned preset cards,
page settings, linked margins/bleed/slug, and command-backed saved presets.
The latest upstream popup Enter/Escape correction is included. Pagenor retains
its Welcome recent-file thumbnails, Home tool visibility, native menus, PDF/PPTX
export and font-replacement dialog footers, save/recovery/input paths and separate
owner-supplied app/document icons. Italian translation changes are deferred per
the owner. There are no new dependencies, platform adapters, unsafe blocks,
format or version changes. Pagenor stays **0.3.0**, with local changes only.

Additional input bounds cover loaded preset counts/edges, 32-bit page-count casts,
finite geometry, preset-capacity failure atomicity and the untitled counter.
Failed page-size confirmation retains the typed settings for correction. The save
popup is explicitly layered above its modal parent after a native UI observation.
Implementation and local validation are recorded in
[New Document](docs/pagenor/NEW_DOCUMENT_PRESETS.md).

## Chapter numbering adaptation — P07.2 (2026-10-11)

Adapted open [upstream #341](https://github.com/storytold/designcraft/pull/341),
by @moppeeer, at pinned head `498e3d401c8214d3f617c59af0893a2eaf62dfd1`.
Six reused Rust source-file Git blob identities match the fetched API snapshot.
Original MIT OR Apache-2.0 licences and copyright remain applicable. This is a
selected port, not an upstream branch merge; contributor registries are unchanged.

Document chapter style/source, command, book chapter resolution, variable and
cross-reference labels, and section prefix/style validation are adapted. Pagenor
also retains partial section settings, supplies the missing three/four-digit
styles, checks loaded labels and numeric conversions, preflights book inputs,
protects dirty open documents, synchronizes successful open-document numbering
and reuses the safe metadata-preserving native writer. IDML chapter numbering is
not mapped. No dependencies, platform adapters, unsafe blocks, branding artwork
or application/native format versions change. Version remains **0.3.0**.
[Behavior and validation](docs/pagenor/CHAPTER_NUMBERING.md).

## Numbering window adaptation — P09.13 (2026-10-11)

Adapted open [upstream #342](https://github.com/storytold/designcraft/pull/342),
by @moppeeer, at `9c683f3ef864cb705db3ae92f480d10684acfce7`. Six reused Rust
source blobs were verified against the pinned API snapshot. The branch's #341
foundation was already adapted as P07.2; it is not reapplied. MIT OR Apache-2.0
licences/copyright remain applicable; contributor registries are unchanged.

The dedicated section/chapter window, menu routing, side actions and shared
translations are adapted. Italian is deferred. Pagenor supplies a command-backed
atomic combined apply operation and retains prior popup guards, New Document,
export/font dialog footers, IME, save and branding work. No dependencies, unsafe
blocks or version changes. Version stays **0.3.0**; work is local.
[Behavior and validation](docs/pagenor/NUMBERING_SECTION_OPTIONS.md).

## Tabs editing adaptation — P07.3 (2026-10-11)

Adapted open [upstream #343](https://github.com/storytold/designcraft/pull/343),
by @aiongg, at `989373114be324319ff57d14596df0649a996cd0`. Nineteen pinned
Rust source-file blobs were verified. Italian is deferred; the Ukrainian-file
update is skipped because that locale is not configured in this fork. MIT OR
Apache-2.0 licences/copyright and contributor registries remain unchanged.

Tabs commands, the floating panel and Paragraph Style Options ruler, shared
controls/icons, validation through paragraph/style writers and native/IDML load
repair are adapted. Pagenor additionally rejects explicit malformed target,
string and width fields. Existing font-size presets, section window, Welcome,
IME, safe saves and lazy font scopes are retained. No dependencies, unsafe blocks,
native format or application version changes. **0.3.0**, local only.
All seven gates pass: 1,524 tests, 3 ignored; 20 WASM packages. Panel/style renders
were inspected. [Behavior and limits](docs/pagenor/TAB_STOPS.md).

## Swatch and Glyphs panel adaptation — P09.14 (2026-10-11)

Adapted open [upstream #344](https://github.com/storytold/designcraft/pull/344),
by @moppeeer, at `d361d84e3812922bb9651a9f51a2110efa97fa80`. Four reused Rust
source-file blobs were verified against the pinned API snapshot. MIT OR Apache-2.0
licences/copyright remain applicable; contributor registries are unchanged.

Four swatch-list copies now share the same width-aware renderer: hovered rows
use white-on-blue, selection shading remains, and names/grid columns avoid floating
scroll bars. Existing font-size preset and document font-scope controls are retained.
Two rendered-geometry regressions and all seven gates pass: 1,526 tests, 3 ignored;
20 WASM packages. Actual egui swatch-hover/Glyphs renders were inspected. No new
dependencies, assets, unsafe blocks or version changes. **0.3.0**, local only.
## Ruler zero point adaptation — P09.15 (2026-10-11)

Adapted open [upstream #345](https://github.com/storytold/designcraft/pull/345),
by @zydvc26s67-web, at `2cba55f623f28ffd53e553fd546d74f6553ace62`. Two pinned
Rust source blobs verified. MIT OR Apache-2.0 licences/copyright remain applicable;
contributor registries unchanged. Drag the ruler corner to set a view-local origin;
double-click resets it. Pagenor scopes drag state per pane and preserves the origin
on a plain click. Document contents/history are unchanged. Two UI regressions,
inspected actual egui renders and all seven gates pass: 1,528 tests, 3 ignored;
20 WASM packages. No dependencies, unsafe blocks or version changes. **0.3.0**, local only.

## Page-number bounds adaptation — P07.4 (2026-10-11)

Adapted open [upstream #346](https://github.com/storytold/designcraft/pull/346),
by @aiongg, at `1ec2136c841ebb0ba2bc2f52481ae9aee7e894cf`. Five pinned Rust
source blobs verified. MIT OR Apache-2.0 licences/copyright remain applicable;
contributor registries unchanged. Document Setup and section start numbers share
whole-number bounds of 1–99,999. Continuing sections use iterative, saturating
page numbering. Book list accumulation saturates; Pagenor retains its existing
checked, preflighted book pagination. Rejected setup requests leave document and
history unchanged. Loaded extreme page numbers and wide host indices are covered;
page labels additionally avoid footnote-only Symbols allocations. Initial all-seven
gate run: 1,530 tests, 3 ignored, 20 WASM packages; final integration checks cover
the additional label safeguard. Version remains **0.3.0**, local only.

## Multiple-click selection adaptation — P07.5 (2026-10-11)

Adapted open [upstream #347](https://github.com/storytold/designcraft/pull/347),
by @zydvc26s67-web, at `67f33cf279381d41f4c09099215d8ecff0ef6442`. Ten pinned
source blobs verified. MIT OR Apache-2.0 licences/copyright remain applicable;
contributor registries unchanged. Explicit TripleClick tool/control/MCP events
and text.selectLine use Pagenor's existing table/footnote-aware selection command.
The branch's older canvas rewrite is superseded by the already-working press
counter, preserving four/five clicks and quick native drawing/threading input.
Batched ruler-corner input receives the same press-origin recovery. Release-time
click upgrades recover complete double/triple clicks within one paint, without
repeating port unthread or downgrading four/five-click selection. Six added
regressions cover engine/control/MCP, rapid native-shaped egui input and ruler reset;
tool tests extend explicit double/triple coverage. Actual word/line renders inspected.

All seven final gates pass: **1,536 tests, 3 ignored; 20 WASM packages**, including
P07.4's additional hostile-file page-label safeguards. No dependencies, assets,
unsafe code, native-format or application-version changes. **0.3.0**, local only.
[Behavior and adaptation](docs/pagenor/MULTICLICK_SELECTION.md).

Native follow-up: the final 0.3.0 `uncommitted-P07.5` bundle is installed locally in
Applications; signature and independent app/document icon hashes pass. Temporary-file
reopen, numbering/chapter restore, Tabs, ruler drag/reset, word selection and the
corrected triple-click request pass. App is left on Welcome with no open documents.
A separate native test under Documents/Codex stalled at the existing synchronous
sibling document-font-folder probe (`read_dir`/kernel open). The OS/filesystem cause
is unresolved; it is retained as a validation limitation, with the sample in the
local #341–347 evidence report. Native Windows/Linux execution remains pending.

## macOS installer adaptation and version checkpoint — P10.3 (2026-10-11)

Reviewed the complete v0.4.0 → v0.5.0 comparison at release revision
`4e0e3949f1eb91c874f2a47d58c2187031f4c97e`. Its only feature contribution is
[#131](https://github.com/storytold/designcraft/pull/131), by @XusBadia, inspected
at head `ec42e542e909ba0ff75a4efb9a95012be11124ee` (merged as
`66c7ce7e96ec286a091878cc2541ee8369aa79d6`). The other release commit bumps the
upstream version and refreshes generated contributor statistics; those are not
ported. This release does not incorporate the separately adapted open #341–347.

The generator’s Git blob `6bad87a0c5485ce34416311151d2d08c026a550a` was verified
against the pinned source. Its MIT OR Apache-2.0 generator/layout approach and
package staging are adapted. Pagenor uses its existing workspace resvg renderer,
256-colour HiDPI TIFF and Pagenor volume/application names. Installer artwork is
newly composed from Appaji Ambarisha’s supplied logo/icon with bundled Inter
caption outlines; upstream artwork and trademarks are not imported. All terms
are recorded in ASSETS.md and packaging/macos/dmg/LICENSE.txt and staged beside
the background. App/document icons and signing/notarization flow are preserved.

On 2026-10-11 the owner authorised relevant version increases, including minor
updates. The local workspace advances 0.3.0 → **0.4.0** for accumulated editor,
Welcome, keyboard, chapter/section/tab-stop and distribution changes. The Cargo
version helper updates only workspace lock entries, with no external dependency
upgrade. This is not a new GitHub publication and does not replace v0.3.0 assets.
See CHANGELOG.md and docs/pagenor/VERSIONING.md.

P10.3 validation: all seven gates pass (1,536 tests, 3 ignored; 20 WASM packages),
nine packaging contracts and ShellCheck pass. Generated TIFF/layout reproduce
byte for byte; alias/HiDPI/outlined vectors/icon clearance pass. The Apple-silicon
0.4.0 development app is installed locally; native Welcome and About version,
actual DMG Finder artwork/layout, separate icon metadata and strict ad-hoc
signature checks pass. Finder’s persistent tab/path/status bars required a 100 pt
window allowance to keep the captions visible, without preference changes.
Native Windows/Linux/Intel execution and signed/notarized releases remain separate.
The existing document-font folder scan stall is not addressed by this update.


## P00.9 — upstream v0.6.0 audit and local adaptations (2026-10-11)

Inspected all 71 distinct release changes at upstream release head
`a9124a0517594ce9872f73637e7b841a4967bcf9`, comparing each individual merge patch
against this fork. [Per-item decisions](docs/pagenor/UPSTREAM_060_AUDIT.md): 45
already covered, 7 adapted here, 16 pending, 3 deferred/outside scope. Adoption is
not complete; Italian remains owner-deferred, upstream sibling branding and
readiness claims do not become Pagenor product content.

- #116, @IHAGI-c, inspected head `ac4c7e6b8ebd08b948c3c468cf8dcfe46a38bdba`:
  shared SVG font bytes/resolvers, adapted to document scopes, canonical aliases,
  selected collection faces and scope/generation-aware screen caches. Synthetic
  fonts now include PostScript names. SVG CSS variation settings remain usvg's;
  existing Pagenor text-frame named instances remain unchanged.
- #229, @skylartaylor, merge `cfac46a8d9971b28d00faa3e48755b3581b0f8c0`: mixed gradient output colour spaces and
  PDF/archival/booklet regression matrix.
- #200, @vbasky, merge `3de58750cd0aa0d766470a4b6c46d4afa15b4a1e`: CLI access to existing PDF/X checks, with
  strict argument count, bounded reads and fork-specific help/tests.
- #299, @aiongg, merge `0b69d6856ca0bca10cfe712d9f8ef724af74c8f4`: distinguish missing linked images from decode
  errors; preserve decode behavior for corrupted nonempty data.
- #258, @kubuntugamer, merge `a50d279cba5647e0e5db1f58539c97f2d8a5c618`: facing-page merge regressions adapted to
  Pagenor's explicit per-page defaults and unsupported tiling refusal.
- #269, @sawizzle, merge `a42fa206774a704cd9420aaa6218fcd21c66f3fc`: installed Pagenor CLI MCP guidance, with
  existing opt-in authenticated control.
- #398, @echelon, head `bb19cccd7e48326ae94e2004621144abd6afffd1`: tar-only Linux
  riscv64 cross distribution. Explicit C/ring/pkg-config routing, validated
  target/architecture and QEMU smoke hook; no upstream version/branding copied.
- #349, @ironinthesoul, head `f05e8f5d9876bb85dfbfe5c084331beefae75707`: behavior
  already covered by the existing host Unicode/object clipboard bridge. Native
  Copy/Cut joins the existing once-only/reversible regression; no second bridge.

Source blobs verified against Git's blob hash:
- #116 `crates/images/src/lib.rs`: `1e81115c4c33b48d3a9ec11b6e159e30e8212ebd`.
- #116 `crates/fonts/src/testing.rs`: `830f3d3584dbe9fb589a093d16b5aac394cd2c6a`.
- #398 `.github/workflows/release.yml`: `856bdbe6ec03f2724f2437629430e8f56668bb6a`.
- #398 `packaging/linux/package.sh`: `fd7fc4ef89c38ae699cf09bd7dbefebed0d8645c`.
- #229 `crates/pdf/src/export.rs`: `72b378a97e841a8269b5ceb7ab8272d9b44500d4`.
- #229 `crates/pdf/tests/gradient_color_spaces.rs`: `c523a418792cac09e598d2ae1e8e6777382a5be0`.
- #200 `apps/designcraft-cli/src/main.rs`: `b1819c3573304db1e4fc1fee3ef3ed421507cd23`.
- #299 `crates/pdf/src/export.rs`: `d92fc242bd011a565c8f1e53631aa22b866abdce`.
- #299 `crates/pdf/src/tests.rs`: `9ac53d057a783d42d9d98dff9e8c535e4b8ab626`.
- #258 `crates/engine/src/cmd/datamerge/tests.rs`: `e4636d109911081a623f0a85aed1407cc65f1d32`.
- #269 `docs/mcp.md`: `772f8a91f7a6ae09bfcf454def24fd97edec22d8`.

Original MIT/Apache notices remain. No contributor registry or owner artwork
changes. All seven local gates pass: 1,548 tests, 3 ignored, all 20 WASM packages.
Twelve packaging contracts, ShellCheck and actionlint pass; inspected canvas scope proofs and PDF extraction/raster regressions and local macOS 0.4.0 bundle verification are separate evidence.
Windows/Linux/RISC-V native execution remains pending at a release milestone.
No GitHub publication or push. Current local build: `uncommitted-P00.9`.

## P11 — editor and publishing adoption (2026-10-11, local checkpoint complete)

Owner authorized the non-CJK queue in docs/pagenor/UPSTREAM_EDITOR_ADOPTION.md.
CJK/Italian and unsafe-policy exceptions are excluded. The complete selected group is verified as local **0.5.0**. No publication.

| PR | Contributor | Inspected head | Local adaptation |
|---|---|---|---|
| [#447](https://github.com/storytold/designcraft/pull/447) | @aiongg | `c537f1ae46761f57db132f75d825a66edafd19b5` | Consume font-menu keys; accessible family fields; Pagenor fallback fixture retained. |
| [#399](https://github.com/storytold/designcraft/pull/399) | @kubuntugamer | `bfafcd5a580cb68200a5e904ff1192c7497c6de8` | Buffered hex input commits/reverts on focus loss or Return, addressing upstream review; RGB rounding. |
| [#388](https://github.com/storytold/designcraft/pull/388) | @moppeeer | `f6e96d21063bb105a17a60e97ac213b093b2b530` | Live Pencil path in layer colour and tool/UI regressions. |
| [#408](https://github.com/storytold/designcraft/pull/408) | @vxkj1211 | `b834b92c0245bc50ec656a603c6f00471a58f0a6` | Vector-drawn align/distribute icons using the existing icon system. |

First four changes: 256 UI tests pass, 1 existing ignore, and focused Pencil
tool regression passes. Upstream MIT/Apache notices and owner asset rights remain.

P11 editor additions verified:

| PR | Pinned upstream head | Contributor |
|---|---|---|
| [#446](https://github.com/storytold/designcraft/pull/446) | `eab0ac7f4bb1221db4d1099585e645d21e30f1d6` | @aiongg |
| [#373](https://github.com/storytold/designcraft/pull/373) | `1fe1ec9cc6fbc04d80532583c5c014ddb8fae2cd` | @aiongg |
| [#416](https://github.com/storytold/designcraft/pull/416) | `bf8c3486f1680e307e3f4baff478f2be9e7daa5a` | @aiongg |
| [#378](https://github.com/storytold/designcraft/pull/378) | `b84a58391fb4cf4fafab563872b2bc58527ce998` | @aiongg |
| [#415](https://github.com/storytold/designcraft/pull/415) | `971cad3c0f0332bf6ca936a81ba189493461b1e5` | @aiongg |
| [#444](https://github.com/storytold/designcraft/pull/444) | `3bda745a520e6e2ed99da8e8675f9272a8d2c30d` | @aiongg |
| [#409](https://github.com/storytold/designcraft/pull/409) | `2bdb0ea01d86cacfa3875968a36c2d4357727305` | @skylartaylor |
| [#445](https://github.com/storytold/designcraft/pull/445) | `60f2c597a573afd169f2602f868da83d1b6a869c` | @aiongg |
| [#442](https://github.com/storytold/designcraft/pull/442) | `94fa7dc2ada7344270139ed8df14c90ebafc902e` | @aiongg |
| [#448](https://github.com/storytold/designcraft/pull/448) | `f5867f26220587422c8ab993fc3360bfbe170428` | @coygeek |

P11 remaining pinned adaptations (all applied; local gates pass):

| PR | Contributor | Inspected head |
|---|---|---|
| [#393](https://github.com/storytold/designcraft/pull/393) | @con3code | `cc836fe4dffc427b7377d00c4212c94904f75e1c` |
| [#434](https://github.com/storytold/designcraft/pull/434) | @aiongg | `c0148325726139627aaff23a168dc6862ce24101` |
| [#411](https://github.com/storytold/designcraft/pull/411) | @noah1400 | `ad173006f7c0b444a19ab1e38414d0dbb6cc963d` |
| [#420](https://github.com/storytold/designcraft/pull/420) | @aiongg | `daf1a0ccb3875a6435a1d56cd4cc18c199847738` |
| [#458](https://github.com/storytold/designcraft/pull/458) | @sanyamsingh001 | `eae9ca097ca4462a80b03cfc3f6557635df4fa10` |
| [#305](https://github.com/storytold/designcraft/pull/305) | @sanyamsingh001 | `21fa11780cbea6801f3e6ad1e0a0f158595119bd` |
| [#456](https://github.com/storytold/designcraft/pull/456) | @sanyamsingh001 | `f58c21228921632d0d106c297a47dab2209ea839` |
| [#457](https://github.com/storytold/designcraft/pull/457) | @sanyamsingh001 | `9cb8d05a3843fc1075f36529e49619ade5df740d` |
| [#402](https://github.com/storytold/designcraft/pull/402) | @KnottyDyes | `00569f42cc696ca2c5878fd625d33f79998b0a74` |
| [#403](https://github.com/storytold/designcraft/pull/403) | @KnottyDyes | `58e57bfba2b70761844e560890d10f74af5b5a8e` |
| [#369](https://github.com/storytold/designcraft/pull/369) | @noah1400 | `dfdc650318fbd48d6ce75b4e4ec3f1b233ac0f51` |
| [#379](https://github.com/storytold/designcraft/pull/379) | @aiongg | `ac4b83fa3930f56c49fa6f817700b8cf692377f3` |
| [#359](https://github.com/storytold/designcraft/pull/359) | @aiongg | `7ed9f37d4383f5f180033d79a2d28f6744a685f0` |
| [#360](https://github.com/storytold/designcraft/pull/360) | @aiongg | `91a01a6774e560511f289628ef45abfc7b1dd06f` |
| [#381](https://github.com/storytold/designcraft/pull/381) | @aiongg | `70a04944a5c42d62efc403a083728618b694a6de` |
| [#384](https://github.com/storytold/designcraft/pull/384) | @aiongg | `fbda023d7f46fdf3bb576af53f222815edf75cd0` |
| [#433](https://github.com/storytold/designcraft/pull/433) | @aiongg | `814102ed351c3d10ac503e79d5c5726a26ebde39` |
| [#422](https://github.com/storytold/designcraft/pull/422) | @aiongg | `279309ad120352c48ca56ae01787e991da18378b` |
| [#421](https://github.com/storytold/designcraft/pull/421) | @aiongg | `2b96dffda145f8ae46c64ac6784632782be6624d` |
| [#427](https://github.com/storytold/designcraft/pull/427) | @MaridWSH | `62faa8eee2d7c8bbee7bff00c4e10da752b71a41` |
| [#372](https://github.com/storytold/designcraft/pull/372) | @aiongg | `0474a8b5eca1634ea73b31a95465cf1ce57eb260` |
| [#357](https://github.com/storytold/designcraft/pull/357) | @aiongg | `d03dd226c5bb2f2ff862dc6bede50310d008df98` |
| [#418](https://github.com/storytold/designcraft/pull/418) | @aiongg | `5d872fbf1eadd18bc1312b2354c74712a49b446e` |
| [#419](https://github.com/storytold/designcraft/pull/419) | @aiongg | `505e748aacddfdbeeb83f857334535bf88670cf5` |
| [#356](https://github.com/storytold/designcraft/pull/356) | @aiongg | `a80d21161f5002f4760b26125548d81f5a6e4835` |
| [#362](https://github.com/storytold/designcraft/pull/362) | @aiongg | `0f3f4e0634b4254ae8294aae1e9c527a735c1da7` |
| [#382](https://github.com/storytold/designcraft/pull/382) | @aiongg | `cdd32a5066a82156a248ee4712b0e7059fce9b1e` |
| [#397](https://github.com/storytold/designcraft/pull/397) | @con3code | `cc12ea576ff80d59c3bbfb4355be3fb66f4179fe` |
| [#297](https://github.com/storytold/designcraft/pull/297) | @JeremyYao | `b5bca7ef64fc3844b06281bbcda09c396b582a9b` |
| [#380](https://github.com/storytold/designcraft/pull/380) | @aiongg | `158f366af8df9c42d94295cf180787b37bacec55` |
| [#452](https://github.com/storytold/designcraft/pull/452) | @skylartaylor | `9f7da3f6abcce29186af9a8660158e60419d069d` |
| [#455](https://github.com/storytold/designcraft/pull/455) | @echelon | `04600f4cbd2ae1f6cb681196235cee4e9261b057` |
| [#374](https://github.com/storytold/designcraft/pull/374) | @wondarkus | `f5da9d2b58bf171c84023b79d9ee4825b3824536` |
| [#295](https://github.com/storytold/designcraft/pull/295) | @aiongg | `8b81f31b770df570bc04a7a84459dd762e3e3a9b` |
| [#298](https://github.com/storytold/designcraft/pull/298) | @aiongg | `6a674b951b7231c0475bdb5ad57e42b7ddf6994b` |

Shared dependencies remain pinned: craft-ui `5c874d51eeb3de70e735c8ce87a59a7dedd47675`, craft-print `c22bb1b0ce6316f84973cb69675fe7331c5886cc`, safe native wrappers retain the workspace unsafe-code prohibition. Full local adaptation and validation notes: docs/pagenor/UPSTREAM_EDITOR_ADOPTION.md.

Linux System appearance portal prerequisite: [#271](https://github.com/storytold/designcraft/pull/271), @luniiya, pinned head `4620d2747edb7d87b5427b945d845489fce5393b`. Numeric/nested variant decoding and subscribe-before-Read ordering are adapted in `apps/pagenor/src/appearance.rs`; watcher cancellation and Pagenor preferences remain local adaptations.

P11 final local evidence: All seven `cargo xtask ci` gates pass: 1,856 tests passed, 6 ignored, no failures; 20 WASM packages. The 12 packaging contracts, ShellCheck and actionlint pass. The local macOS app is built and inspected separately. Windows/Linux native GUI and printing execution, signed/notarized distribution and GitHub publication remain milestone work.

The final local 0.5.0 app is installed at `/Applications/Pagenor.app`; its executable matches the packaged build byte-for-byte. Strict/deep signature checks, 0.5.0 executable/plist versions and separate supplied app/document icon resources pass. The previous 0.4.0 app is retained in a recoverable local backup. No GitHub publication.

## P11.1 — local editor usability, 2026-10-11

Pagenor 0.5.1 improves the imported canvas/docking/table foundations in response
to owner feedback: visible scrollbar tracks, Arrow cell selection, the Table
panel border/fill controls, functional groups, close/toggle actions and vertical
stacking with scrolling, plus a taller bottom bar. These are Pagenor adaptations
using the same pinned safe craft-ui implementation and existing table formatting
engine, not additional upstream PR adoption. See docs/pagenor/EDITOR_USABILITY.md.
No publication or native Windows/Linux runtime validation is implied.

P11.2 (2026-10-11) is a local 0.5.1 maintenance correction: document-window Close
keeps the workspace alive and returns to Welcome; explicit Quit retains save-all
and native input teardown. The right icon bar is widened to 44 pixels. No new
upstream adoption or dependency change. See docs/pagenor/WELCOME_RECENT_DOCUMENTS.md.

## Local master-page workflow — P11.3 (2026-10-11)

Owner-requested Pagenor work reuses the existing parent spread model, composition
and override architecture. Adds left/right editing, master settings, repeating
header/footer/number frames, inherited guides and default-master persistence. No
new upstream patch is imported in this pass. Shared Rust and the unsafe-code ban
remain. See [Master-page notes](docs/pagenor/MASTER_PAGES.md).


## Right toolbar columns — P11.4 (2026-10-11)

Owner-requested local 0.5.2 work extends Pagenor's grouped panel rail with an
independent one/two-column toggle and saved workspace preference. Reuses the
existing icons and shared docking implementation; no new upstream patch or
dependency. See [Right toolbar notes](docs/pagenor/RIGHT_TOOLBAR_COLUMNS.md).

## Selected October recheck adaptations — P11.11 (2026-10-12, local 0.6.0)

The following MIT OR Apache-2.0 source changes have been adapted locally, retaining applicable
original licences and copyright. These are inspected source pins, not whole-branch merges.
Validation and outstanding work are tracked in
[the implementation notes](docs/pagenor/UPSTREAM_ADOPTION_20261012.md).

| PR | Author | Pinned source head | Local adaptation |
|---|---|---|---|
| [#504](https://github.com/storytold/designcraft/pull/504) | @aiongg | `2077d9f088dc41bde6b201b098a596ded61fafd7` | Installed interface fallbacks for Indic and other scripts, with marks checked, shared face data, background work and refresh generations; retain existing CJK policy. |
| [#498](https://github.com/storytold/designcraft/pull/498) | @aiongg | `fe8a724da44ecd9cc997118340c55390f2e20825` | Malformed note/marker payload extraction repair, checked footnote/endnote/editorial insertion, wholesale text replacement cleanup; retain rich clipboard IDs. |
| [#506](https://github.com/storytold/designcraft/pull/506) | @noah1400 | `257fb08dbdc1140223e8db1f897845bff6913d9c` | Locked per-process recovery namespaces and coordinated takeover of released runs. Preserve legacy flat entries for explicit recovery; refuse shared-folder fallback and removed/replaced locks. |
| [#485](https://github.com/storytold/designcraft/pull/485) | @aiongg | `1a8df25fddb6e555bbbc11578a262585fcc0f1b2` | Use the stronger existing native writer for export, snippet, book, library, ASE, XML, MCP and CLI output. Preserve file metadata; add directory-flush diagnostics and bounded Windows replacement retries. |
| [#509](https://github.com/storytold/designcraft/pull/509) | @mjohnson7cd | `a02fc011bf5af1300a7e59ecb146487138fe38d1` | Paragraph panel optical margin alignment checkbox through type.para with undo. |
| [#510](https://github.com/storytold/designcraft/pull/510) | @japbcoelho | `5a1e9de92ed789abc176ab668fee68378a224ee3` | One RGB/CMYK component scale per colour, preserving Gray and finite/range checks. |
| [#511](https://github.com/storytold/designcraft/pull/511) | @japbcoelho | `72bdc0a9197985b000105b6291daf89d6cc9fd06` | Fractional PDF/SVG natural dimensions through direct Place, place cursor, fitting, tables and data merge; retain EPS validation. |
| [#473](https://github.com/storytold/designcraft/pull/473) | @MarekSpodBiedry | `30cbd67e3de32bf5d810c6569880b54a2bfffdc1` | SameParaStyleSpacing import, composition, IDML export and explicit-null inheritance/serialization. |
| [#474](https://github.com/storytold/designcraft/pull/474) | @MarekSpodBiedry | `5ab84500fed60deb67a3ef451d20fb765ad86a18` | Single-sided pages use the first listed master; preserve geometric left/right selection for RTL binding and shared snapping. |
| [#497](https://github.com/storytold/designcraft/pull/497) | @aiongg | `cde814c0dff8aaa5321b5fe4cccf9fd9f53d9d3d` | Alt shrinking/flipping, centered Alt+Shift aspect resize and Shift side-handle centering. |
| [#488](https://github.com/storytold/designcraft/pull/488) | @MarekSpodBiedry | `233aef8986b1113b16a50841a5dc7b519489e94b` | Fractional-scale app-bar width slack, retaining genuine narrow-window overflow. |
| [#486](https://github.com/storytold/designcraft/pull/486) | @aiongg | `63375bc84d3ce976ca7b1321f09ab142f85d25f0` | Filled Selection and hollow Direct Selection vector arrow icons. |
| [#503](https://github.com/storytold/designcraft/pull/503) | @aiongg | `7b903cf6b16b5baf5e43699b61875961900dcc11` | Shift+W Presentation toggle, remembered return mode and consumed Escape preserving tool/selection. |
| [#491](https://github.com/storytold/designcraft/pull/491) | @noah1400 | `0a9bfa57352e4872562487415ab1e3e4bf688f50` | Alt/Option wheel pointer zoom with modifier latching through smoothed/trackpad motion; retain existing command/pinch and plain wheel routes. |
| [#499](https://github.com/storytold/designcraft/pull/499) | @aiongg | `a73d106ab45158386ea2bb493412ec04b80de9a2` | Interactive command search with localized/English/id matches, keyboard navigation, enabled states, shortcut hints and restored focus; cap query and result count. |

### Additional P11.11 adaptations

| PR | Author | Pinned source head | Local adaptation |
|---|---|---|---|
| [#472](https://github.com/storytold/designcraft/pull/472) | @moppeeer | `3943e4c8b74956f659fab5c0731a209033752fd9` | Transformed text and group bounds for incremental repaint; depth cap falls back to full repaint. |
| [#505](https://github.com/storytold/designcraft/pull/505) | @KnottyDyes | `9b7b5d0e9d976559007441bb1ee8ea61e06c1f95` | Bounded raw ICC profile registration, profile IDs/classes/channel counts; profile-picker and PDF/X integration remain outside these foundations. |
| [#475](https://github.com/storytold/designcraft/pull/475) | @noah1400 | `40e8dbc6fb358a12cdd5099acef10d8552107d87` | Case-insensitive CLI export dispatch with native save-a-copy, PDF options validation and early unsupported-format errors. |
| [#500](https://github.com/storytold/designcraft/pull/500) | @aiongg | `47d17ca6f50883eaf1861a10fabb78b0816d9a12` | Custom text-wrap paths in composition, resize/bake and IDML with bounded finite geometry; retain existing contour modes. |
| [#487](https://github.com/storytold/designcraft/pull/487) | @aiongg | `56c8ed96673efaf553d44ac0f9f0ad6cc72182c3` | Code-drawn tool cursors, including pen add/delete/close/convert; preserve linking cursors and WASM/system fallbacks. |
| [#463](https://github.com/storytold/designcraft/pull/463) | @KnottyDyes | `97b753619a51f31e5786434bed5d6499e055a62b` | Document paragraph hyphenation defaults and object fitting parameters. |
| [#469](https://github.com/storytold/designcraft/pull/469) | @moppeeer | `90407a892aa6feb2223a3c4968f400b4daa63c6e` | Page presets, orientation, linked margins, bleed/slug and Adjust Layout in Document Setup; preserve untouched margins. |
| [#467](https://github.com/storytold/designcraft/pull/467) | @sanyamsingh001 | `004668eba87ab0686e95dde93e1147e52995deb9` | Explicitly edited dialog fields, captured text-frame targets, mixed-selection preservation, partial inset updates, table style creation and Shift+Tab. |
| [#468](https://github.com/storytold/designcraft/pull/468) | @soro-82 | `2b55f2ed0d9e76c8f0327194e850fbc21fe10a29` | Remaining Create Guides and Margins/Columns UI with document ruler units and page/master-side targeting; retain stronger existing explicit master inheritance. |
| [#477](https://github.com/storytold/designcraft/pull/477) | @luniiya | `fe21f3c16cd38b67f84f34b30ac9a85b529e353e` | Favorite theme stars and header appearance shortcuts, adapted to Pagenor’s existing System/light/dark brightness settings. |
| [#466](https://github.com/storytold/designcraft/pull/466) | @moppeeer | `26e75c8608f3fe6fa33c6ae2c655edc747af9cb9` | Font picker filters, sample sizes, recent fonts and expandable styles. Use small table metadata for filters; visible rows alone load preview faces. Interaction verification still in progress. |
| [#462](https://github.com/storytold/designcraft/pull/462) | @sanyamsingh001 | `1e526c66bdaf54b2e45ce64a69326e81c026dad4` | Bounded styled Markdown import and style mapping, retaining unsupported constructs as literal text with warnings. |
| [#459](https://github.com/storytold/designcraft/pull/459) | @sanyamsingh001 | `caa6421a3e9079a557daf4816cdc15e50b4d3932` | Respect primary-chain capacity removal during cleanup and avoid deleting groups containing empty tail frames; retain existing reflow decisions. |
| [#482](https://github.com/storytold/designcraft/pull/482) | @xristosmoug | `af9bddf34d0c2d978524c5b759e530e2c230c2ef` | Strip build-time extended attributes before signing, preserve signed DMG contents and verify the finished image plus installed copy. Local package-helper verification passes. |
| [#478](https://github.com/storytold/designcraft/pull/478) | @aiongg | `3d7638295a04ffda7c23bff445111b578df78a02` | Dispatch canvas pointer, key, clipboard and IME input in arrival order, including click/drag promotion and final release coordinates. Retain Pagenor multi-click selection, inline dead-key input, Alt-hand gestures, selected-text Alt-copy and platform shortcut distinctions. |
| [#460](https://github.com/storytold/designcraft/pull/460) | @sanyamsingh001 | `ab80e7f5b03b26787338727e6703ef362c8a2caf` | Resize/snapping preview agreement for fixed/scaled strokes, reflections and reachable dimension matches; retain centered resize and baseline/guide competition. Resize regression suites pass. |
| [#362](https://github.com/storytold/designcraft/pull/362) update | @aiongg | `0437ab27f5d3d6eb443d671d8194f93b7a741b9c` | Interpret PDF content bounds without rasterizing a 2048px preview; arbitrary clip/mask bounds remain conservative. |
| [#421](https://github.com/storytold/designcraft/pull/421) update | @aiongg | `8f30ba1c55342caff2bdd4af76c23b3614eeb602` | Avoid nested whole-line RTL ActualText; keep per-run cluster text and decoration. PDF regressions and Poppler codepoint-retention checks pass; exact viewer spacing/mark order remains a follow-up. |
| [#427](https://github.com/storytold/designcraft/pull/427) update | @MaridWSH (earlier @skylartaylor) | `d0120bf6a642beb92e31e026fc8a9bc64778bc13` | Preserve dash spacing in optical kerning, retaining ordinary Latin kerning. |

## Next adaptation batch — P11.13 (2026-10-12)

The owner authorised the selected queue through upstream #565. Per-change authors,
pinned heads, scope and validation are in [the adaptation log](docs/pagenor/UPSTREAM_ADOPTION_070.md).
P11.13.1 adapts the final nested-panic undo correction from #411 by @noah1400 at
`60f463f03cbe35396e49f3601c42fdf46d662fc2`, preserving Pagenor's earlier restore
before recovery and its command/document safeguards. Original MIT OR Apache-2.0
licences and copyright notices remain applicable. Pending rows are provenance
for reviewed candidates, not claims of completed implementation.

P11.13 adapts the related UI group #525, #527, #532, #533, #541 and #552:
tint/Frame/Text controls, View grouping and rendered document grid, selected-paragraph
Justification, Properties transforms, and the Tools Color Picker. Pinned heads/authors,
Pagenor-specific adaptations and validation are in the same 0.7.0 adaptation log.

P11.13 additionally adapts #519, #528, #550 and #561 (curved corners, one-time
image fitting, content transforms and saved clipping paths). Pagenor retains its
placement validation/PDF crop/wrap metadata, adds finite-transform guards, corrects
the native content outline found during QA, and records clipping capability as native
format 4. Pinned authors/heads and measured checks are in UPSTREAM_ADOPTION_070.md.

P11.13 adapts #551, #520, #521, #548 and #560 for table text colors, validated
swatches, command result IDs, scoped/lazy SVG PostScript lookup and IDML family/style
preservation. Pagenor's caret formatting, font activation epochs, document precedence,
variable SVG CSS and small-table catalogue scan remain. Sources and checks are in
the pinned 0.7.0 adaptation log; no private font or new font asset is committed.
