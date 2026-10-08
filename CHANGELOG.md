# Changelog

## Unreleased

### Font discovery

- macOS system discovery now includes file-backed fonts registered with Core Text,
  alongside system, shared and user font directories.
- Catalogue TTF, OTF, TTC and OTC files and variable named instances lazily;
  deduplicate overlapping paths and family/style entries.

### Branding

- Use Appaji Ambarisha’s supplied Pagenor artwork for app icons, start screen,
  splash and About views. Preserve original editable SVGs and reserve branding rights.
- Generate portable outlined SVG, PNG, ICO and ICNS artwork with Rust tooling.
- Align Linux desktop/icon identifiers and Windows/macOS package metadata with Pagenor.
- Give the independent Windows installer its own stable upgrade/component identities;
  installations made with the inherited installer identities need separate migration review.

### Development baseline

- Connect Pagenor to `appajid/pagenor`; use its source and issue links in Help, CLI and MCP.
- Make project Help available before opening a document and disable unconfigured community links.

- Add macOS, Windows and Linux quality CI and a Rust 1.95 minimum-version check.
- Inherit Rust 1.95 across workspace packages, matching the locked UI dependencies.
- Make the tooling crate inherit workspace lints, including the unsafe-code prohibition.
- Run locked workspace checks and include the web application in the WASM gate.
- Correct RTL and rendering test assumptions affected by the product rename;
  update PSD slice iteration for current Clippy without changing decode behaviour.
- Verify the packaging-lint download before extracting it.
- Track development phases, validation evidence and outstanding audit findings.
