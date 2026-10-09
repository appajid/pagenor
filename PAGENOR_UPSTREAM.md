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
