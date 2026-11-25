# irisui

Retained-mode, GPU-accelerated UI framework built on `wgpu` (`30.0.2`), `taffy`, `glyphon`, and `winit`.

`irisui` re-exports the layered Iris UI workspace crates:

- `irisui-core`: Safe generational arena (`UiTree`), styling, event dispatch, focus, and hit-testing.
- `irisui-layout`: Incremental Taffy flexbox solver with $O(M)$ subtree boundary invalidation.
- `irisui-text`: `cosmic-text` / `glyphon` text measurement, shaping cache, and font management.
- `irisui-wgpu`: Signed-distance-field (SDF) quad/border/shadow pipeline, scissor stack, and layer caching.
- `irisui-widgets`: Declarative `UiScope` DSL, virtualized lists/grids/trees, menus, modals, and property inputs.
- `irisui-dock`: Split-tree docking layout, tab reordering, tear-off windows, and drop navigation.
- `irisui-winit`: `winit` input adapter, HiDPI scaling, and frameless window drag/resize hosting.
- `irisui-workbench`: Multi-pane desktop shell (`Workbench`, `WorkbenchHost`) with `ActionBus` and dialog stack.

## Features

- `bundled-fonts` *(default)*: Forwards `irisui-text/bundled-fonts` for ready-to-render Latin, symbols, math, and emoji fonts.
- `bundled-sans`, `bundled-symbols`, `bundled-emoji`: Granular font embedding controls; disable default features (`default-features = false`) to register fonts manually at runtime.

## Running the Standalone Example

```bash
cargo run -p irisui --example hello_workbench
```

## Platform Support & MSRV

- **Minimum Supported Rust Version (MSRV):** Rust `1.93` (`edition = "2024"`).
- **Target Platforms:** Native desktop (`Windows`, `Linux` X11/Wayland, `macOS`) with `wgpu` (`30.0.2`). Android builds are supported at the widget/core level with `irisui-widgets` clipboard disabled (`default-features = false`). `wasm32` / web targets are not currently supported (`std::time::Instant` and native desktop windowing).

## License

Source code is licensed under `MPL-2.0` (`LICENSE-MPL-2.0`). Embedded Noto font files (when `bundled-*` features are enabled in `irisui-text`) are licensed under SIL Open Font License 1.1 (`SIL OFL-1.1`).
