# irisui-dock

Binary split-tree docking and floating pane manager for Iris UI (`#![forbid(unsafe_code)]`):

- Split-ratio clamping (`[0.10, 0.90]`), splitter drag & double-click reset, and structural revision tracking (`DockState` / `DockTree`).
- Tab strip reordering, 5-way drop navigator overlays, tear-off floating panes, and workspace layout serialization via `serde`.

Licensed under `MPL-2.0`.
