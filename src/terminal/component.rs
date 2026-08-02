use crate::finder::search::{FinderSearchKind, FinderSearchOrder};
use crate::finder::{layout::FinderLayout, search::FilterKind};
use crate::terminal::core::QwxEvent;
use crate::terminal::echo::Echo;

use crossterm::terminal::WindowSize;
use std::io::{Result, Write};

#[doc = "The main application trait, combining Editor and Finder functionality"]
pub trait App: Editor + Finder {
    /// Handle an event in the application.
    ///
    /// This method is called whenever an event occurs in the terminal, such as a key press or a paste action. Implementations of this trait should define how the application responds to these events.
    fn on_event(&mut self, event: &QwxEvent);

    /// Render the application state to the terminal.
    ///
    /// This method is responsible for drawing the current state of the application to the terminal. It takes a mutable reference to a writer (which could be stdout or any other type implementing the `Write` trait), an `Echo` instance for handling terminal echoing, and the current window size.
    /// Implementations of this trait should define how the application is visually represented in the terminal.
    ///
    /// # Arguments
    /// * `w` - A mutable reference to a type implementing the `Write` trait (e.g., `stdout` or a `Vec<u8>`).
    /// * `echo` - An instance of the `Echo` struct, used for handling terminal echoing.
    /// * `window` - The current size of the terminal window, provided as a `WindowSize` struct.
    /// # Returns
    /// * `Result<()>` - Returns an `Ok(())` if rendering is successful, or an `Err` if an error occurs during the rendering process.
    fn render<W: Write>(&self, w: &mut W, echo: &Echo, window: &WindowSize) -> Result<()>;
}

#[doc = "The Finder trait, representing the file-finding functionality of the application"]
pub trait Finder {
    /// Default values for the Finder component.
    const DEFAULT_SEARCH_KIND: FinderSearchKind = FinderSearchKind::Name;
    /// Default values for the Finder component.
    const DEFAULT_SEARCH_ORDER: FinderSearchOrder = FinderSearchOrder::Ascending;
    /// Default values for the Finder component.
    const DEFAULT_FILTER_KIND: FilterKind = FilterKind::Equal;

    /// The default layout for the Finder component.
    const DEFAULT_LAYOUT: FinderLayout = FinderLayout::Grid;
    /// The default search path for the Finder component.
    const DEFAULT_RESULTS: &'static [&'static str] = &[];
    /// A list of all available layouts for the Finder component.
    const ALL_LAYOUTS: &'static [FinderLayout] = &[
        FinderLayout::Grid,
        FinderLayout::Commander,
        FinderLayout::Miller,
        FinderLayout::Mosaic,
        FinderLayout::SideBySide,
    ];
    /// A list of all available search kinds for the Finder component.
    const ALL_SEARCH_KINDS: &'static [FinderSearchKind] = &[
        FinderSearchKind::Name,
        FinderSearchKind::Extension,
        FinderSearchKind::Size,
        FinderSearchKind::Date,
        FinderSearchKind::Owner,
        FinderSearchKind::Updated,
        FinderSearchKind::Readable,
        FinderSearchKind::Writable,
        FinderSearchKind::Executable,
    ];
    /// A list of all available filter kinds for the Finder component.
    const ALL_FILTER_KINDS: &'static [FilterKind] = &[
        FilterKind::Equal,
        FilterKind::NotEqual,
        FilterKind::Directory,
        FilterKind::File,
        FilterKind::Include,
        FilterKind::Exclude,
        FilterKind::Contains,
        FilterKind::NotContains,
        FilterKind::LessThan,
        FilterKind::GreaterThan,
        FilterKind::EqualTo,
    ];

    /// A list of all available search orders for the Finder component.
    const ALL_SEARCH_ORDERS: &'static [FinderSearchOrder] =
        &[FinderSearchOrder::Ascending, FinderSearchOrder::Descending];

    /// Get the current search kind for the Finder component.
    ///
    /// This method returns the current search kind, which determines how the Finder component searches for files and directories. Implementations of this trait should define how the search kind is managed and retrieved.
    ///
    fn finder_search_kind(&self) -> FinderSearchKind;
    
    /// Get the current search order for the Finder component.
    ///
    /// This method returns the current search order, which determines the order in which search results are displayed. Implementations of this trait should define how the search order is managed and retrieved.
    ///
    fn finder_search_order(&self) -> FinderSearchOrder;

    /// Get the current filter kind for the Finder component.
    ///
    /// This method returns the current filter kind, which determines how the Finder component filters search results. Implementations of this trait should define how the filter kind is managed and retrieved.
    ///
    fn finder_filter_kind(&self) -> FilterKind;
    /// Set the current search kind for the Finder component.
    ///
    /// This method allows the application to change the search kind of the Finder component. Implementations of this trait should define how the search kind is updated and how it affects the search behavior of the Finder component.
    ///
    /// # Arguments
    /// * `kind` - A `FinderSearchKind` enum value representing the desired search kind for the Finder component. This could be one of several predefined search kinds that determine how the Finder component searches for files and directories.
    ///
    fn set_finder_search_kind(&mut self, kind: FinderSearchKind);
    /// Set the current search order for the Finder component.
    ///
    /// This method allows the application to change the search order of the Finder component. Implementations of this trait should define how the search order is updated and how it affects the display of search results.
    ///
    /// # Arguments
    /// * `order` - A `FinderSearchOrder` enum value representing the desired search order for the Finder component. This could be one of several predefined search orders that determine how search results are organized and displayed in the terminal.
    ///
    fn set_finder_search_order(&mut self, order: FinderSearchOrder);

    /// Set the current filter kind for the Finder component.
    ///
    /// This method allows the application to change the filter kind of the Finder component. Implementations of this trait should define how the filter kind is updated and how it affects the filtering of search results.
    ///
    /// # Arguments
    /// * `kind` - A `FilterKind` enum value representing the desired filter kind for the Finder component. This could be one of several predefined filter kinds that determine how search results are filtered and displayed in the terminal.
    ///
    fn set_finder_filter_kind(&mut self, kind: FilterKind);

    /// Capture input for the Finder component.
    ///
    /// This method is called to handle user input for the Finder component. Implementations of this trait should define how the application processes input characters and how it affects the state of the Finder component.
    ///
    fn finder_capture(&mut self, input: char);

    /// Search for files or directories based on the provided query.
    ///
    /// This method is called to perform a search operation within the application. Implementations of this trait should define how the application searches for files or directories based on the given query string.
    ///
    /// # Arguments
    /// * `query` - A string slice representing the search query. This could be a filename, directory name, or any other search term relevant to the application's context.
    ///   
    fn find(&mut self, query: &str);

    /// Retrieve the results of the last search operation.
    ///
    /// This method returns a vector of strings representing the results of the most recent search operation performed by the `search` method. Implementations of this trait should define how the search results are stored and retrieved.
    ///
    fn finder_results(&self) -> Vec<String>;

    /// Get the current layout of the Finder component.
    ///
    /// This method returns the current layout of the Finder component, which determines how the search results and other related information are displayed in the terminal. Implementations of this trait should define how the layout is managed and retrieved.
    ///
    fn finder_layout(&self) -> FinderLayout;

    /// Set the layout of the Finder component.
    ///
    /// This method allows the application to change the layout of the Finder component. Implementations of this trait should define how the layout is updated and how it affects the display of search results and related information in the terminal.
    ///
    /// # Arguments
    /// * `layout` - A `FinderLayout` enum value representing the desired layout for the Finder component. This could be one of several predefined layouts that determine how information is organized and displayed in the terminal.
    ///
    fn set_finder_layout(&mut self, layout: FinderLayout);
}

#[doc = "The Editor trait, representing the text-editing functionality of the application"]
pub trait Editor {}
