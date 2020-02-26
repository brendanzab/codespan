//! Source file support for diagnostic reporting.

use std::ops::Range;

/// A location in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Location {
    /// The line number in the source file.
    pub line_number: usize,
    /// The column number in the source file.
    pub column_number: usize,
}

/// A line within a source file.
pub struct Line<Source> {
    /// The starting byte index of the line.
    pub start: usize,
    /// The line number.
    pub number: usize,
    /// The source of the line.
    pub source: Source,
}

impl<Source> Line<Source>
where
    Source: AsRef<str>,
{
    /// The column index at the given byte index in the source file.
    /// This is the number of characters to the given byte index.
    ///
    /// If the byte index is smaller than the start of the line, then `0` is returned.
    /// If the byte index is past the end of the line, the column index of the last
    /// character `+ 1` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codespan_reporting::files::Line;
    ///
    /// let line = Line {
    ///     start: 2,
    ///     number: 2,
    ///     source: "🗻∈🌏",
    /// };
    ///
    /// assert_eq!(line.column_index(0), 0);
    /// assert_eq!(line.column_index(line.start + 0), 0);
    /// assert_eq!(line.column_index(line.start + 1), 0);
    /// assert_eq!(line.column_index(line.start + 4), 1);
    /// assert_eq!(line.column_index(line.start + 8), 2);
    /// assert_eq!(line.column_index(line.start + line.source.len()), 3);
    /// ```
    pub fn column_index(&self, byte_index: usize) -> usize {
        match byte_index.checked_sub(self.start) {
            None => 0,
            Some(relative_index) => {
                let line_source = self.source.as_ref();
                let column_index = line_source
                    .char_indices()
                    .map(|(i, _)| i)
                    .take_while(|i| *i < relative_index)
                    .count();

                if relative_index >= line_source.len()
                    || line_source.is_char_boundary(relative_index)
                {
                    column_index
                } else {
                    column_index - 1
                }
            },
        }
    }

    /// The 1-indexed column number at the given byte index.
    pub fn column_number(&self, byte_index: usize) -> usize {
        self.column_index(byte_index) + 1
    }
}

/// Files that can be used for pretty printing.
pub trait Files {
    type FileId: Copy + PartialEq + PartialOrd + Eq + Ord + std::hash::Hash;
    type Origin: std::fmt::Display;
    type LineSource: AsRef<str>;

    /// The origin of a file.
    fn origin(&self, id: Self::FileId) -> Option<Self::Origin>;

    /// The line at the given index.
    fn line(&self, id: Self::FileId, line_index: usize) -> Option<Line<Self::LineSource>>;

    /// The index of the line at the given byte index.
    fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize>;

    /// The location of the given byte index.
    fn location(&self, id: Self::FileId, byte_index: usize) -> Option<Location> {
        let line_index = self.line_index(id, byte_index)?;
        let line = self.line(id, line_index)?;

        Some(Location {
            line_number: line.number,
            column_number: line.column_number(byte_index),
        })
    }
}

/// A single source file.
///
/// This is useful for simple language tests, but it might be worth creating a
/// custom implementation when a language scales beyond a certain size.
#[derive(Debug, Clone)]
pub struct SimpleFile<Origin, Source> {
    /// The origin of the file.
    origin: Origin,
    /// The source code of the file.
    source: Source,
    /// The starting byte indices in the source code.
    line_starts: Vec<usize>,
}

fn line_starts<'a>(source: &'a str) -> impl 'a + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}

impl<Origin, Source> SimpleFile<Origin, Source>
where
    Origin: std::fmt::Display,
    Source: AsRef<str>,
{
    /// Create a new source file.
    pub fn new(origin: Origin, source: Source) -> SimpleFile<Origin, Source> {
        SimpleFile {
            origin,
            line_starts: line_starts(source.as_ref()).collect(),
            source,
        }
    }

    /// Return the origin of the file.
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the source of the file.
    pub fn source(&self) -> &Source {
        &self.source
    }

    fn line_start(&self, line_index: usize) -> Option<usize> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Some(self.line_starts[line_index]),
            Ordering::Equal => Some(self.source.as_ref().len()),
            Ordering::Greater => None,
        }
    }

    fn line_range(&self, line_index: usize) -> Option<Range<usize>> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
    }
}

impl<Origin, Source> Files for SimpleFile<Origin, Source>
where
    Origin: std::fmt::Display + Clone,
    Source: AsRef<str>,
{
    type FileId = ();
    type Origin = Origin;
    type LineSource = String;

    fn origin(&self, (): ()) -> Option<Origin> {
        Some(self.origin.clone())
    }

    fn line_index(&self, (): (), byte_index: usize) -> Option<usize> {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => Some(line),
            Err(next_line) => Some(next_line - 1),
        }
    }

    fn line(&self, (): (), line_index: usize) -> Option<Line<String>> {
        let range = self.line_range(line_index)?;

        Some(Line {
            start: range.start,
            number: line_index + 1,
            source: self.source.as_ref()[range].to_owned(),
        })
    }
}

/// A file database that can store multiple source files.
///
/// This is useful for simple language tests, but it might be worth creating a
/// custom implementation when a language scales beyond a certain size.
#[derive(Debug, Clone)]
pub struct SimpleFiles<Origin, Source> {
    files: Vec<SimpleFile<Origin, Source>>,
}

impl<Origin, Source> SimpleFiles<Origin, Source>
where
    Origin: std::fmt::Display,
    Source: AsRef<str>,
{
    /// Create a new files database.
    pub fn new() -> SimpleFiles<Origin, Source> {
        SimpleFiles { files: Vec::new() }
    }

    /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
    pub fn add(&mut self, origin: Origin, source: Source) -> usize {
        let file_id = self.files.len();
        self.files.push(SimpleFile::new(origin, source));
        file_id
    }

    /// Get the file corresponding to the given id.
    pub fn get(&self, file_id: usize) -> Option<&SimpleFile<Origin, Source>> {
        self.files.get(file_id)
    }
}

impl<Origin, Source> Files for SimpleFiles<Origin, Source>
where
    Origin: std::fmt::Display + Clone,
    Source: AsRef<str>,
{
    type FileId = usize;
    type Origin = Origin;
    type LineSource = String;

    fn origin(&self, file_id: usize) -> Option<Origin> {
        Some(self.get(file_id)?.origin().clone())
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Option<usize> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line(&self, file_id: usize, line_index: usize) -> Option<Line<String>> {
        self.get(file_id)?.line((), line_index)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_SOURCE: &str = "foo\nbar\r\n\nbaz";

    #[test]
    fn line_starts() {
        let file = SimpleFile::new("test", TEST_SOURCE);

        assert_eq!(
            file.line_starts,
            [
                0,  // "foo\n"
                4,  // "bar\r\n"
                9,  // ""
                10, // "baz"
            ],
        );
    }

    #[test]
    fn line_span_sources() {
        let file = SimpleFile::new("test", TEST_SOURCE);

        let line_sources = (0..4)
            .map(|line| {
                let line_range = file.line_range(line).unwrap();
                &file.source[line_range]
            })
            .collect::<Vec<_>>();

        assert_eq!(line_sources, ["foo\n", "bar\r\n", "\n", "baz"]);
    }
}