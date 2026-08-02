#[derive(Clone, PartialEq, Eq)]
/// Represents the layout of the Finder component in the application.
pub enum FinderLayout {
    ///
    /// # Grid Layout
    ///     
    /// This layout organizes the Finder component into a grid structure, dividing the interface into distinct sections for root directories, sub-root directories, root files, and sub-root files. Each section is clearly delineated, allowing users to easily navigate and manage their files and directories.
    ///
    ///
    /// ```text
    /// ┌──────────────────────────────────────────────────┐
    /// │                   RESEARCH                       │
    /// └──────────────────────────────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRECTORIES      │ SUB ROOTS DIRECTORIES    │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ ROOTS FILES           │ SUB ROOTS FILES          │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// └───────────────────────┴──────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRS FOUNDED     │ SUB ROOTS DIRS FOUNDED   │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ SUB ROOT DIRS FOUNDED │ SUB ROOT FILES FOUNDED   │
    /// └───────────────────────┴──────────────────────────┘
    /// ```
    Grid,
    ///
    ///
    /// # Side-by-Side Layout
    ///
    /// This layout presents the Finder component in a side-by-side format, with directories displayed on the left and files on the right. This arrangement allows users to view and interact with both directories and files simultaneously, enhancing navigation and file management efficiency.  
    ///
    /// ```text
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS                  │ FILES                   │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// └───────────────────────┴─────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS FOUNDED          │ FILES FOUNDED           │
    /// └───────────────────────┴─────────────────────────┘
    /// ```
    ///
    SideBySide,

    ///
    /// # Miller Layout
    ///
    /// This layout divides the Finder component into multiple sections, including parent directories, active directory, child directories, and files. It provides a comprehensive view of the directory structure, allowing users to easily navigate through different levels of directories and manage their files effectively.
    ///
    ///  ```text
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT DIRS   │ ACTIVE DIR      │ CHILD DIRS    │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// ├───────────────┴─────────────────┴───────────────┤
    /// │                    FILES                        │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT FOUNDED│ DIRS FOUNDED    │ CHILD FOUNDED │
    /// └───────────────┴─────────────────┴───────────────┘
    /// ```
    Miller,
    ///
    /// # Commander Layout
    ///
    /// ```text
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌─────────────────────────────────────────────────┐
    /// │                  DIRECTORIES                    │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │                      │                          │
    /// │ CURRENT DIRECTORY    │ CURRENT FILES            │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// └──────────────────────┴──────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │ DIRS FOUNDED         │ FILES FOUNDED            │
    /// └──────────────────────┴──────────────────────────┘
    ///```
    Commander,
    ///
    /// # Mosaic Layout
    ///
    /// This layout arranges the Finder component in a mosaic pattern, allowing for a more flexible and visually appealing presentation of directories and files. It provides a unique way to view and interact with the file system, enhancing the user experience by offering a dynamic and organized interface.
    ///
    /// ```text
    /// ┌──────────────┬──────┬──────┐
    /// │              │ src/ │ app/ │
    /// │    Root /    ├──────┼──────┤
    /// │              │ doc/ │ lib/ │
    /// └──────────────┴──────┴──────┘
    /// ```
    ///
    Mosaic,
}
