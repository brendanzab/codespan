use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use filemap::{FileMap, FileName};
use index::{ByteIndex, ByteOffset};

#[derive(Debug, Default)]
pub struct CodeMap {
    files: Vec<Arc<FileMap>>,
}

impl CodeMap {
    /// Creates an empty `CodeMap`.
    pub fn new() -> CodeMap {
        CodeMap::default()
    }

    /// The next start index to use for a new filemap
    fn next_start_index(&self) -> ByteIndex {
        let end_index = self.files
            .last()
            .map(|x| x.span().end())
            .unwrap_or_else(ByteIndex::none);

        // Add one byte of padding between each file
        end_index + ByteOffset(1)
    }

    /// Adds a filemap to the codemap with the given name and source string
    pub fn add_filemap(&mut self, name: FileName, src: String) -> Arc<FileMap> {
        let file = Arc::new(FileMap::with_index(name, src, self.next_start_index()));
        self.files.push(file.clone());
        file
    }

    /// Adds a filemap to the codemap with the given name and source string
    pub fn add_filemap_from_disk<P: Into<PathBuf>>(&mut self, name: P) -> io::Result<Arc<FileMap>> {
        let file = Arc::new(FileMap::from_disk(name, self.next_start_index())?);
        self.files.push(file.clone());
        Ok(file)
    }

    /// Looks up the `File` that contains the specified byte index.
    pub fn find_file(&self, index: ByteIndex) -> Option<&Arc<FileMap>> {
        use std::cmp::Ordering;

        self.files
            .binary_search_by(|file| match () {
                () if file.span().start() > index => Ordering::Greater,
                () if file.span().end() < index => Ordering::Less,
                () => Ordering::Equal,
            })
            .ok()
            .map(|i| &self.files[i])
    }
}
