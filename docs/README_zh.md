# Aeon Engine

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-orange.svg)](https://opensource.org/licenses/MPL-2.0)
[![Status](https://img.shields.io/badge/status-in_active_development-green)](https://github.com/aethelisdev/aeon-engine)
[![Rust](https://img.shields.io/badge/rust-v1.99.0-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![YouTube](https://img.shields.io/badge/AeonEngine-FF0000?style=flat&logo=youtube&logoColor=white)](https://youtube.com/@Aeonengine)
[![Instagram](https://img.shields.io/badge/AeonEngine-E4405F?style=flat&logo=instagram&logoColor=white)](https://instagram.com/aeonengine)
[![X / Twitter](https://img.shields.io/badge/AeonEngine-000000?style=flat&logo=x&logoColor=white)](https://x.com/aeonengine)

[English](../README.md) | [Türkçe](README_tr.md) | [日本語](README_ja.md) | **简体中文**

![Aeon Engine Play Mode](../assets/screenshots/aeonengineplaymode.png)

## 什么是 Aeon Engine？
这是一款几乎完全基于 Safe Rust 编写的、全模块化游戏引擎。

## 开发初衷
我一直对 Rust 的安全性与高性能深感兴趣，并希望为自己的游戏打造一款轻量且高度模块化的引擎。市面上的主流引擎对于我的需求来说过于庞大臃肿，而现有的轻量引擎在画面表现上又差强人意。因此，为了兼顾高画质与轻量化，我决定从零开始自研这款引擎。

## Aeon Engine 的目标
在开发过程中设定的主要目标如下：

- **模块化（Modularity）**：引擎中的任何子系统均可随时剥离或替换，完全不受过多依赖项的制约。
- **自适应系统**：启动时实现零负载运行，渲染、物理、音频等模块完全根据用户需求动态启用或禁用。此外，针对低配置设备，计划后续增加一键从内存中彻底清理上述模块的释放机制。
- **Rust 代码与可视化编程深度融合（RedWrite）**：RedWrite 是一套旨在将 Rust 代码与可视化节点编程实时双向绑定的系统。目标是在任何一方所做的改动都能毫秒级同步反馈至另一方。
- **支持在 Android 设备上作为编辑器运行**：在模块化与性能表现稳定后，计划将其扩展为可直接在 Android 移动端进行游戏开发的独立编辑器，以此打破“移动设备无法进行完整游戏开发”的刻板印象。

## 当前开发状态
Aeon Engine 目前正处于积极开发阶段。

大部分核心系统已搭建完成，但 API 及相关特性在后续版本中可能会有所变动。

## 如何运行
请确保本地已正确配置 Rust 环境。

```bash
git clone https://github.com/aethelisdev/aeon-engine.git
cd aeon-engine

# 启动 Aeon Hub（项目管理器与启动器）
cargo run --release

# 或者直接启动编辑器：
cargo run -p ae_engine --release
```

## 设计哲学

- **模块化架构**
- **轻量易用的编辑器**
- **极致性能**
- **Safe Rust**
- **避免过度设计与冗余复杂度**
- **完全开源**

## 开源协议

Aeon Engine 源码基于 **Mozilla Public License 2.0 (MPL 2.0)** 协议授权。

- **独立开发者与创作者**：可自由使用 Aeon Engine 构建商业化、闭源的游戏项目，无需公开自身的游戏逻辑或业务源码。
- **引擎内核修改**：若对引擎底层核心代码 进行了修改，则该部分修改必须依据 MPL 2.0 协议保持开源。
- **商业与企业级授权**：如需定制商业协议、专有引擎分支或商业支持服务，请联系 AethelisDEV。