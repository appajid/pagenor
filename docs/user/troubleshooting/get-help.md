---
title: Report a problem
description: What to include and where to post when Pagenor misbehaves or crashes.
order: 20
verified: 0.7.0
---

Post in the [issue tracker](https://github.com/appajid/pagenor/issues), and attach the log file. The log is the most useful thing you can send after a crash.

## What to include

- Your Pagenor version and how you installed it (installer, portable, AppImage and so on).
- Your operating system and graphics card.
- What you did, what you expected and what happened.
- The log file `pagenor.log`.

## Where the log is

Pagenor writes `logs/pagenor.log` in its settings folder, beside `ui.json` and `prefs.json`.

| System  | Log folder                                                              |
|---------|-------------------------------------------------------------------------|
| Windows | `%APPDATA%\Pagenor\logs\`                                           |
| macOS   | `~/Library/Application Support/Pagenor/logs/`                       |
| Linux   | `~/.config/pagenor/logs/` (or `$XDG_CONFIG_HOME/pagenor/logs/`) |

Each start moves the previous log to `pagenor.1.log`, and that one to `pagenor.2.log`. The log from a run that crashed survives the next start, so attach `pagenor.1.log` if you have restarted since. The file stops growing at 16 MiB.

## More detail in the log

Set the environment variable `RUST_LOG` to `debug` before starting Pagenor to log more. The default is `info` for Pagenor's own parts and `warn` for everything else.
