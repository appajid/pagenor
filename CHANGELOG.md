# Pagenor changelog

Versions below are local development checkpoints; no new release is published.

## 0.3.0 — 2026-10-10

- Record the accumulated Pagenor work since 0.2.1: lazy native/system font refresh,
  scoped/variable-font resolution, clipboard and recovery reliability, `.pgnr`,
  supplied owner branding/splash, editor/style/PDF controls, advanced data merge,
  typography/threading, INDD subset import, EPS/PowerPoint and reviewed AI edits.
- Retain and audit all 32 inherited application/workflow contributions listed in
  the upstream v0.4.0 release. Adapt distribution/documentation #132/#135
  under the same 0.3.0 version, as requested by the owner.
- Add FreeBSD tar packaging, Flatpak bundles for x86_64/aarch64 and AppImage
  zsync update routing to Pagenor’s repository, plus download/build documentation.
  Package notices include the owner branding rights and bundled-font licences.
  Native package execution and publication remain milestone checks.
- Keep the Rust architecture, unsafe-code prohibition and native document format 3.

See [release audit](docs/pagenor/UPSTREAM_040_AUDIT.md),
[phase evidence](PHASE_STATUS.md) and [version policy](docs/pagenor/VERSIONING.md).
