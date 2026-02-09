//! Archive storage trait for abstraction over storage backends.
//!
//! This module provides the [`ArchiveStorage`] trait which enables testing
//! archive operations without actual ZIP file I/O, and supports future
//! alternative backends.

use std::collections::HashMap;

use crate::Result;

/// Trait for archive storage backends.
///
/// This trait abstracts over the underlying storage mechanism (ZIP files,
/// in-memory storage, etc.) to enable unit testing and support alternative
/// backends.
pub trait ArchiveStorage {
    /// Read a file from the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist or cannot be read.
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>>;

    /// Write a file to the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()>;

    /// Check if a file exists in the archive.
    fn file_exists(&self, path: &str) -> bool;

    /// Get all file names in the archive.
    fn file_names(&self) -> Vec<String>;
}

/// In-memory storage for testing purposes.
///
/// This implementation stores all files in memory, enabling unit tests
/// to run without filesystem access or ZIP operations.
///
/// # Example
///
/// ```
/// use cdx_core::archive::MemoryStorage;
/// use cdx_core::archive::ArchiveStorage;
///
/// let mut storage = MemoryStorage::new();
/// storage.write_file("test.txt", b"Hello, world!").unwrap();
///
/// assert!(storage.file_exists("test.txt"));
/// assert_eq!(storage.read_file("test.txt").unwrap(), b"Hello, world!");
/// ```
#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    files: HashMap<String, Vec<u8>>,
}

impl MemoryStorage {
    /// Create a new empty in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Create a storage with pre-populated files.
    #[must_use]
    pub fn with_files(files: HashMap<String, Vec<u8>>) -> Self {
        Self { files }
    }

    /// Get the number of files stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if the storage is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Remove a file from storage.
    ///
    /// Returns the file contents if it existed.
    pub fn remove(&mut self, path: &str) -> Option<Vec<u8>> {
        self.files.remove(path)
    }

    /// Clear all files from storage.
    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl ArchiveStorage for MemoryStorage {
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| crate::Error::MissingFile {
                path: path.to_string(),
            })
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()> {
        self.files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn file_names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_new() {
        let storage = MemoryStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_memory_storage_write_read() {
        let mut storage = MemoryStorage::new();

        storage.write_file("test.txt", b"Hello").unwrap();
        assert_eq!(storage.read_file("test.txt").unwrap(), b"Hello");
    }

    #[test]
    fn test_memory_storage_file_exists() {
        let mut storage = MemoryStorage::new();

        assert!(!storage.file_exists("test.txt"));
        storage.write_file("test.txt", b"data").unwrap();
        assert!(storage.file_exists("test.txt"));
    }

    #[test]
    fn test_memory_storage_file_names() {
        let mut storage = MemoryStorage::new();

        storage.write_file("a.txt", b"a").unwrap();
        storage.write_file("b.txt", b"b").unwrap();

        let names = storage.file_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a.txt".to_string()));
        assert!(names.contains(&"b.txt".to_string()));
    }

    #[test]
    fn test_memory_storage_read_missing() {
        let mut storage = MemoryStorage::new();
        let result = storage.read_file("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_storage_overwrite() {
        let mut storage = MemoryStorage::new();

        storage.write_file("test.txt", b"first").unwrap();
        storage.write_file("test.txt", b"second").unwrap();

        assert_eq!(storage.read_file("test.txt").unwrap(), b"second");
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_memory_storage_remove() {
        let mut storage = MemoryStorage::new();

        storage.write_file("test.txt", b"data").unwrap();
        let removed = storage.remove("test.txt");

        assert_eq!(removed, Some(b"data".to_vec()));
        assert!(!storage.file_exists("test.txt"));
    }

    #[test]
    fn test_memory_storage_remove_nonexistent() {
        let mut storage = MemoryStorage::new();
        assert!(storage.remove("nonexistent").is_none());
    }

    #[test]
    fn test_memory_storage_clear() {
        let mut storage = MemoryStorage::new();

        storage.write_file("a.txt", b"a").unwrap();
        storage.write_file("b.txt", b"b").unwrap();
        storage.clear();

        assert!(storage.is_empty());
    }

    #[test]
    fn test_memory_storage_with_files() {
        let mut files = HashMap::new();
        files.insert("existing.txt".to_string(), b"content".to_vec());

        let mut storage = MemoryStorage::with_files(files);

        assert!(storage.file_exists("existing.txt"));
        assert_eq!(storage.read_file("existing.txt").unwrap(), b"content");
    }

    #[test]
    fn test_memory_storage_binary_data() {
        let mut storage = MemoryStorage::new();
        let binary = vec![0x00, 0xFF, 0x7F, 0x80, 0x01];

        storage.write_file("binary.dat", &binary).unwrap();
        assert_eq!(storage.read_file("binary.dat").unwrap(), binary);
    }

    #[test]
    fn test_memory_storage_empty_file() {
        let mut storage = MemoryStorage::new();

        storage.write_file("empty.txt", b"").unwrap();

        assert!(storage.file_exists("empty.txt"));
        assert!(storage.read_file("empty.txt").unwrap().is_empty());
    }

    #[test]
    fn test_memory_storage_nested_paths() {
        let mut storage = MemoryStorage::new();

        storage
            .write_file("path/to/nested/file.txt", b"nested")
            .unwrap();

        assert!(storage.file_exists("path/to/nested/file.txt"));
        assert_eq!(
            storage.read_file("path/to/nested/file.txt").unwrap(),
            b"nested"
        );
    }
}
