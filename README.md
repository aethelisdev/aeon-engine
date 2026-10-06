# Aeon Engine


[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-orange.svg)](https://opensource.org/licenses/MPL-2.0)
[![Status](https://img.shields.io/badge/status-in_active_development-green)](https://github.com/aethelisdev/aeon-engine)
[![Rust](https://img.shields.io/badge/rust-v1.99.0-orange?logo=rust\&logoColor=white)](https://www.rust-lang.org/)
[![YouTube](https://img.shields.io/badge/AeonEngine-FF0000?style=flat\&logo=youtube\&logoColor=white)](https://youtube.com/@Aeonengine)
[![Instagram](https://img.shields.io/badge/AeonEngine-E4405F?style=flat\&logo=instagram\&logoColor=white)](https://instagram.com/aeonengine)
[![X / Twitter](https://img.shields.io/badge/AeonEngine-000000?style=flat\&logo=x\&logoColor=white)](https://x.com/aeonengine)

**English** | [Türkçe](docs/README_tr.md) | [日本語](docs/README_ja.md) | [简体中文](docs/README_zh.md)

![Aeon Engine Play Mode](./assets/screenshots/aeonengineplaymode.png)

## What is Aeon Engine?
It is a fully modular game engine written almost entirely in safe Rust.

## Why was Aeon Engine Created?
I was interested in Rust's safety and performance, and I wanted a lightweight, modular engine for my own game. The other engines on the market were too heavy for my needs, while the lightweight ones were not visually satisfying. Therefore, I wanted an engine that was both very high quality visually and lightweight. That is why I started developing my own engine.

## What is the Purpose of Aeon Engine?
The goals I set during the development process are as follows:

- **Modularity**: The ability to easily change or completely remove any system of the engine without being exposed to too many dependencies if I want to remove or change it.

- **A system that adapts according to user preferences**: Initially, the system starts without any load, and modules such as rendering, physics, and audio are enabled or disabled according to the user's needs. I am also considering adding a button in the long term for low-end computers that allows these features to be cleared from RAM.

- **Combining Rust code with visual programming (RedWrite)**: RedWrite is a system that I aim to combine Rust code with visual programming simultaneously. My goal is for changes made on both sides to be reflected instantly and bidirectionally to each other.

- **The ability to work as an editor on Android devices**: If I achieve good results and high performance in a modular, working system, I aim to break the perception that game development is not possible on Android devices by developing my Aeon engine as an editor that can directly produce games on Android devices.

## Current Status
Aeon Engine is currently under active development.
Most of the core systems have been developed, but the API and features may change over time.

## How to Run
Make sure Rust is installed on your machine.

```bash
git clone https://github.com/aethelisdev/aeon-engine.git
cd aeon-engine

# Start Aeon Hub (Project Manager & Launcher)
cargo run --release

# or launch the Editor directly:
cargo run -p ae_engine --release
```

## Design Philosophy

* **Modular design**
* **Simple editor**
* **Performance**
* **Safe Rust**
* **Avoiding unnecessary complexity**
* **Open Source**

## License

The Aeon Engine source code is licensed under the **Mozilla Public License 2.0 (MPL 2.0)**.
* **Independent developers and game creators**: You are free to create commercial, closed-source games using Aeon Engine. You do not need to share your game logic or game source code.
* **Engine modifications**: If you modify the main engine files, these engine modifications must be shared under the MPL 2.0 license.
* **Commercial and enterprise licensing**: For custom commercial licensing, custom engine forks, or enterprise support options, contact AethelisDEV.
