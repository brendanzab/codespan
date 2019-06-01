//! Data structures for tracking source positions in the MLTT language.

#![warn(rust_2018_idioms)]

mod file;
mod index;
mod location;
mod span;

pub use crate::file::*;
pub use crate::index::*;
pub use crate::location::*;
pub use crate::span::*;
