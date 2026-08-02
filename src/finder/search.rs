pub const DEFAULT_FINDER_SEARCH_KIND: FinderSearchKind = FinderSearchKind::Name;
pub const DEFAULT_FINDER_SEARCH_ORDER: FinderSearchOrder = FinderSearchOrder::Ascending;
pub const DEFAULT_FILTER_FOR_EQUALITY: char = '=';
pub const DEFAULT_FILTER_FOR_INEQUALITY: char = '!';
pub const DEFAULT_FILTER_FOR_GREATER_THAN: char = '>';
pub const DEFAULT_FILTER_FOR_LESS_THAN: char = '<';
pub const DEFAULT_FILTER_FOR_GREATER_THAN_OR_EQUAL: char = '≥';
pub const DEFAULT_FILTER_FOR_LESS_THAN_OR_EQUAL: char = '≤';
pub const DEFAULT_FILTER_FOR_CONTAINS: char = '~';
pub const DEFAULT_FILTER_FOR_NOT_CONTAINS: char = '-';
pub const DEFAULT_FILTER_FOR_STARTS_WITH: char = '^';
pub const DEFAULT_FILTER_FOR_ENDS_WITH: char = '$';
pub const DEFAULT_FILTER_FOR_REGEX: char = '/';
pub const DEFAULT_FILTER_FOR_WILDCARD: char = '*';
pub const DEFAULT_FILTER_FOR_EXECUTABLE: char = 'x';
pub const DEFAULT_FILTER_FOR_DIRECTORY: char = 'd';
pub const DEFAULT_FILTER_FOR_FILE: char = 'f';
pub const DEFAULT_FILTER_FOR_SYMLINK: char = 'l';
pub const DEFAULT_FILTER_FOR_HIDDEN: char = 'h';
pub const DEFAULT_FILTER_FOR_READABLE: char = 'r';
pub const DEFAULT_FILTER_FOR_WRITABLE: char = 'w';
pub const DEFAULT_FILTER_FOR_EXTENSION: char = '.';
pub const DEFAULT_FILTER_FOR_NAME: char = 'n';
pub const DEFAULT_FILTER_FOR_SIZE: char = 's';
pub const DEFAULT_FILTER_FOR_DATE: char = 't';
pub const DEFAULT_FILTER_FOR_OWNER: char = 'o';
pub const DEFAULT_FILTER_FOR_UPDATED: char = 'u';
pub const DEFAULT_FILTER_FOR_PERMISSIONS: char = 'p';

// Default values for the FinderSearch struct
pub const DEFAULT_FINDER_SEARCH_PATH: &str = ".";
#[derive(Clone)]

/// A struct representing a search operation in the Finder component.
pub enum FinderSearchKind {
    Name,
    Extension,
    Size,
    Date,
    Owner,
    Updated,
    Readable,
    Writable,
    Executable,
}

/// A struct representing a search operation in the Finder component.
#[derive(Clone)]
pub enum FilterKind {
    Equal,
    NotEqual,
    Directory,
    File,
    Include,
    Exclude,
    Contains,
    NotContains,
    LessThan,
    GreaterThan,
    EqualTo,
    Extension(&'static str),
}
#[derive(Clone)]

/// A struct representing a search operation in the Finder component.
pub enum FinderSearchOrder {
    Ascending,
    Descending,
}
#[derive(Clone)]

/// A struct representing the result of a search operation in the Finder component.
pub enum FinderSearchResult {
    Found(
        Vec<String>, // List of found items
    ),
    NotFound,
}
/// A struct representing a search operation in the Finder component.
#[derive(Clone)]
pub struct FinderSearch {
    pub path: String,
    pub filters: Vec<FilterKind>,
    pub c: char,
    pub query: String,
    pub kind: FinderSearchKind,
    pub order: FinderSearchOrder,
}

impl Default for FinderSearch {
    fn default() -> Self {
        Self {
            path: DEFAULT_FINDER_SEARCH_PATH.to_string(),
            filters: Vec::new(),
            c: DEFAULT_FILTER_FOR_EQUALITY,
            query: String::new(),
            kind: DEFAULT_FINDER_SEARCH_KIND,
            order: DEFAULT_FINDER_SEARCH_ORDER,
        }
    }
}
