# Quaternary Walk eXtended

`qwx` is an ultra-fast, modal terminal application designed for developers, system administrators,
and teams looking to optimize their workflow without ever leaving the command line.

Developed to remove distractions, `qwx` renders heavy graphical IDEs unnecessary by merging code
editing, server management, web research, security auditing, and collaboration into a single, cohesive,
and lightning fast text-based interface.

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
a database client, a task manager, a web browser, and multiple terminal windows disrupts concentration.

Graphical interfaces impose their own logic, consume unnecessary resources, and force mouse usage,
breaking the flow of thought.

Inspired by the mathematical rigor of tiling window managers and the efficiency of modal editors,
`qwx` was created to offer a radical solution: a universal, generic system.

It was built because there was no environment capable of treating data not merely as a file
trapped in a window but as a particle of pure information.

By using a Tesseract-based architecture, `qwx` allows you to view a single source of truth
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
| **Workspace**   | A 3D volume containing a logical group of data.           |
| **View**        | Viewing angle of a specific Workspace.                    |
| **Node**        | In-memory quantum state of an architecture element.       |
| **Layer**       | Document modification state at a specific timeline point. |
| **Grid**        | Geometric adjacency relationships between each Panel.     |
| **Menu**        | Contextual Panel interface for rapid actions.             |

## Modes

| NAME           | DEFINITION                                                                                              |
| -------------- | ------------------------------------------------------------------------------------------------------- |
| **Normal**     | The default navigation and layout management state for rapid spatial operations.                        |
| **Editor**     | Direct text insertion and modification mode.                                                            |
| **Finder**     | Fast project directory and file navigation overlay.                                                     |
| **Menu**       | Contextual command execution and file system quick actions (`!mkdir`, `!touch`, etc.).                  |
| **Search**     | In-buffer text pattern search and match navigation.                                                     |
| **WebSearch**  | Unified Search Hub, DevSecOps intelligence, package lookup, Git workflow, and CVE vulnerability audit.  |
| **Player**     | Integrated Spotify music player & TUI controller (search, queue, playlists, playback, devices).         |
| **Zen**        | Isolates and centers the active panel while hiding the grid. Other panels run in the background.        |
| **Rescue**     | Emergency mode. Freezes complex rendering and background parsing to save connection under high latency. |
| **Broadcast**  | Transforms terminal into a native collaborative pair-programming space with real-time cursor sync.      |
| **Ephemeral**  | Secure, floating scratchpad layer. Data is never stored and gets instantly destroyed upon exit.         |

## SEARCH HUB & DEVSECOPS INTELLIGENCE

The newly introduced **Search Hub** (`WebSearch` mode) directly bridges external developer intelligence, security audits, and Git workflows into your terminal session without any context switching.

### Supported Search Providers

- **1. All** (`Alt + 1`): General web search and instant technical answers powered by DuckDuckGo.
- **2. GitHub** (`Alt + 2`): Search public repositories, inspect star counts, forks, open issues, and repository descriptions.
- **3. GitLab** (`Alt + 3`): Search projects and repositories on GitLab.
- **4. Wikipedia** (`Alt + 4`): Technical encyclopedic lookup with instant in-terminal article previews.
- **5. CVE / Security** (`Alt + 5`): Live vulnerability queries against global databases (OSV.dev / NVD).
- **6. Hacker News** (`Alt + 6`): Tech news, engineering discussions, and community discussions powered by Algolia.
- **7. Local Audit** (`Alt + 7` or `Alt + a`): Automated zero-setup dependency audit scanning local project files (`Cargo.lock`, `Cargo.toml`) against the OSV.dev vulnerability database.

### Integrated Git & GitHub Workflow

From within the Search Hub, you can execute Git actions directly on search results:
- **Clone Repository** (`Alt + c` / `Ctrl + c`): Interactively clone a selected repository to your local workspace.
- **Create Branch** (`Alt + b` / `Ctrl + b`): Prompt to create a new branch in the current project repository.
- **Checkout Branch** (`Alt + s`): Switch to an existing Git branch.
- **Create Pull Request** (`Alt + p`): Interactive step-by-step wizard to create and publish a GitHub Pull Request (repository, title, description, head branch, base branch, auth token).
- **Export Markdown Report** (`Alt + e` / `Ctrl + e`): Generate and export a Markdown audit/search report (e.g. `cve-security-report.md` or `search-report.md`).
- **Open in Browser** (`Alt + o` / `Ctrl + o`): Open the selected result URL in your default system web browser.

## SPOTIFY MUSIC PLAYER TUI

`qwx` includes a fully embedded, real-time terminal music player for Spotify (`Player` mode via `Alt + p`, `Alt + m`, or `:player`).

### Key Capabilities

- **Now Playing Display**: Real-time progress bar, track title, artists, album, device badge, volume, repeat and shuffle indicators.
- **Search & Queue Integration**: Instant search across tracks, albums, and playlists with category cycling (`c`), direct playback (`Enter`), or queuing tracks (`a`).
- **Device Management**: View and switch active Spotify Connect devices seamlessly.
- **Playlists & Saved Songs**: Browse user playlists and liked songs directly in terminal.
- **Authentication**: Zero-friction setup via environment variables (`SPOTIFY_TOKEN` / `SPOTIFY_ACCESS_TOKEN`, `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`), persistent config (`~/.config/qwx/spotify.json`), or interactive in-TUI configuration.

## KEYBOARD SHORTCUTS

### Editor Mode Shortcuts (Insert & Edit)

The Editor Mode is dedicated to direct text insertion and modification.

| Shortcut    | Action               | Description                                                                                              |
|:------------|:---------------------|:---------------------------------------------------------------------------------------------------------|
| `Esc`       | **Exit / Clear**     | Clears the current text selection. If no selection exists, exits Editor mode and returns to Normal mode. |
| `Ctrl + s`  | **Save**             | Writes the current file modifications to the disk.                                                       |
| `Ctrl + z`  | **Undo**             | Reverts the latest text change.                                                                          |
| `Ctrl + y`  | **Redo**             | Restores the previously reverted text change.                                                            |
| `Ctrl + v`  | **Paste**            | Inserts the clipboard contents at the cursor position.                                                   |
| `Ctrl + k`  | **Delete to End**    | Deletes all characters from the current cursor position to the end of the line.                          |
| `Alt + x`   | **Select Line**      | Selects the entire line currently under the cursor.                                                      |
| `Alt + d`   | **Delete Selection** | Deletes the currently selected text block.                                                               |
| `Tab`       | **Indent**           | Inserts 4 spaces for strict and consistent indentation.                                                  |
| `Enter`     | **New Line**         | Inserts a line break and moves the cursor to the next line.                                              |
| `Backspace` | **Delete Left**      | Removes the character immediately preceding the cursor.                                                  |
| `Delete`    | **Delete Right**     | Removes the character immediately under the cursor.                                                      |

### Normal Mode Shortcuts (Navigation & Layout)

The Normal Mode is the default state of qwx, used for lightning-fast spatial navigation, panel management, and quick text manipulations without entering insert mode.

| Shortcut                        | Action                 | Description                                                                       |
|:--------------------------------|:-----------------------|:----------------------------------------------------------------------------------|
| `e`                             | **Enter Editor**       | Switches to Editor mode at the current cursor position.                           |
| `o`                             | **Append New Line**    | Jumps to the end of the current line, inserts a new line, and enters Editor mode. |
| `h`, `j`, `k`, `l`              | **Move Cursor**        | Moves the cursor Left (`h`), Down (`j`), Up (`k`), or Right (`l`).                |
| `PageUp` / `PageDn`             | **Fast Scroll**        | Jumps 15 lines up or down for rapid vertical navigation through the file.         |
| `u`                             | **Undo**               | Reverts the last edit operation.                                                  |
| `U` / `Ctrl + y`                | **Redo**               | Restores the undone edit operation.                                               |
| `y`                             | **Yank (Copy)**        | Copies the current line or selection to the clipboard.                            |
| `p`                             | **Paste**              | Pastes the clipboard contents at the current cursor position.                     |
| `n`                             | **Next Match**         | Jumps to the next search match.                                                   |
| `N`                             | **Prev Match**         | Jumps to the previous search match.                                               |
| `Ctrl + s`                      | **Save**               | Writes changes to disk.                                                           |
| `x`                             | **Select Line**        | Selects the current line for quick manipulation.                                  |
| `d`                             | **Delete Selection**   | Instantly deletes the currently highlighted selection.                            |
| `Esc`                           | **Clear Selection**    | Drops the current text selection.                                                 |
| `Ctrl + h, j, k, l`             | **Shift Focus**        | Shifts the active workspace focus to the Left, Bottom, Top, or Right panel.       |
| `Ctrl + r`                      | **Rotate Clockwise**   | Rotates the physical views of the panels in a clockwise direction.                |
| `Alt + r`                       | **Rotate Counter**     | Rotates the physical views of the panels in a counter-clockwise direction.        |
| `Alt + f`                       | **Finder Mode**        | Opens the file finder overlay to navigate the project directory.                  |
| `Alt + d`                       | **Dmenu / Menu Mode**  | Opens the command menu (dmenu) for rapid execution.                               |
| `Alt + /`                       | **Buffer Search Mode** | Opens the in-buffer search prompt to find patterns within the active node.        |
| `s`, `Alt + s, w`               | **Search Hub Mode**    | Opens the global Search Hub & DevSecOps security audit suite.                     |
| `Alt + p`, `Alt + m`, `:player` | **Player Mode**        | Opens the integrated Spotify Music Player TUI.                                    |
| `q`                             | **Quit**               | Terminates the qwx environment.                                                   |

### Search Hub & DevSecOps Mode Shortcuts

When inside the Search Hub (`WebSearch` mode):

| Shortcut                    | Action                  | Description                                                                                            |
|:----------------------------|:------------------------|:-------------------------------------------------------------------------------------------------------|
| `Enter`                     | **Execute / Submit**    | Runs search query or confirms the current interactive prompt step.                                     |
| `Esc`                       | **Close / Cancel**      | Cancels active prompt or exits Search Hub back to Normal mode.                                         |
| `Tab` / `Shift + Tab`       | **Cycle Provider**      | Cycles forward or backward through search providers.                                                   |
| `Alt + 1` .. `Alt + 7`      | **Select Provider**     | Directly switches to a provider (1: All, 2: GitHub, 3: GitLab, 4: Wikipedia, 5: CVE, 6: HN, 7: Audit). |
| `Up` / `Down`, `Ctrl+p / n` | **Select Result**       | Moves the selection cursor through the search results list.                                            |
| `PageUp` / `PageDn`         | **Scroll Preview**      | Scrolls up or down inside the result preview pane.                                                     |
| `Alt + a` / `Ctrl + a`      | **Run Local Audit**     | Immediately triggers a local dependency vulnerability CVE audit.                                       |
| `Alt + c` / `Ctrl + c`      | **Clone Repository**    | Opens interactive prompt to clone selected repository into local directory.                            |
| `Alt + b` / `Ctrl + b`      | **Create Branch**       | Opens interactive prompt to create a new Git branch.                                                   |
| `Alt + s`                   | **Checkout Branch**     | Opens interactive prompt to switch / checkout a Git branch.                                            |
| `Alt + p`                   | **Create Pull Request** | Launches interactive 6-step wizard to create a GitHub Pull Request.                                    |
| `Alt + e` / `Ctrl + e`      | **Export Report**       | Exports results or security audit findings as a Markdown report file.                                  |
| `Alt + o` / `Ctrl + o`      | **Open in Browser**     | Opens the URL of the selected item in the default web browser.                                         |

### Spotify Music Player Mode Shortcuts

When inside the Music Player (`Player` mode):

| Shortcut                 | Action                | Description                                                                                       |
|:-------------------------|:----------------------|:--------------------------------------------------------------------------------------------------|
| `Space`                  | **Play / Pause**      | Toggles audio playback state.                                                                     |
| `n` / `>`                | **Next Track**        | Skips to the next track.                                                                          |
| `p` / `<`                | **Previous Track**    | Returns to the previous track.                                                                    |
| `+` / `-`                | **Volume Up / Down**  | Increases or decreases volume by 5%.                                                              |
| `Left` / `Right`         | **Seek Position**     | Rewinds or fast-forwards track position by 5 seconds (in Now Playing tab).                        |
| `f`                      | **Seek Prompt**       | Opens prompt to seek to a specific position in seconds.                                           |
| `v`                      | **Set Volume**        | Opens prompt to set volume percentage directly (0-100%).                                          |
| `z` / `s`                | **Toggle Shuffle**    | Toggles playback shuffle mode ON / OFF.                                                           |
| `r`                      | **Cycle Repeat**      | Cycles repeat mode (Off -> Context -> Track).                                                     |
| `Shift + r` / `F5`       | **Refresh State**     | Refreshes current playback state, devices, and playlists from Spotify.                            |
| `Tab` / `Shift + Tab`    | **Cycle Tabs**        | Navigates across tabs (Now Playing, Search, Queue, Playlists, Devices, Config).                   |
| `1` .. `6`               | **Direct Tab Select** | Jumps directly to tab (1: Now Playing, 2: Search, 3: Queue, 4: Playlists, 5: Devices, 6: Config). |
| `j` / `k`, `Down` / `Up` | **Navigate List**     | Moves cursor up or down in current list.                                                          |
| `Enter`                  | **Play / Select**     | Plays selected track/album/playlist or activates selected item/action.                            |
| `/`                      | **Search Prompt**     | Opens interactive search prompt in Search tab.                                                    |
| `c`                      | **Cycle Category**    | In Search tab, switches search category (Tracks, Albums, Playlists).                              |
| `a`                      | **Add to Queue**      | Adds the selected track from search results into the player queue.                                |
| `d` / `Delete`           | **Remove from Queue** | Removes the selected track from the queue tab.                                                    |
| `t`                      | **Set Token**         | Opens prompt to update Spotify Access Token.                                                      |
| `Esc` / `q`              | **Exit Player**       | Closes the player and returns to Normal mode.                                                     |
