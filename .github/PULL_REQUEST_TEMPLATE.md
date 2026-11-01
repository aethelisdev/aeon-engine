<!--
  Aeon Engine Pull Request Template
  Please ensure your PR strictly adheres to the project's standards and checklist below.
-->

## Description
<!-- Provide a clear, concise technical description of your changes, the rationale behind them, and what problem they resolve. -->

Fixes / Relates to: #<!-- Issue number if applicable -->

## Type of Change
<!-- Select all that apply -->
- [ ]  Bug fix (non-breaking change fixing an existing issue)
- [ ]  New feature (non-breaking change adding new engine/UI capability)
- [ ]  Breaking change (fix or feature causing existing APIs or behaviours to break)
- [ ]  Performance optimization / Refactoring (no functional API changes)
- [ ]  Documentation / Tests only

## Affected Subsystems & Crates
<!-- Select all affected crates -->
- [ ] `irisui` (Autonomous Crates.io UI Framework: `iris-core`, `iris-widgets`, `iris-dock`, `iris-workbench`, `iris-wgpu`, etc.)
- [ ] `ae_core` (ECS, State Management, Event Bus)
- [ ] `ae_renderer` (AeonGPU, Shaders, PBR, Shadows, Render passes)
- [ ] `ae_2d` (2D Engine Subsystem: `ae-2d-core`, `ae-2d-render`, `ae-2d-physics`, `ae-2d-animation`, `ae_2d`)
- [ ] `ae_physics` (3D Rapier3D integration, Kinematic Character Controller, Colliders)
- [ ] `ae_animation` / `ae_audio` (3D Skeletal animation, Audio playback)
- [ ] `ae_editor` / `ae_editor_ui` (3D/2D Gizmos, Editor Views, Declarative Tooling)
- [ ] `ae_engine` / `ae_launcher` (Application runtime, Dimension Coordinator, Aeon Hub)
- [ ] Build & Tooling (Cargo configs, CI/CD, Scripts)

---

## Strict Quality & Safety Checklist
<!-- 
  All items in this checklist are MANDATORY for pull requests to be reviewed and merged.
  Check each box once verified locally.
-->

- [ ] **100% Safe Rust Mandate**: Zero `unsafe` blocks or keywords are introduced anywhere in the codebase.
- [ ] **Zero Clippy Warnings**: Verified via `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings, 0 errors).
- [ ] **Formatting Compliance**: Verified via `cargo fmt --check` (100% compliant with rustfmt).
- [ ] **Test Suite Passing**: Verified via `cargo test --workspace` (all unit and integration tests passed).
- [ ] **SPDX & Copyright Header**: Every new and modified source file (`.rs`, `.wgsl`) begins with the required license header (use file creation year, e.g. 2026, 2027):
  ```rust
  // SPDX-License-Identifier: MPL-2.0
  // Copyright (c) <YEAR> AethelisDEV and Aeon Engine contributors.
  ```
- [ ] **Documentation Quality**: Every new/modified public module, struct, enum, trait, and function includes comprehensive `///` doc-comments explaining purpose, invariants, and failure modes.
- [ ] **Zero Hot-Loop Allocations**: No hidden heap allocations (`format!`, `String`, dynamic `Vec`, `.clone()`) introduced in per-frame `update`/`render` hot paths.
- [ ] **No Magic String Heuristics**: Engine and UI decisions are 100% data-driven with explicit Rust enums and types, without inspecting string names (`name.contains(...)`).
- [ ] **No Linter Suppressions**: No `#[allow(...)]` or Clippy suppression attributes added to bypass compiler/linter warnings.

---

## Contributor License Agreement (CLA) & Intellectual Property
- [ ] **Licensing & Originality**: I confirm that this pull request consists entirely of my own original work (or work appropriately licensed under compatible open-source terms), and I agree to license this contribution under the **Mozilla Public License 2.0 (MPL-2.0)**.

> **Why is this required?**  
> Aeon Engine's core will always remain free and open source under MPL-2.0. This agreement protects the engine and its contributors from patent/copyright disputes, and legally enables closed/proprietary SDK integrations (such as console platform ports for PlayStation, Nintendo, and Xbox).

