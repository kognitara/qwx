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

```txt
                        
                        Default mode

+-------------------------------------------------------+
|                           f                           |
|    +-------------------+     +-------------------+    |
|    | a                 |     | b                 |    |
|    |                   |     |                   |    |
|    |       ~~~~~       |     |       ~~~~~       |    |
|    |       ~~~~~       |     |       ~~~~~       |    |
|    |                   |     |                   |    |
|    |              % 1¹ |     |              % 1¹ |    | f The current visible Face.
|    +-------------------+     +-------------------+    | p A Panel
|                                                       | 1 The Current Workspace.
|    +-------------------+     +-------------------+    | ¹ The current View.
|    | c                 |     | d                 |    | % The state of the active Node.
|    |                   |     |                   |    | ~ The Panel data.
|    |       ~~~~~       |     |       ~~~~~       |    |
|    |       ~~~~~       |     |       ~~~~~       |    |
|    |                   |     |                   |    |
|    |              % 1¹ |     |              % 1¹ |    |
|    +-------------------+     +-------------------+    |
|                                                       |
+-------------------------------------------------------+

                          Zen mode

                        a The current visible Panel
                        h A hidden Panel 
                        f The Face
                        ¹ The current View
                        1 The current Workspace

+-----------------------------------------------------------------------+
|                                                                       |                                                      
|                                 f                                     |
|                                                                       |          
|        +-------------------------------------------------------+      |     +-----------------------------------------------+
|        | a                                                     |      |     | h                                             |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     +-----------------------------------------------+  
|        |                                                       |      |     +-----------------------------------------------+
|        |              ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~           |      |     | h                                             |
|        |              ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~           |      |     |                                               |
|        |              ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~           |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     +-----------------------------------------------+
|        |                                                       |      |     +-----------------------------------------------+
|        |                                                       |      |     | h                                             |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                       |      |     |                                               |
|        |                                                  % 1¹ |      |     |                                               |
|        +-------------------------------------------------------+      |     +-----------------------------------------------+
|                                                                       |                                                      
|                                                                       |    
|                                                                       |                                                       
+-----------------------------------------------------------------------+                                                      
