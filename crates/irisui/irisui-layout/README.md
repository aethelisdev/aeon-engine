# irisui-layout

Taffy-backed flexbox layout adapter for `irisui-core` (`#![forbid(unsafe_code)]`):

- Full-tree solving and $O(M)$ boundary-scoped subtree layout updates (`LayoutEngine::untrack_subtree`, `compute_subtree_layout`).

Licensed under `MPL-2.0`.
