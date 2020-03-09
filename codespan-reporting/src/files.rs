//! Source file support for diagnostic reporting.

use std::ops::Range;

/// Files that can be used for pretty diagnostic rendering.
///
/// A lifetime parameter `'a` is provided to allow any of the returned values to returned by reference.
/// This is to workaround the lack of higher kinded lifetime parameters.
/// This can be ignored if this is not needed, however.
pub trait Files<'a> {
    type FileId: 'a + Copy + PartialEq;
    type Origin: 'a + std::fmt::Display;
    type Source: 'a + AsRef<str>;

    /// The origin of a file.
    fn origin(&'a self, id: Self::FileId) -> Option<Self::Origin>;

    /// The source of a file.
    fn source(&'a self, id: Self::FileId) -> Option<Self::Source>;

    /// The index of the line at the given byte index.
    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize>;

    /// The user-facing line number at the given line index.
    ///
    /// This can be useful for implementing something like the
    /// [C preprocessor's `#line` macro][line-macro],
    /// but is usually 1-indexed from the beginning of the file.
    ///
    /// [line-macro]: https://en.cppreference.com/w/c/preprocessor/line
    #[allow(unused_variables)]
    fn line_number(&'a self, id: Self::FileId, line_index: usize) -> Option<usize> {
        Some(line_index + 1)
    }

    /// The index of the line at the given byte index.
    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<Range<usize>>;
}

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
/// use codespan_reporting::files;
///
/// let source = "\n\n🗻∈🌏\n\n";
///
/// assert_eq!(files::column_index(source, 0..1, 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 0), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 1), 0);
/// assert_eq!(files::column_index(source, 2..13, 2 + 4), 1);
/// assert_eq!(files::column_index(source, 2..13, 2 + 8), 2);
/// assert_eq!(files::column_index(source, 2..13, 2 + 10), 2);
/// assert_eq!(files::column_index(source, 2..13, 2 + 11), 3);
/// assert_eq!(files::column_index(source, 2..13, 2 + 12), 3);
/// ```
pub fn column_index(source: &str, line_range: Range<usize>, byte_index: usize) -> usize {
    let end_index = std::cmp::min(byte_index, std::cmp::min(line_range.end, source.len()));

    (line_range.start..end_index)
        .filter(|byte_index| source.is_char_boundary(byte_index + 1))
        .count()
}

/// The 1-indexed column number at the given byte index.
///
/// # Example
///
/// ```rust
/// use codespan_reporting::files;
///
/// let source = "\n\n🗻∈🌏";
/// let line_range = 2..13;
///
/// assert_eq!(files::column_number(source, 0..1, 0), 1);
/// assert_eq!(files::column_number(source, 2..13, 0), 1);
/// assert_eq!(files::column_number(source, 2..13, 2 + 0), 1);
/// assert_eq!(files::column_number(source, 2..13, 2 + 1), 1);
/// assert_eq!(files::column_number(source, 2..13, 2 + 4), 2);
/// assert_eq!(files::column_number(source, 2..13, 2 + 8), 3);
/// assert_eq!(files::column_number(source, 2..13, 2 + 10), 3);
/// assert_eq!(files::column_number(source, 2..13, 2 + 11), 4);
/// assert_eq!(files::column_number(source, 2..13, 2 + 12), 4);
/// ```
pub fn column_number(source: &str, line_range: Range<usize>, byte_index: usize) -> usize {
    column_index(source, line_range, byte_index) + 1
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

/// Return the starting byte index of each line in the source string.
///
/// This can make it easier to implement new `Files` implementations.
///
/// # Example
///
/// ```rust
/// use codespan_reporting::files;
///
/// let source = "foo\nbar\r\n\nbaz";
/// let line_starts: Vec<_> = files::line_starts(source).collect();
///
/// assert_eq!(
///     line_starts,
///     [
///         0,  // "foo\n"
///         4,  // "bar\r\n"
///         9,  // ""
///         10, // "baz"
///     ],
/// );
///
/// fn line_index(line_starts: &[usize], byte_index: usize) -> Option<usize> {
///     match line_starts.binary_search(&byte_index) {
///         Ok(line) => Some(line),
///         Err(next_line) => Some(next_line - 1),
///     }
/// }
///
/// assert_eq!(line_index(&line_starts, 5), Some(1));
/// ```
pub fn line_starts<'source>(source: &'source str) -> impl 'source + Iterator<Item = usize> {
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
            Ordering::Less => self.line_starts.get(line_index).cloned(),
            Ordering::Equal => Some(self.source.as_ref().len()),
            Ordering::Greater => None,
        }
    }
}

impl<'a, Origin, Source> Files<'a> for SimpleFile<Origin, Source>
where
    Origin: 'a + std::fmt::Display + Clone,
    Source: 'a + AsRef<str>,
{
    type FileId = ();
    type Origin = Origin;
    type Source = &'a str;

    fn origin(&self, (): ()) -> Option<Origin> {
        Some(self.origin.clone())
    }

    fn source(&self, (): ()) -> Option<&str> {
        Some(self.source.as_ref())
    }

    fn line_index(&self, (): (), byte_index: usize) -> Option<usize> {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => Some(line),
            Err(next_line) => Some(next_line - 1),
        }
    }

    fn line_range(&self, (): (), line_index: usize) -> Option<Range<usize>> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
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

impl<'a, Origin, Source> Files<'a> for SimpleFiles<Origin, Source>
where
    Origin: 'a + std::fmt::Display + Clone,
    Source: 'a + AsRef<str>,
{
    type FileId = usize;
    type Origin = Origin;
    type Source = &'a str;

    fn origin(&self, file_id: usize) -> Option<Origin> {
        Some(self.get(file_id)?.origin().clone())
    }

    fn source(&self, file_id: usize) -> Option<&str> {
        Some(self.get(file_id)?.source().as_ref())
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Option<usize> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Option<Range<usize>> {
        self.get(file_id)?.line_range((), line_index)
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
                let line_range = file.line_range((), line).unwrap();
                &file.source[line_range]
            })
            .collect::<Vec<_>>();

        assert_eq!(line_sources, ["foo\n", "bar\r\n", "\n", "baz"]);
    }
}
