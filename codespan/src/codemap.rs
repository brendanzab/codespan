use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use {FileMap, FileName};
use pos::{ByteOffset, BytePos};

#[derive(Debug)]
pub struct CodeMap {
    files: Vec<Arc<FileMap>>,
}

impl CodeMap {
    /// Creates an empty `CodeMap`.
    pub fn new() -> CodeMap {
        CodeMap { files: Vec::new() }
    }

    /// The next start position to use for a new filemap
    fn next_start_pos(&self) -> BytePos {
        let end_pos = self.files
            .last()
            .map(|x| x.span().hi())
            .unwrap_or(BytePos::none());

        // Add one byte of padding between each file
        end_pos + ByteOffset(1)
    }

    /// Adds a filemap to the codemap with the given name and source string
    pub fn add_filemap(&mut self, name: FileName, src: String) -> Arc<FileMap> {
        let file = Arc::new(FileMap::new(name, src, self.next_start_pos()));
        self.files.push(file.clone());
        file
    }

    /// Adds a filemap to the codemap with the given name and source string
    pub fn add_filemap_from_disk<P: Into<PathBuf>>(&mut self, name: P) -> io::Result<Arc<FileMap>> {
        let file = Arc::new(FileMap::from_disk(name, self.next_start_pos())?);
        self.files.push(file.clone());
        Ok(file)
    }

    /// Looks up the `File` that contains the specified byte position.
    pub fn find_file(&self, pos: BytePos) -> Option<&Arc<FileMap>> {
        use std::cmp::Ordering;

        self.files
            .binary_search_by(|file| match () {
                () if file.span().lo() > pos => Ordering::Greater,
                () if file.span().hi() < pos => Ordering::Less,
                () => Ordering::Equal,
            })
            .ok()
            .map(|i| &self.files[i])
    }
}
