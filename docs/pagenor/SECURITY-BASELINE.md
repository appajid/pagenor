# Pagenor security baseline

- Upstream source is pinned before modification.
- Local automation/control is OFF by default.
- Control transport, when explicitly enabled, must remain loopback-only.
- Do not add telemetry without an explicit product decision and user-facing opt-in.
- Pin release dependencies and GitHub Actions to immutable revisions where practical.
- Verify hashes for externally downloaded build/test artifacts before execution/extraction.
- Build Pagenor release binaries from source under our own release pipeline.
- Publish SHA-256 checksums for release artifacts.
- Preserve Rust `unsafe_code = "forbid"` unless a separately reviewed exception is required.
