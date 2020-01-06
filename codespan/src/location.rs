#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{ColumnIndex, LineIndex};

/// A location in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Location {
    /// The line index in the source file.
    pub line: LineIndex,
    /// The column index in the source file.
    pub column: ColumnIndex,
}

impl Location {
    /// Construct a new location from a line index and a column index.
    pub fn new(line: impl Into<LineIndex>, column: impl Into<ColumnIndex>) -> Location {
        Location {
            line: line.into(),
            column: column.into(),
        }
    }
}
