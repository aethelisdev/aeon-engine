# Aeon Plugin Host

Native shared-library plugin loader (`libloading`) and hot-reload coordinator (`PluginManager`, `NativePluginBackend`).

## Scope
- **In scope**: Workspace plugin discovery, ABI compatibility checks, cache-bypass dynamic loading, and static `ScriptingBackend` registration.
- **Out of scope**: Plugin interface definitions (`ae_plugin_api`) and example gameplay code (`game_logic`).
