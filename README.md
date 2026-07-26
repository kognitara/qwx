# Qwx

**[WIP] - Conceptual Terminal Multiplexer & Monolithic Editor**

*Notice: This package is currently a placeholder to reserve the name on `crates.io`. The software is under heavy active development and the first release is not yet functional.*

## About The Project

`qwx` is an ambitious, ultra-versatile terminal environment built in Rust. Designed to completely eliminate context-switching, it merges a text editor, a terminal multiplexer, and a workspace dashboard into a single, unified interface, driven entirely by the keyboard.

At its core, `qwx` relies on a highly structured **10-layer Base-12 hierarchical architecture** (scaling from global network Sessions down to individual memory Nodes), all rendered asynchronously to ensure zero latency.

## Current Status

We are currently in the initial implementation phase. The theoretical foundation is complete, and we are laying down the low-level `crossterm` rendering engine and the core matrix state management.

- [x] Architectural design and strict matrix hierarchy finalized.
- [ ] Core data structures (Nodes, Layers, Views, Panes) implementation.
- [ ] Asynchronous event loop and rendering engine.
- [ ] Base keyboard mapping via Leader key (`Alt`).

Stay tuned. The terminal is about to get a lot bigger.
