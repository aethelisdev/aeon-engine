---
name: Bug Report
about: Create a report to help us improve Aeon Engine
title: '[BUG] '
labels: ['bug', 'triage']
assignees: ''
---

## Summary
<!-- A clear and concise description of the bug. -->

## Environment
<!-- Run `cargo --version` and check your GPU/Driver details -->
- **Aeon Version/Commit:** 
- **OS:** [e.g. Linux (Arch / Ubuntu 24.04 / Wayland / X11), Windows 10/11, macOS Sonoma/Sequoia (M1/M2/M3/Intel)]
- **Rust Toolchain:** [e.g. 1.98.0]
- **GPU & Driver:** [e.g. NVIDIA RTX 3070 Ti, Driver 610.xx / Mesa]
- **Graphics Backend:** [e.g. Vulkan, DX12, Metal]

## Steps to Reproduce
1. Go to '...'
2. Run 'cargo run --bin ...'
3. Perform action '...'
4. See error

## Expected Behavior
<!-- What you expected to happen -->

## Actual Behavior
<!-- What actually happened (crash, visual artifact, memory leak, panic) -->

## Relevant Crate / Subsystem
<!-- Select/specify affected area (e.g., aeon_render, iris_ui, ecs, scene_graph) -->
- [ ] Render / WGPU Pipeline
- [ ] UI / Iris
- [ ] Scene Graph / Hierarchy
- [ ] ECS / Core
- [ ] Asset Pipeline
- [ ] Other: 

## Logs & Backtraces
<!-- Run with `RUST_BACKTRACE=1 cargo run` if it panicked -->
<details>
<summary>Terminal Output / Backtrace</summary>

```text
// Paste your logs, panics, or WGPU validation errors here