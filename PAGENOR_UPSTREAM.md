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
