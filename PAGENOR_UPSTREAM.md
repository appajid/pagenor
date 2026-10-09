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
