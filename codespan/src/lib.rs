//! Utilities for working with source code and printing nicely formatted
//! diagnostic information like warnings and errors.

#[macro_use]
extern crate failure;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod codemap;
mod filemap;
mod pos;

pub use self::codemap::CodeMap;
pub use self::filemap::{FileMap, FileName};
pub use self::pos::{ByteIndex, ByteOffset, ByteSpan, ColumnIndex, ColumnNumber, LineIndex,
                    LineNumber, RawIndex, RawOffset};
