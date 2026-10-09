# MCP server

`pagenor-cli mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server that lets Claude and
other agents drive Pagenor: create documents, lay out frames and text, apply styles, place images, and look at
the result as PNG images. It speaks newline-delimited JSON-RPC 2.0 on stdin/stdout (protocol `2025-06-18`, also
`2025-03-26` and `2024-11-05`); logs go to stderr. The implementation is the `pagenor-mcp` crate.

## Register it

Build the CLI (`cargo build --release -p pagenor-cli`), then:

```sh
# Headless: an in-process engine, no window
claude mcp add pagenor -- /path/to/target/release/pagenor-cli mcp

# Drive the running desktop app with a shared random capability.
export PAGENOR_CONTROL_TOKEN="$(openssl rand -hex 32)"
pagenor --control 7979 &
claude mcp add pagenor-app -- /path/to/target/release/pagenor-cli mcp --connect 7979
```

Other MCP clients use the same command line, e.g. in a JSON config:

```json
{"mcpServers": {"pagenor": {"command": "/path/to/pagenor-cli", "args": ["mcp"]}}}
```

## Modes

| | Headless (`mcp`) | Connected (`mcp --connect PORT` or `LOOPBACK:PORT`) |
|---|---|---|
| Engine | In-process `pagenor_engine::Session`, starts with an empty Letter document (`--sample` opens the sample magazine) | The app's session, through its loopback control channel ([control-protocol.md](control-protocol.md)) |
| Rendering | `pagenor-render` (CPU) | The app's renderer |
| Window tools (`screenshot`, `click`, `drag`, `menu_list`, `ui_inspect`, `ui_set`, `dialog_*`) | Return an error explaining how to connect | Work |
| UI-only commands via `execute` (`view.*`, `window.*`, `help.*`, `edit.dynamicSpelling`, and `app.*` except `app.links`) | Error | Work |

In connected mode every tool maps to a control-channel method (`engine.execute`, `ui.render`, `ui.pointer`, …), so
what the agent does shows up live in the window. Both processes require the same
`PAGENOR_CONTROL_TOKEN`. Ensure the MCP client receives this environment variable
when it launches the CLI; registration alone does not supply it. Missing/wrong tokens
fail before dispatch. Requests/replies are bounded and destinations are loopback only.
Lost replies are reported without replaying commands; a subsequent explicit call can
reconnect. Headless mode has no token requirement.

## Coordinates

Points (1/72 in), y down, in **spread** space: on a single-page spread the page's top-left is (0, 0); on a facing
spread the right page starts at x = page width. `inspect_document` lists page bounds per spread. Page indices in
`render_page` / `export_png` are 0-based. `pointer` events in connected mode may use `"space": "screen"` (egui points,
see `ui_inspect` → `canvasRect`).

## Text offsets

Positions in a story's text are **UTF-8 byte offsets**, not character counts: `anchor` / `focus` of `text.select`,
`start` / `end` of `story.replaceRange` and of `find.find` matches, the `pos` that `text.placeCaret`, `text.insert`
and `text.move` return, a story's `length` in `get_story` and `inspect_document`, and its `overset` position in
`get_story`. A character outside ASCII takes 2 to 4 bytes (é 2, — and は 3), so the story `はただ商店` has length 15
and its second character starts at 3. An offset inside a character snaps back to the start of that character.

## Tools

| Tool | What it does |
|---|---|
| `list_commands` | Every command (id, label, menu path, shortcut, params, enabled), with `filter` / `enabledOnly` |
| `execute` | Run any command: `{command, params}` |
| `batch` | Run `commands: [{command, params}]` (or `script` text, one `command.id {json}` per line) in order, stopping at the first error (reports `failedIndex`); `"$N.path"` parameters use earlier results ([agents.md](agents.md)) |
| `inspect_document` | Pages, spreads, items (ids, bounds, fill, story), stories (overset), layers, styles, swatches, selection |
| `get_story` | Text, frames, paragraphs, lines, overset of a story (`story` or text `frame`) |
| `set_story_text` | Replace a story's text (`\n` = new paragraph) |
| `new_document` | `file.new` (preset, width, height, pages, facingPages, columns, gutter, margins, bleed, title) or `sample: true` |
| `open_document` / `save_document` | Native `.pagenor` files |
| `place_image` | Image from `path` or `base64` into `frame`, the selected frame, or a new frame at x, y, width |
| `render_page` | Render a page → MCP image content (PNG, base64); optional `path` to also save it |
| `export_png` | Write a page as PNG (or JPEG by extension) |
| `screenshot` | Whole app window → image content (connected only; captured to a temp file and read back) |
| `select_tool` | Activate a tool (`selection`, `type`, `rectangleFrame`, `line`, …) |
| `pointer` | Mouse gesture events (`down`/`drag`/`up`/`move`/`doubleclick`) for the active tool, optional `tool` |
| `key` | Key press with modifiers (headless: active tool → command shortcuts → tool shortcuts) |
| `type_text` | Type at the text insertion point (or into the focused dialog field in the app) |
| `click` / `drag` | Real pointer input at screen points (connected only) |
| `menu_list`, `ui_inspect`, `ui_set` | Menu tree, UI state, change UI state (connected only) |
| `dialog_set` / `dialog_confirm` / `dialog_cancel` | Fill in and close the open dialog (connected only) |

Tool failures come back as results with `isError: true` and a message the model can act on; protocol errors use
JSON-RPC error codes. Resources: `pagenor://document` (document.inspect) and `pagenor://commands`.

## Example

```text
new_document     {"preset": "A4", "margins": 36}
execute          {"command": "frame.create", "params": {"rect": [36, 36, 559, 200], "content": "text", "text": "Hello"}}
                 → {"id": 8, "story": 9}
set_story_text   {"story": 9, "text": "Summer Fair\nSaturday in the park"}
execute          {"command": "type.char", "params": {"size": 28}}
place_image      {"path": "/tmp/photo.jpg", "x": 36, "y": 220, "width": 523}
render_page      {"page": 0}       → PNG image
```

Or with the Type tool, the way a user would:

```text
pointer    {"tool": "type", "events": [{"kind": "down", "x": 72, "y": 72}, {"kind": "drag", "x": 300, "y": 200}, {"kind": "up", "x": 300, "y": 200}]}
type_text  {"text": "Hello"}
key        {"key": "Enter"}
key        {"key": "Escape"}
```

## Community links

`app.links` returns Pagenor's repository (https://github.com/appajid/pagenor) and issue tracker. The project-page aliases `website` and `appPage` use that repository until a separate website is configured. `discord` is null because no Pagenor community destination has been supplied. `help.github`, `help.issues`, `help.appPage`, `help.website` and `help.app {app: "pagenor"}` open configured destinations; `help.discord` returns an unavailable error. `help.about {tab?}` shows the About window (tabs `about`, `contributors`, `models`). The MCP server's `serverInfo.websiteUrl` is the project page.
