//! Utilities for working with source code and printing nicely formatted
//! diagnostic information like warnings and errors.

#[macro_use]
extern crate failure;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(feature = "serde_derive")]
extern crate serde;
#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

mod codemap;
mod filemap;
mod index;
mod span;

pub use codemap::CodeMap;
pub use filemap::{FileMap, FileName};
pub use index::{Index, Offset};
pub use index::{ByteIndex, ByteOffset};
pub use index::{ColumnIndex, ColumnNumber, ColumnOffset};
pub use index::{LineIndex, LineNumber, LineOffset};
pub use index::{RawIndex, RawOffset};
pub use span::{ByteSpan, Span};
