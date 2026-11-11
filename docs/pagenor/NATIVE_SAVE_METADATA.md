# Native saved-file metadata — P02.12

Existing Save, Save As and Save a Copy still share the staged native writer
introduced in P02.11. Before replacing an existing destination, Pagenor now
preserves macOS Finder tags, readable extended attributes, Unix owner/group and
mode, and macOS extended access-control entries. Linux copies visible extended
attributes, including exposed POSIX ACLs. New documents retain the prior creation
and umask behavior. Document serialization, recovery ownership, history, font
scopes, `.pgnr` and legacy saved paths stay with their existing subsystems.

## Implementation

The staged document remains private while its content is written. Metadata is
applied only after the complete write succeeds, before file sync and replacement.
Changed Unix IDs use safe `std::os::unix::fs::fchown`; unchanged IDs avoid a needless
ownership operation. Mode bits follow ownership restoration. Descriptor-based
`rustix` operations enumerate/copy attributes without reading the old document
bytes. Staging attributes absent from the original are removed. On macOS,
`exacl` copies extended ACLs, including deliberately clearing inherited rules
when the original has no ACL. Linux ACL attributes are copied after mode bits
so chmod does not subsequently alter the restored ACL mask.

The new MIT-licensed `exacl` 0.13.0 dependency is macOS-only. `rustix` 1.1.5 was
already present transitively and is now an explicit macOS/Linux engine dependency.
No GUI dependency, new Pagenor crate, unsafe block, document-format change or
version bump is introduced. The workspace and engine forbid unsafe code.

Attribute names are bounded to 64 KiB. One reusable 1 MiB value buffer and an
8 MiB total copy budget bound memory and work; old document contents are not
loaded again. An oversized, inaccessible or uncopyable attribute/owner/ACL returns
an error before replacement. The previous saved file remains intact and the
existing command path preserves dirty/saved state and recovery. Volumes reporting
no xattr support have no attribute list to copy. No fallback silently drops
metadata after a copy error.

## Regression evidence

Four added tests cover repeated saves retaining real Finder-tag plist bytes,
opaque binary xattrs and ownership/modes; a real differing macOS group;
extended ACL retention and removal of unwanted staged access rules; and a
metadata failure after complete content preparation retaining previous bytes,
cleaning staging and allowing retry. Together with P02.11 there are seven writer
tests. Two regressions run against the preceding mode-only behavior fail because
the tag disappears and the denied-read ACL is lost; both pass with metadata
preservation enabled. The failure case uses deterministic metadata-error injection.
The differing-group test performs actual fchown on an isolated QA file.

All seven final quality gates pass: fmt, check, strict Clippy, workspace tests,
assets, layers and WASM. Total: **1,488 passed, 3 ignored; 20 WASM packages**.

The final signed Mac bundle opened an isolated tagged QA document through File
Open. Type Tool insertion added `METADATA-CHECK`; the dirty star appeared and
Cmd+S removed it. Archive inspection confirms the saved edit and changed content
SHA-256. Finder-tag binary plist, opaque binary xattr, deny-read ACL, UID/GID and
0640 mode match the pre-save values exactly. No staging file remains. Closing
the saved fixture returns to Welcome. Owner documents were not edited.
The file-chooser clipboard operation timed out without inserting the new path;
setting its path field directly completed the test.

Installed app: `/Applications/Pagenor.app`, version **0.3.0**, build
`uncommitted-P02.12`. Strict signature verification passes. App/document ICNS
hashes match their distinct, previously verified assets. Only obsolete generated
engine/UI build archives were cleared for disk space (3,602 MiB); source, assets,
documents and the active build artifacts were retained. Final gates/build ran
with `CARGO_INCREMENTAL=0`. Production writer-only Linux and Windows strict Clippy checks compile the
exact repository module in a small isolated harness, avoiding unrelated cross-
compilation toolchain requirements. These checks do not execute either platform.

## Limits and remaining work

Windows still uses portable permission/staged replacement behavior. Windows ACLs,
alternate data streams, DOS attributes, encryption and creation-time retention
require a dedicated safe implementation and native tests before metadata parity
is claimed. Native Linux execution is also pending. Linux privileged attributes
not visible to the saving user cannot be audited by this implementation.

Creation timestamps, platform file flags, immutable files, hard-link identity,
network/cloud filesystems, hostile concurrent path changes and full power-loss
or parent-directory-sync guarantees remain outside this task. A resource fork
larger than the attribute budget refuses Save; it is not discarded. Metadata
copying can be refused by OS security policy even when content writing is allowed.
Errors leave the last saved document available and the current edit unsaved.
Version remains **0.3.0**. All work stays local; no GitHub publication.

API references inspected alongside the pinned dependency sources:
[standard-library ownership](https://doc.rust-lang.org/std/os/unix/fs/fn.fchown.html),
[rustix descriptor metadata](https://docs.rs/rustix/1.1.5/rustix/fs/index.html),
[exacl](https://crates.io/crates/exacl/0.13.0).
