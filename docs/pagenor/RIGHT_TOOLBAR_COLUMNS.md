# Right toolbar columns — P11.4

Local Pagenor 0.5.2, 2026-10-11. Owner-requested UI improvement.

Click the chevrons at the top of the right panel toolbar to switch between one
and two columns. Window → Right Toolbar: Double Column uses the same command.
The choice is independent of the left toolbar and is saved in app preferences
and named custom workspaces. Existing preferences/workspaces default to one column.

The rail uses 44 logical pixels for one column and 80 for two. Functional groups,
32-pixel panel buttons, active states, panel menus, dragging and vertical panel
stacks remain. The chevrons stay above the scrolling list so they remain
accessible in smaller windows. Both the legacy and shared docking layouts use
the same renderer; flyouts account for the selected toolbar width.

Implementation: shared Rust egui UI, UI-only command `window.panelsDoubleColumn`,
serialized `panelsDoubleColumn` preference/workspace field. No new platform
adapter, dependencies, assets or unsafe code. No new upstream PR imported.

Validation: three focused regressions pass (legacy/shared layout switching,
short-window scrolling with a pinned switch, preference/workspace compatibility).
All seven `cargo xtask ci` gates pass: 1,878 tests passed, six ignored, no failures;
20 WASM packages. Actual egui one-column, two-column and shared-dock renders were
inspected. The macOS arm64 0.5.2 bundle (`local-0.5.2-right-toolbar`) builds and
passes strict/deep ad-hoc signature checks. Native Windows/Linux execution remains
a milestone check. The separate bundle preserves the owner's running session;
no installed application was replaced and no GitHub publication was performed.
