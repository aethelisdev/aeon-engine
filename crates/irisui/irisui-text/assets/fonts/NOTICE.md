# Embedded Font Notices

The `.ttf` font files in this directory belong to the Google **Noto Fonts** family and are distributed under the **SIL Open Font License, Version 1.1** (`OFL-1.1`), included in full in `OFL.txt`.

| File | Font Family | Feature Flag (`irisui-text`) |
| :--- | :--- | :--- |
| `NotoSans-Regular.ttf` | Noto Sans | `bundled-sans` |
| `NotoSansSymbols-Regular.ttf` | Noto Sans Symbols | `bundled-symbols` |
| `NotoSansSymbols2-Regular.ttf` | Noto Sans Symbols 2 | `bundled-symbols` |
| `NotoSansMath-Regular.ttf` | Noto Sans Math | `bundled-symbols` |
| `NotoEmoji-Regular.ttf` | Noto Emoji | `bundled-emoji` |

All five feature flags are grouped under the default `bundled-fonts` feature. Disable default features (`default-features = false`) in `irisui-text` (or `irisui`) to omit embedded font binaries completely and register custom or OS fonts via `TextSystem::load_font_bytes`, `TextSystem::load_static_font`, or `TextSystem::load_system_fonts`.
