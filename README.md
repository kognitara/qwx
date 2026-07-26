# Qwx

**A monolithic, modal terminal environment built in Rust.**

> **Status: Work In Progress (WIP)**
> Qwx is currently in active early development. It is being built as a central hub for coding, system manipulation, and text editing without breaking context.

## Vision & Architecture
Qwx is designed to eliminate context switching by bringing the entire workflow into a single, cohesive terminal environment. 

* **Modal Control:** Fast, keyboard-driven navigation centered around the `Alt` key as the Leader.
* **Tiling Layouts:** Dynamic pane management inspired by tiling window managers (like xmonad).
* **Deep Hierarchy:** Structured across 10 distinct layers (Session, Bank, Environment, Face, Pane, Menu, Workspace, View, Layer, Node).
* **Base 12:** Core layout and grid logic structured around a base-12 architecture.

## Current Development (Bank 1)
The active development sprint is focused on the core rendering engine and document editing features:
- [x] Basic data structures (`Node`, `Layer`, `View`) implemented.
- [x] `crossterm` rendering engine online.
- [x] 2x2 grid rendering with quadrant isolation.
- [x] Workspace and pane navigation shortcuts.

## Next Steps
- Transition to `Mode::Insert` for actual text input inside panes.
- Dynamic layout cycling (`Alt + Space`).
- YAML-based state persistence.
- Network engine integration via `ssh2`.

## Installation
Currently reserved on `crates.io`. Source code build instructions will be provided once the MVP is stable.
