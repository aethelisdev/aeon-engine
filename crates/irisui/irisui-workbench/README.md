# irisui-workbench

High-level desktop application and multi-pane IDE shell framework (`Workbench`, `WorkbenchHost`) for Iris UI (`#![forbid(unsafe_code)]`):

- Declarative menubar, statusbar, panel catalog (`ScopedPanelRegistry`), and surface styling.
- Selective $O(M)$ dirty-panel layout synchronization (`DualLayoutEliminator`), floating dock pane routing, modal stack (`DialogHost`), and typed `ActionBus`.

Licensed under `MPL-2.0`.
