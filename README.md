# Aeon Engine


[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-orange.svg)](https://opensource.org/licenses/MPL-2.0)
[![Status](https://img.shields.io/badge/status-in_active_development-green)](https://github.com/aethelisdev/aeon-engine)
[![Rust](https://img.shields.io/badge/rust-v1.97.1-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![YouTube](https://img.shields.io/badge/AeonEngine-FF0000?style=flat&logo=youtube&logoColor=white)](https://youtube.com/@Aeonengine)
[![Instagram](https://img.shields.io/badge/AeonEngine-E4405F?style=flat&logo=instagram&logoColor=white)](https://instagram.com/aeonengine)
[![X / Twitter](https://img.shields.io/badge/AeonEngine-000000?style=flat&logo=x&logoColor=white)](https://x.com/aeonengine)


Aeon Engine Rust Game Engine & Editor

## What is Aeon Engine?
It is a fully modular game engine written almost entirely in safe Rust.

## Why was Aeon Engine created?
I was interested in Rust’s security and performance, and I wanted a lightweight, modular engine for my own game. The other engines on the market were far too heavy for my needs, so I started developing my own engine.

## What is the goal of the Aeon Engine?
The goals I’ve set during the development process are as follows:

 - **100% Modularity**: the ability to disassemble and replace any system in the engine whether I want to remove it or replace it without being subject to too many dependencies 
 
 - **A system that adapts to the user's preferences**: initially, the system will start up with no load, and modules such as rendering, physics, and sound will be enabled or disabled based on the user's needs. I'm also considering adding a button for low end computers that allows them to clear RAM in the long term
 
 - **Combining visuals and Rust code (Redprint)**: redprint is a feature I’ve named that aims to synchronously combine Rust code and visual programming; my goal is for changes made on either side to be reflected on the other side.
 
 - **The ability to work as an editor on Android devices**: If I achieve good results and performance in a modular, disengageable system, I aim to break the perception that game development isn’t possible on Android by making my Aeon engine capable of directly producing games on Android devices.
 
 - **Creating games using prompts (Vibe Gaming)**: I want to enable my engine to create games using prompts by integrating the latest AI API. This way, I plan to build a useful editor for anyone with game ideas who wants to give it a try.

## Current Status
 Aeon Engine is currently under active development
 
 Most of the core systems have been implemented, but the API and features are subject to change.

 ## Design Philosophy
 
- **Modular design**
- **Simple editor**
- **Performance**
- **Safe Rust**
- **To avoid unnecessary complexity**
- **Open Source**

##

## Licensing

Aeon Engine source code is licensed under the **Mozilla Public License 2.0 (MPL 2.0)**.

- **For Indie Developers and Game Creators**: You are free to build commercial, closed-source games using Aeon Engine. You do not need to share your game logic or game source code.
- **Engine Modifications**: If you modify the core engine files (`.rs`), those engine modifications must be shared under the MPL 2.0 license.
- **Commercial & Enterprise Licensing**: For custom commercial licensing, proprietary engine forks, or enterprise support options, contact AethelisDEV.
