# Aeon Assets

Headless asset directory scanning, safe file operations, external asset ingestion, CPU thumbnail rasterization, and background `rayon` parsing pipelines.

## Scope
- **In scope**: Workspace content discovery (`scanner`), `import_asset_path` / `import_dropped_paths`, glTF/`FBX2glTF` background tasks, and `.aeworld` scene asset loading.
- **Out of scope**: `irisui` UI panels, `winit` windowing, `rfd` dialogs, and editor undo history.
