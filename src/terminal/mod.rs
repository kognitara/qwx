/// The `terminal` module is responsible for managing the terminal interface of the application. It provides functionality for rendering text, handling user input, and managing the overall terminal state. The module is designed to be efficient and responsive, ensuring a smooth user experience when interacting with the terminal-based application.
pub mod component;
/// The `core` module contains the core functionality of the terminal interface, including event handling, terminal state management, and other essential operations required for the terminal to function correctly.
pub mod core;
/// The `echo` module provides functionality for managing terminal echoing, allowing the application to control how user input is displayed in the terminal. It includes methods for rendering text and shapes with specific styles and attributes.
pub mod echo;
/// The `style` module defines the styling options available for terminal rendering, including colors, attributes, and border styles. It provides a structured way to apply consistent styling across different components of the terminal interface.
pub mod style;
