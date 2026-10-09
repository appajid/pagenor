# Desktop diagnostics — P00.6

Upstream #122 is adapted at the pinned revision in
[Pagenor provenance](../../PAGENOR_UPSTREAM.md). The desktop installs its logger
before argument parsing and before the existing engine panic hook. Diagnostic files
are opened after arguments, so `--version` does not create or rotate logs. Recovery,
font scanning and other existing `log::` records now have a persistent desktop sink.
Open/startup/final-recovery and ordinary control-server failures use it too.

The current file is `pagenor.log`; each launch rotates it to `pagenor.1.log`, then
`pagenor.2.log`. These live under the existing settings directory:

| Platform | Log directory |
|---|---|
| macOS | `~/Library/Application Support/Pagenor/logs/` |
| Windows | `%APPDATA%\Pagenor\logs\` |
| Linux / other Unix | `$XDG_CONFIG_HOME/pagenor/logs/`, or `~/.config/pagenor/logs/` when unset |

Settings/preference paths are unchanged. `PAGENOR_NO_PREFS` skips preferences, not
diagnostics. New Unix directories/files use private permissions; existing directory
permissions are retained. Creating the current file is exclusive after rotation,
so a concurrent new file is not truncated. Rotation is not a transaction between
multiple running instances; records follow their open file handles, and native
Windows/Linux behavior still needs the milestone matrix.

Each current file is capped at **16 MiB**, including any cap notice. The notice
may be shortened when the remaining byte budget is small. Later records continue
on stderr. Each formatted UTF-8 record is capped at **64 KiB**, including metadata
and its newline; long messages are truncated while formatting and marked. Before
file attachment, at most **512 records and 1 MiB total** are buffered; excess
records are counted and a note is emitted if file space permits. A failed file
attachment/write disables the file sink and clears buffering. Diagnostics then use
best-effort stderr; errors do not panic or close the document app. An explicit later
file attachment can retry; there is no automatic write retry.

Defaults are `warn,pagenor*=info`. `RUST_LOG` replaces that filter with comma-separated
levels/targets, e.g. `warn,pagenor_fonts=debug`; `off` suppresses records. Module
names match themselves and `::` descendants; a suffix `*` matches a name prefix.
The most specific match wins. This is a small directive parser, not env_logger's
regex filter syntax. At most 256 comma-separated parts are considered; specs larger
than 64 KiB use the default. Records include UTC milliseconds, severity, thread,
target and the existing emitter's message. Diagnostics can include file paths and
error excerpts.

Control authentication remains off unless `--control` is used. Configured tokens
are not echoed by control diagnostics, and generated capability announcements stay
on direct best-effort stderr, outside the file logger. The existing bounded control
queue and no-replay client behavior remain. Headless CLI logging is unchanged; the
new logger is in the desktop host and adds no dependency or unsafe code.

Tests cover filters, timestamps, rotation, early record replay, count/byte/record/file
limits, UTF-8 formatting, deterministic failed writes and private Unix files. A
fresh-process fixture closes stderr before startup, installs the real global logger
and engine panic hook, then verifies diagnostic files and rotation with configured
and generated control tokens. It uses synthetic credentials and temporary files,
without running the GUI or opening user documents. The strengthened cap regression
failed against the proposal (146 bytes with a 100-byte cap) and passes after the fix.
Logger-only production strict Clippy checks pass for Windows GNU and Linux GNU;
these do not establish native execution or full application cross-builds. Existing
AI TLS dependencies require C cross compilers absent on this Mac. The running app
bundle is preserved. No GitHub publishing or phase/milestone exit is implied.


## Local validation — 2026-10-09

All seven `cargo xtask ci` gates pass: formatting, workspace/all-target check,
strict workspace/all-target Clippy, tests, asset attribution, layering and WASM.
Results: **1,282 passed, zero failed, 3 ignored**; **20 WASM packages** and
**24 workspace packages without layering violations**. The desktop suite has
39 tests, including 19 logging tests, 15 control transport tests and control-port
validation. Unix permission coverage is included in this macOS count.

The production logging module, with the same pinned `log` dependency, separately
passes strict Clippy for `x86_64-pc-windows-gnu` and `x86_64-unknown-linux-gnu`.
These checks deliberately isolate the logger from the app's existing TLS C
cross-compiler requirement. They establish compilation, not native execution,
file-rotation behavior on Windows/Linux, or a complete application cross-build.
No user log directory, preferences, recovery files or running bundle is modified
by the synthetic regression fixtures. No GitHub push or remote CI is performed.

`cargo build --locked -p pagenor` passes on macOS. The built executable returns
`pagenor 0.2.1` for `--version`, with the existing diagnostic file metadata unchanged.
The running development bundle was not replaced.
