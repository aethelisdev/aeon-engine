# irisui-widgets

Declarative `UiScope` widget DSL and retained UI controls for Iris UI (`#![forbid(unsafe_code)]`):

- Containers (`RowBuilder`, `ColumnBuilder`), scroll areas (`ScrollAreaView`), 2D pan-and-zoom viewports, and collapsing sections.
- Virtualized lists, grids, data tables, and retained/virtualized tree views.
- Property editors (numeric scrubbers, RGB/HSV color picker popups, comboboxes, single-line & multiline text inputs), cascading menus, and modals.
- Lightweight `SimpleUiHost` (`SimpleUiState`) combining `UiTree`, `LayoutEngine`, `TextSystem`, and `IrisSurfaceHost` for standalone overlays without `winit` or workbench docking.

Optional default feature `clipboard-os` integrates OS clipboard access via `arboard` on non-Android native desktop platforms.

- **MSRV:** Rust `1.93` (`edition = "2024"`).
- **Platform Notes:** Native desktop (`Windows`, `Linux`, `macOS`); Android is supported when `default-features = false` omits `arboard`. `wasm32` is not currently supported (`std::time::Instant`).

Licensed under `MPL-2.0` (`LICENSE-MPL-2.0`).
