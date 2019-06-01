#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;

use itertools::Itertools;

use crate::{
    file::File,
    index::{ByteIndex, ByteOffset, RawIndex},
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct Files<S = String> {
    files: Vec<Arc<File<S>>>,
}

impl<S> Files<S> {
    /// Creates an empty `Files`.
    pub fn new() -> Files<S> {
        Files::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<File<S>>> {
        self.files.iter()
    }
}

impl<S: AsRef<str>> Files<S> {
    /// The next start index to use for a new file
    fn next_start_index(&self) -> ByteIndex {
        let end_index = self
            .files
            .last()
            .map(|x| x.span().end())
            .unwrap_or_else(ByteIndex::none);

        // Add one byte of padding between each file
        end_index + ByteOffset(1)
    }

    /// Adds a file to the files with the given name and source string
    pub fn add_file(&mut self, name: String, src: S) -> Arc<File<S>> {
        let file = Arc::new(File::with_index(name, src, self.next_start_index()));
        self.files.push(file.clone());
        file
    }

    /// Looks up the `File` that contains the specified byte index.
    pub fn find_file(&self, index: ByteIndex) -> Option<&Arc<File<S>>> {
        self.find_index(index).map(|i| &self.files[i])
    }

    pub fn update(&mut self, index: ByteIndex, src: S) -> Option<Arc<File<S>>> {
        self.find_index(index).map(|i| {
            let min = if i == 0 {
                ByteIndex(1)
            } else {
                self.files[i - 1].span().end() + ByteOffset(1)
            };
            let max = self
                .files
                .get(i + 1)
                .map_or(ByteIndex(RawIndex::max_value()), |file| file.span().start())
                - ByteOffset(1);
            if src.as_ref().len() <= (max - min).to_usize() {
                let start_index = self.files[i].span().start();
                let name = self.files[i].name().to_owned();
                let new_file = Arc::new(File::with_index(name, src, start_index));
                self.files[i] = new_file.clone();
                new_file
            } else {
                let file = self.files.remove(i);
                match self
                    .files
                    .first()
                    .map(|file| file.span().start().to_usize() - 1)
                    .into_iter()
                    .chain(self.files.iter().tuple_windows().map(|(x, y)| {
                        eprintln!("{} {}", x.span(), y.span());
                        (y.span().start() - x.span().end()).to_usize() - 1
                    }))
                    .position(|size| size >= src.as_ref().len() + 1)
                {
                    Some(j) => {
                        let start_index = if j == 0 {
                            ByteIndex(1)
                        } else {
                            self.files[j - 1].span().end() + ByteOffset(1)
                        };
                        let name = file.name().to_owned();
                        let new_file = Arc::new(File::with_index(name, src, start_index));
                        self.files.insert(j, new_file.clone());
                        new_file
                    },
                    None => self.add_file(file.name().to_owned(), src),
                }
            }
        })
    }

    fn find_index(&self, index: ByteIndex) -> Option<usize> {
        use std::cmp::Ordering;

        self.files
            .binary_search_by(|file| match () {
                () if file.span().start() > index => Ordering::Greater,
                () if file.span().end() < index => Ordering::Less,
                () => Ordering::Equal,
            })
            .ok()
    }
}

impl<S: AsRef<str> + From<String>> Files<S> {
    /// Adds a file to the files with the given name and source string
    pub fn add_file_from_disk(&mut self, name: String) -> io::Result<Arc<File<S>>> {
        let file = Arc::new(File::from_disk(name, self.next_start_index())?);
        self.files.push(file.clone());
        Ok(file)
    }
}

impl<S> Default for Files<S> {
    fn default() -> Self {
        Files { files: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::span::Span;

    fn check_maps(files: &Files, expected_files: &[(RawIndex, &str, &str)]) {
        println!("{:?}", files);
        assert_eq!(files.files.len(), expected_files.len());
        let mut prev_span = Span::new(0.into(), 0.into());
        for (i, (file, &(start, name, src))) in files.files.iter().zip(expected_files).enumerate() {
            println!("{}: {:?} <=> {:?}", i, file, (start, name, src));
            assert_eq!(file.name(), name, "At index {}", i);
            assert_eq!(ByteIndex(start), file.span().start(), "At index {}", i);
            assert!(prev_span.end() < file.span().start(), "At index {}", i);
            assert_eq!(file.src(), src, "At index {}", i);

            prev_span = file.span();
        }
    }

    #[test]
    fn update() {
        let mut files = Files::new();

        let a_span = files.add_file("a".into(), "a".into()).span();
        let b_span = files.add_file("b".into(), "b".into()).span();
        let c_span = files.add_file("c".into(), "c".into()).span();

        files.update(a_span.start(), "aa".into()).unwrap();
        check_maps(&files, &[(3, "b", "b"), (5, "c", "c"), (7, "a", "aa")]);

        files.update(b_span.start(), "".into()).unwrap().span();
        check_maps(&files, &[(3, "b", ""), (5, "c", "c"), (7, "a", "aa")]);

        files.update(c_span.start(), "ccc".into()).unwrap();
        check_maps(&files, &[(3, "b", ""), (7, "a", "aa"), (10, "c", "ccc")]);
    }
}
