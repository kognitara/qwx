# Quaternary Walk eXtended

`qwx` is an ultra-fast, modal terminal application designed for developers, system administrators,
and teams looking to optimize their workflow without ever leaving the command line.

Developed to eliminate distractions, `qwx` renders heavy graphical IDEs unnecessary by merging code
editing, server management, and collaboration into a single, cohesive, and lightning fast
text-based interface.

More than just a text editor, `qwx` is a multidimensional environment engine.

It strictly separates the physical display from the logical data.

This decoupling allows you to display, manipulate, and link any type of data across multiple
perspectives without ever duplicating information in memory.

## PHILOSOPHY

1. Keep your hands on the keyboard's home row. Every action should be natural.
2. No mouse, no pop-ups, no context switching, no friction.
3. The interface must never stand between the developer and their code.
4. Designed for absolute speed, latency is not tolerated.
5. A structured visual grid system for instant navigation between workspaces.
6. A single tool to unify your engineering processes under one standard.
7. Mergeable panels to shape the interface to your needs.
8. The workspace adapts instantly to the task's cognitive load.
9. Maintain a state of deep focus.

## WHY

Modern software development is fragmented. Constantly juggling between a text editor,
a database client, a task manager, and multiple terminal windows disrupts concentration.

Graphical interfaces impose their own logic, consume unnecessary resources, and force mouse usage,
breaking the flow of thought.

Inspired by the mathematical rigor of tiling window managers and the efficiency of modal editors,
`qwx` was created to offer a radical solution a universal, generic system.

It was built because there was no environment capable of treating data not merely as a fill
trapped in a window but as a particle of pure information.

By utilizing a Tesseract-based architecture, `qwx` allows you to view a single source of truth
from infinite angles (Views) and within infinite contexts (Workspaces), all while ensuring
absolute control over the interface via atomic keyboard commands.

`qwx` was not created for just text editing, it was created to manipulate information at
the speed of thought.

## VOCABULARY

| NAME            | DEFINITION                                                |
| --------------- | --------------------------------------------------------- |
| **Bank**        | Supreme logical anchor of the qwx architecture.           |
| **Session**     | Bidirectional projection of a system environment.         |
| **Environment** | Dedicated configuration for a specific workflow.          |
| **Face**        | Global macroscopic layout of the screen interface.        |
| **Facet**       | One of two opposing virtual sides of a Face.              |
| **Panel**       | Physical 2D window anchored to the grid.                  |
| **Workspace**   | A 3D volume containing a logical group of data            |
| **View**        | Viewing angle of a specific Workspace.                    |
| **Node**        | In-memory quantum state of an architecture element.       |
| **Layer**       | Document modification state at a specific timeline point. |
| **Grid**        | Geometric adjacency relationships between each Panel.     |
| **Menu**        | Contextual Panel interface for rapid actions.             |

## Modes

| NAME          | DEFINITION                                                                                              |
| ------------- | ------------------------------------------------------------------------------------------------------- |
| **Zen**       | Isolates and centers the active panel while hiding the grid. Other panels run in the background.        |
| **Rescue**    | Emergency mode. Freezes complex rendering and background parsing to save connection under high latency. |
| **Broadcast** | Transforms terminal into a native collaborative pair-programming space with real-time cursor sync.      |
| **Ephemeral** | Secure, floating scratchpad layer. Data is never stored and gets instantly destroyed upon exit.         |

### Editor Mode Shortcuts (Insert & Edit)

The Editor Mode is dedicated to direct text insertion and modification.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Esc` | **Exit / Clear** | Clears the current text selection. If no selection exists, exits Editor mode and returns to Normal mode. |
| `Ctrl + s` | **Save** | Writes the current file modifications to the disk. |
| `Ctrl + k` | **Delete to End** | Deletes all characters from the current cursor position to the end of the line. |
| `Alt + x` | **Select Line** | Selects the entire line currently under the cursor. |
| `Alt + d` | **Delete Selection** | Deletes the currently selected text block. |
| `Tab` | **Indent** | Inserts 4 spaces for strict and consistent indentation. |
| `Enter` | **New Line** | Inserts a line break and moves the cursor to the next line. |
| `Backspace` | **Delete Left** | Removes the character immediately preceding the cursor. |
| `Delete` | **Delete Right** | Removes the character immediately under the cursor. |

### Normal Mode Shortcuts (Navigation & Layout)

The Normal Mode is the default state of qwx, used for lightning-fast spatial navigation, panel management, and quick text manipulations without entering insert mode.

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `e` | **Enter Editor** | Switches to Editor mode at the current cursor position. |
| `o` | **Append New Line** | Jumps to the end of the current line, inserts a new line, and enters Editor mode. |
| `h`, `j`, `k`, `l` | **Move Cursor** | Moves the cursor Left (`h`), Down (`j`), Up (`k`), or Right (`l`). |
| `PageUp` / `PageDn` | **Fast Scroll** | Jumps 15 lines up or down for rapid vertical navigation through the file. |
| `x` | **Select Line** | Selects the current line for quick manipulation. |
| `d` | **Delete Selection** | Instantly deletes the currently highlighted selection. |
| `Esc` | **Clear Selection** | Drops the current text selection. |
| `Ctrl + h, j, k, l` | **Shift Focus** | Shifts the active workspace focus to the Left, Bottom, Top, or Right panel. |
| `Ctrl + r` | **Rotate Clockwise** | Rotates the physical views of the panels in a clockwise direction. |
| `Alt + r` | **Rotate Counter** | Rotates the physical views of the panels in a counter-clockwise direction. |
| `Alt + f` | **Finder Mode** | Opens the file finder overlay to navigate the project directory. |
| `Alt + d` | **Dmenu Mode** | Opens the command menu (dmenu) for rapid execution. |
| `Alt + /` | **Search Mode** | Opens the search buffer to find patterns within the active node. |
| `q` | **Quit** | Terminates the qwx environment. |
