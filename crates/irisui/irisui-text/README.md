# irisui-text

GPU typography, font shaping cache, and caret hit-testing engine built on `cosmic-text`, `glyphon`, and `wgpu` (`#![forbid(unsafe_code)]`).

## Embedded Fonts & Size Control

By default, `bundled-fonts` embeds five Google Noto TTF font binaries (~3.9 MB total) into `TextSystem::new()`:

- `bundled-sans`: `NotoSans-Regular.ttf` (~556 KB)
- `bundled-symbols`: `NotoSansSymbols-Regular.ttf`, `NotoSansSymbols2-Regular.ttf`, `NotoSansMath-Regular.ttf` (~1.48 MB)
- `bundled-emoji`: `NotoEmoji-Regular.ttf` (~1.9 MB)
- `bundled-fonts` *(default)*: Enables `bundled-sans`, `bundled-symbols`, and `bundled-emoji`.

To omit embedded font binaries completely, set `default-features = false` on `irisui-text` (or `irisui`) and register fonts dynamically via `TextSystem::load_static_font`, `TextSystem::load_font_bytes`, or `TextSystem::load_system_fonts`.

## Licenses

- Rust source code is licensed under `MPL-2.0`.
- The `.ttf` font files under `assets/fonts/` belong to the Google Noto Fonts collection and are licensed under the **SIL Open Font License, Version 1.1** (`assets/fonts/OFL.txt`, `assets/fonts/NOTICE.md`).
