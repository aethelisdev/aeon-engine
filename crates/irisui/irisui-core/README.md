# irisui-core

Core retained-mode UI data structures for the Iris UI framework:

- Generational `SlotMap` widget arena (`UiTree`) with `#![forbid(unsafe_code)]`.
- Flexbox/style tokens, themes, and layer priorities.
- Pointer/keyboard/drag/scroll dispatch engine (`EventState`), tag-scoped interaction queries, and numeric undo/redo stacks.

Licensed under `MPL-2.0`.
