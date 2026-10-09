# Desktop control hardening — P00.5

Upstream #151 is adapted locally at the pinned revision in
[Pagenor provenance](../../PAGENOR_UPSTREAM.md). The shared Rust architecture and
`unsafe_code = "forbid"` remain. The desktop server is off by default;
`--control <port>` deliberately enables it. The same capability token is required
by the desktop server and connected CLI/MCP clients. Headless usage is unchanged.

The [protocol](../control-protocol.md) documents limits and setup. Authentication
runs before requests reach the UI. Request/reply bytes, reflected fields, concurrent
connections, pending UI requests, per-connection requests and synthetic click/drag
work are bounded. Failures to start the transport do not close the document app.
A full request queue rejects before dispatch. UI-reply timeouts acknowledge neither
cancellation nor rollback; a queued command may still execute.

The former client automatically replayed a request after any transport failure.
A loopback regression reproduced duplicate execution when an edit was applied but
its reply was lost. The adapted client reports that uncertainty, drops the connection
and sends no automatic retry. The next explicit call can reconnect with a new ID.
Invalid reply IDs/status and invalid JSON also invalidate the connection. Request
serialization is bounded while writing, and request ID exhaustion returns an error.

Validation covers auth success/failure and actual loopback delivery to the UI queue,
HTTP/junk rejection, byte/CRLF/reflection/request/connection limits, queue pressure,
connected MCP forwarding, lost replies and explicit reconnect, reply framing and ID
exhaustion. The egui handler test sends maximum numeric click/drag counts and checks
its actual queued events, plus zero-count behavior. No user document or running
application is controlled by these fixtures. Test credentials are synthetic.

Native Windows/Linux execution remains part of the milestone matrix; local macOS
and WASM checks do not establish native platform behavior. Existing AI TLS cross-builds
require C cross compilers absent on this Mac. The running development app bundle is
not replaced by this source adaptation. Adobe/dongle validation is cancelled per the
owner's 2026-10-09 instruction; existing native font support stays.

## Local validation — 2026-10-09

`cargo xtask ci` passes all seven gates: formatting, workspace/all-target check,
strict workspace/all-target Clippy, workspace tests, asset attribution, layering and
WASM. Results: **1,262 passed, zero failed, 3 ignored**; **20 WASM packages**;
24 workspace packages with no layering violations. This includes 15 desktop control
transport tests, 22 MCP tests and the actual egui pointer-queue limit regression.
The final gate was run after the bounded UI queue was added. The lost-reply test
failed against the old client, demonstrating mutation replay before the fix.

`cargo build --locked -p pagenor` and the built executable’s `--version` check
also pass. This is local verification. There was no GitHub push or remote CI. Native
Windows/Linux execution and a newly launched desktop bundle remain unverified for
this adaptation; the running app bundle is preserved. No phase or milestone exit
is implied.
