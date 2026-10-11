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

The 2026-10-10 checkpoint advanced 0.2.1 → 0.3.0. On 2026-10-11 the owner
lifted the version hold and authorised relevant minor updates. The local version
advances to **0.4.0** for accumulated Welcome/recent-document, keyboard/input,
new-document, chapter/section/tab-stop and editor improvements, plus the branded
macOS installer. This is a local development checkpoint, not a published stable
release. Pagenor owns its version sequence independently of upstream.

The next completed selected upstream group advances **0.4.0 → 0.5.0** on
2026-10-11: 47 proposals plus two prerequisite changes, full local quality gates
and an inspected Mac build. This is a local development checkpoint; native
Windows/Linux validation and publication remain separate milestone work.

The native document format version is separate. Changing the application version
does not migrate or rename `.pgnr` documents. File-format changes require their
own compatibility/migration work and tests.

Routine development stays local. The owner authorized a **v0.3.0 development
prerelease** on 2026-10-10, with matching source, the verified Apple-silicon Mac
development build, release notes and checksums. GitHub’s prerelease flag does
not change the 0.3.0 Cargo version. Release notes must identify pending native
platform/package validation and signing/notarization status accurately. A stable
cross-platform release requires the actual native matrix/package validation at
the milestone, following PHASE_STATUS.md. Never reuse a published version for
different artifacts.

The next maintenance checkpoint is **0.5.1**, for canvas/table/panel usability and
a larger status bar. This continues the 0.5 editor group; the native document
format remains 3. Local verification is recorded in EDITOR_USABILITY.md.
