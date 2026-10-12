---
title: Pagenor closes during startup
description: Diagnose a startup or graphics-backend failure.
order: 10
verified: 0.7.0
---

Use the package for your system and architecture, and extract portable archives before
launching. If startup still fails, retain the log and [report the problem](get-help.md)
with the app version and operating system.

Pagenor renders page content on the CPU and uses the graphics backend to present the
interface. The native host tracks failed startup backends and can try an alternative
on a later launch. An explicit `WGPU_BACKEND` environment variable overrides automatic
selection; remove a custom override when checking the default behavior.

On Windows, `WGPU_BACKEND=dx12` selects DirectX 12. On other systems use the supported
backend for that platform. Driver errors should be reported with the log rather than
attributed to a specific hardware model without evidence.
