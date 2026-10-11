# Control protocol

The desktop control channel is off by default. Start it explicitly with
`--control <port>`. It listens on `127.0.0.1` and speaks JSON lines: one request
object per line, one reply per line. Headless scripts do not need this transport.

Every request requires the same capability token as the app. Set
`PAGENOR_CONTROL_TOKEN` to a random secret of 32–1,024 UTF-8 bytes in the app and
client environments. If unset in the app, a fresh random 64-character hex token
is generated and printed once to stderr for the client to use. Invalid configured
tokens or startup failures disable the channel and show an app status message.
Tokens are not included in documents, preferences, error responses or request logs.

```sh
export PAGENOR_CONTROL_TOKEN="$(openssl rand -hex 32)"
pagenor --control 7979 &
pagenor-cli app --port 7979 --method ui.inspect '{}'
```

A client sends the token as a top-level field on every request:

```json
{"id": 1, "token": "<same capability token>", "method": "engine.execute", "params": {"command": "frame.create", "params": {"rect": [36, 36, 300, 200], "content": "text"}}}
{"id": 1, "ok": true, "result": {"id": 10, "story": 11}}
```

Missing or wrong tokens get `unauthorized` and close the connection before UI
or engine dispatch. Malformed input also closes the connection before later lines
can run, including an HTTP request followed by a JSON body. Blank lines are skipped.
`method` must be nonempty and at most 256 bytes; `params` is optional. Reflected
IDs are numbers, null or strings of at most 128 bytes; other IDs become null.

The server bounds request lines to 4 MiB (excluding CR/LF), concurrent connections
to 16, queued UI requests to 16 and authenticated requests per connection to
4,096. A full UI queue rejects the request before dispatch. Socket reads and writes
have a 30-second idle timeout; waiting for the app reply has a 60-second timeout.
A reply timeout does not cancel an already queued request; it may still execute.
These are operation/idle limits, not a total connection deadline. The final
EOF-terminated request remains supported. Click count is clamped to 1–16 and drag
steps to 1–10,000.

Connected CLI/MCP clients require `PAGENOR_CONTROL_TOKEN`, allow only loopback
destinations, serialize requests within 4 MiB and bound newline-terminated replies
to 64 MiB including the newline. A reply must match the request ID. A lost or invalid
reply drops the connection and returns an error: a command may already have run,
so the client never automatically replays it. A subsequent explicit call can reconnect.
Transport: `apps/pagenor/src/control_server.rs`; client: `crates/mcp/src/backend.rs`.

| Method | Params | What it does |
|---|---|---|
| `engine.execute` / `ui.menu.invoke` | `{command, params}` | Run any engine or UI command (see `engine.commands`). `ui.menu.invoke` without `params` acts like choosing the menu item: a command labelled "…" that takes parameters opens its dialog |
| `engine.commands` | — | Every command: id, label, menu path, shortcut, params doc, enabled |
| `document.inspect` | — | Pages, spreads, items, stories (overset), styles, swatches, selection |
| `ui.inspect` | — | Tool, UI state, view (zoom/origin), canvas rect, perf |
| `ui.menu.list` / `ui.tool.list` | — | Menu tree / Tools panel groups |
| `ui.tool.select` | `{tool}` | Select a tool (`selection`, `type`, `rectangleFrame`, …) |
| `ui.pointer` | `{events:[{kind: down\|drag\|up\|move\|doubleclick\|tripleclick, x, y, space?: "screen"\|"canvas", clicks?}], mods?}` | Drive the active tool through the same code path as the mouse; `clicks` on a `down` is the press count (3 = triple click); returns the selection, the tool and its cursor at the last event (e.g. `LoadedText`, `ThreadLink`, `Unthread`) |
| `ui.key` / `ui.text` | `{key, shift?, alt?, cmd?}` / `{text}` | Synthetic keyboard input (typing into a text frame) |
| `ui.move` / `ui.click` / `ui.drag` | screen points, `button?: left\|right\|middle` | Real egui pointer input — reaches every widget, menu and panel |
| `ui.set` | `{brightness?, panel?, rulers?, guides?, frameEdges?, baselineGrid?, textThreads?, screenMode?, zoom?, page?, fit?}` | UI state |
| `ui.dialog.open` | `{id, fields?}` | Open a dialog by id (e.g. `paragraphStyleOptions` or `characterStyleOptions` with `{name, section}`, or `{new: true}` for a new style) |
| `ui.dialog.set` / `ui.dialog.confirm` / `ui.dialog.cancel` | `{field, value}` | Fill and confirm the open dialog |
| `ui.resize`, `ui.focus` | | Window control |
| `ui.screenshot` | `{path?}` | PNG of the whole window |
| `ui.render` | `{path?, page?, scale?, bleed?}` | Render a page headlessly (PNG; base64 if no path) |
| `app.open` / `app.save` / `app.export` / `app.quit` | | Files |

On desktop, `app.quit` checkpoints unsaved documents before requesting window
close. Recovery failures return `ok: false` and keep Pagenor open. A successful
response acknowledges the request; the subsequent close event checks recovery
again and can cancel closing if a later write fails. `app.quit` is also available
as a UI command through `engine.execute` and `ui.menu.invoke`.

Text `edit.copy` and `edit.cut` invoked through these command routes publish text
through the host's normal egui clipboard output. The command response does not
acknowledge a completed platform clipboard write. Object copies remain internal.

Data merge runs through `engine.execute` like any command: `data.source.select`, `data.fields`, `data.placeholder.add` /
`.remove`, `data.source.affix`, `data.join`, `data.sort`, `data.options` (including `skipWarnings`), `data.preview` /
`data.preview.stop`, `data.grid.create`, `data.merge`. `data.merge` creates and activates a new merged document; the
template stays as it was. `pdf` on that command writes the merged pages with the existing PDF export. Sources are CSV,
TSV, JSON, `.xlsx`, and `.xls` ([agents.md](agents.md#data-merge)).

Headless window screenshots (locked screen, hidden window): `cargo run -p pagenor-ui-egui --example ui_shot -- script.jsonl`, where each line is one of the requests above, `{"shot": "/abs/out.png"}` or `{"steps": n}` (renders the whole UI offscreen with wgpu).

The MCP server (`pagenor-cli mcp`) wraps the same methods for Claude and other agents.

AI window, background tasks and review commands are documented in [AI assistant](ai-assistant.md).
