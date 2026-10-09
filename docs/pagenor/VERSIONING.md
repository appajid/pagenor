# Pagenor versioning

`[workspace.package].version` in the root Cargo.toml is the single source of
truth. Every workspace package inherits it. Change it with:

```sh
cargo xtask version set X.Y.Z
```

This updates workspace entries in Cargo.lock without upgrading external
dependencies. Native executables, About, package names and installer metadata
read that version. Check `cargo xtask version` and executable `--version` after
building; run the complete quality gate before committing.

Before 1.0, advance the minor number for a completed feature/distribution group
and reset patch to zero. Use the next patch for maintenance fixes to the same
feature group. Use a prerelease suffix such as `0.5.0-rc.1` for candidates; remove
it only when the intended release validation is complete. Do not increment for
each PR or claim a release merely because the local source version changed.

The local 2026-10-10 checkpoints are 0.2.1 → 0.3.0 for accumulated application
features/release auditing, then 0.4.0 for distribution additions. Pagenor owns its
version sequence independently of upstream. Future maintenance starts at 0.4.1;
the next feature group starts at 0.5.0 (candidate suffixes as appropriate).

The native document format version is separate. Changing the application version
does not migrate or rename `.pgnr` documents. File-format changes require their
own compatibility/migration work and tests.

Local development commits stay local. Publish tags, releases and platform builds
at the owner's milestone, following PHASE_STATUS.md. A GitHub release requires
actual native matrix/package validation; signing/notarization status must be
reported accurately. Never reuse a published version for different artifacts.
