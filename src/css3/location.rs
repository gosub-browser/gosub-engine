use core::fmt::{Debug, Formatter};

/// Location holds the start position of the given element in the data source
#[derive(Clone, PartialEq)]
pub struct Location {
    /// Line number, starting with 1
    line: u32,
    /// Column number, starting with 1
    column: u32,
    /// Byte offset, starting with 0
    offset: u32,
}

impl Location {
    pub(crate) fn inc_line(&mut self) {
        self.line += 1;
    }
    pub(crate) fn inc_column(&mut self) {
        self.column += 1;
    }
    pub(crate) fn set_column(&mut self, col: u32) {
        self.column = col;
    }
    pub(crate) fn inc_offset(&mut self) {
        self.offset += 1;
    }
}

impl Default for Location {
    /// Default to line 1, column 1
    fn default() -> Self {
        Self::new(1, 1, 0)
    }
}

impl Location {
    /// Create a new Location
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Get the line number
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Get the column number
    pub fn column(&self) -> u32 {
        self.column
    }

    /// Get the offset
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.column)
    }
}
