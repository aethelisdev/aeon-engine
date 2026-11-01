# Contributing to Aeon Engine

Thank you for your interest in contributing to Aeon Engine and the Iris UI framework. We welcome contributions that align with our architectural standards, performance guarantees, and memory-safety principles.

This guide outlines our development workflow, required verification commands, and contribution policies.

---

## 1. Core Principles & Quality Standards

All pull requests submitted to this repository must strictly adhere to the following invariants:

1. **100% Safe Rust Mandate:**  
   The workspace strictly enforces `#![forbid(unsafe_code)]`. Memory safety, pointer validity, and concurrency correctness are guaranteed at compile time. Pull requests introducing `unsafe` code will be rejected.
2. **Zero Compiler & Linter Warnings:**  
   The codebase compiles cleanly with `-D warnings`. Using suppression attributes (such as `#[allow(clippy::...)]` or compiler warning overrides) to circumvent warnings is prohibited; issues must be solved at the root architectural level.
3. **Deterministic Performance & Hot-Path Allocation Lock:**  
   Code executing in per-frame `update`, `render`, `hit_test`, or physics steps must avoid dynamic heap allocations (`format!`, `String`, dynamic `Vec` expansion, or redundant `.clone()` operations).
4. **Data-Driven & Explicit Type System:**  
   Engine and UI decisions are driven by explicit Rust enums, structs, and ECS components. Inspecting string names (`name.contains(...)`) for subsystem logic is prohibited.
5. **SPDX & Copyright Header:**  
   Every new or modified source file (`.rs`, `.wgsl`) must begin with the required license header:
   ```rust
   // SPDX-License-Identifier: MPL-2.0
   // Copyright (c) <YEAR> AethelisDEV and Aeon Engine contributors.
   ```
   *(Use the file creation year, e.g. 2026, 2027).*

---

## 2. Getting Started & Local Setup

### Prerequisites
- Latest stable Rust toolchain (refer to `rust-toolchain.toml`).
- Vulkan, Metal, or DirectX 12 compatible GPU / driver for WGPU rendering.
- Standard platform dependencies for windowing (`winit`) and clipboard (`arboard`).

### Building and Running
```bash
# Clone the repository
git clone https://github.com/aethelisdev/aeon-engine.git
cd aeon-engine

# Build the entire workspace
cargo build --workspace

# Launch Aeon Hub (Project Manager)
cargo run --release

# Alternatively, launch the Editor directly:
cargo run -p ae_engine --release
```

---

## 3. Mandatory Verification Suite

Before submitting a pull request, you **must** run the following verification commands locally:

1. **Code Formatting:**
   ```bash
   cargo fmt --check
   ```
   *(Must be 100% compliant with standard rustfmt).*

2. **Linter Inspection:**
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```
   *(Must complete with 0 warnings and 0 errors).*

3. **Test Suite:**
   ```bash
   cargo test --workspace
   ```
   *(All unit and integration tests across all workspace crates must pass).*

---

## 4. Submitting a Pull Request (PR Workflow)

> **Important**: Before investing time into substantial new features, large subsystem additions, or architectural refactors, please open a GitHub Issue or Discussion first to align with maintainers.

1. **Create a Topic Branch:**  
   Branch off the latest `main` branch with a descriptive name:
   ```bash
   git checkout -b feat/describe-feature
   # or
   git checkout -b fix/describe-bug
   ```
2. **Commit Standards:**  
   Write concise, technical commit messages following the Conventional Commits specification:
   - `feat(scope): add new capability`
   - `fix(scope): resolve issue description`
   - `refactor(scope): streamline internal implementation`
   - `docs(scope): update documentation`
3. **Complete the PR Template:**  
   When opening a pull request on GitHub, fill out all fields and checkboxes in the provided template (`.github/PULL_REQUEST_TEMPLATE.md`).

---

## 5. Contributor License Agreement (CLA) & Intellectual Property

By submitting a pull request to Aeon Engine, you confirm that:
1. The contribution consists entirely of your own original work (or work appropriately licensed under compatible open-source terms).
2. You agree to license the contribution under the **Mozilla Public License 2.0 (MPL-2.0)**.

### Why is this required?
- **Open Source Commitment:** The core of Aeon Engine will always remain free and open source under the MPL-2.0 license.
- **Platform Integrity & Console Ports:** This agreement protects the engine and its community from patent/copyright disputes, and legally enables closed/proprietary SDK integrations (such as console platform ports for PlayStation, Nintendo, and Xbox).

---

## 6. Security Vulnerabilities

Please **do not report security vulnerabilities through public GitHub Issues or Pull Requests**. For responsible, confidential disclosure procedures, refer to our [Security Policy](SECURITY.md).

