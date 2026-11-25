# irisui-winit

Platform windowing bridge between `winit` (`0.30.9+`) and `irisui-core` / `irisui-widgets` (`#![forbid(unsafe_code)]`):

- Keyboard/mouse/scroll normalization (`WinitState`), HiDPI scaling, and hardware cursor mapping.
- Frameless window drag/double-click maximize (`WindowDragHost`) and 8-directional border resize hosting (`FramelessWindowHost`).

## Platform Support & MSRV

- **MSRV:** Rust `1.93` (`edition = "2024"`).
- **Platforms:** Native desktop (`Windows`, `Linux` X11/Wayland, `macOS`); `wasm32` is not currently supported.

Licensed under `MPL-2.0` (`LICENSE-MPL-2.0`).
