//! Utilities for working with source code and printing nicely formatted
//! diagnostic information like warnings and errors.
//!
//! # Optional Features
//!
//! Extra functionality is accessible by enabling feature flags. The features
//! currently available are:
//!
//! - **serialization** - Adds `Serialize` and `Deserialize` implementations
//!   for use with `serde`
//! - **memory_usage** - Adds `HeapSizeOf` implementations for use with the
//!   `heapsize` crate

#[macro_use]
extern crate failure;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(feature = "serialization")]
extern crate serde;
#[cfg(feature = "serialization")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "memory_usage")]
extern crate heapsize;
#[cfg(feature = "memory_usage")]
#[macro_use]
extern crate heapsize_derive;

mod codemap;
mod filemap;
mod index;
mod span;

pub use crate::codemap::CodeMap;
pub use crate::filemap::FileMap;
pub use crate::filemap::{ByteIndexError, LineIndexError, LocationError, SpanError};
pub use crate::index::{ByteIndex, ByteOffset};
pub use crate::index::{ColumnIndex, ColumnNumber, ColumnOffset};
pub use crate::index::{Index, Offset};
pub use crate::index::{LineIndex, LineNumber, LineOffset};
pub use crate::index::{RawIndex, RawOffset};
pub use crate::span::{ByteSpan, Span};
