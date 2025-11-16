// Recent ROMs list management
//
// Tracks recently opened ROM files for quick access.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default recent ROMs file path
const RECENT_ROMS_FILE: &str = "recent_roms.toml";

/// Maximum number of recent ROMs to track
const MAX_RECENT_ROMS: usize = 10;

/// Recent ROMs list
///
/// Maintains a list of recently opened ROM files with metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentRomsList {
    /// List of recent ROM entries (most recent first)
    roms: Vec<RecentRomEntry>,
}

/// Entry for a recently opened ROM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRomEntry {
    /// Path to the ROM file
    pub path: PathBuf,

    /// Last accessed timestamp
    pub last_accessed: String,

    /// Display name (file name without extension)
    pub display_name: String,
}

impl RecentRomsList {
    /// Create a new empty recent ROMs list
    ///
    /// # Returns
    ///
    /// A new empty list
    pub fn new() -> Self {
        Self::default()
    }

    /// Load recent ROMs list from file or create default
    ///
    /// # Returns
    ///
    /// The loaded or default list
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    /// Load recent ROMs list from file
    ///
    /// # Returns
    ///
    /// Result containing the list or an error
    pub fn load() -> Result<Self, io::Error> {
        let contents = fs::read_to_string(RECENT_ROMS_FILE)?;
        toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save recent ROMs list to file
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn save(&self) -> Result<(), io::Error> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(RECENT_ROMS_FILE, contents)
    }

    /// Add a ROM to the recent list
    ///
    /// If the ROM is already in the list, it's moved to the top.
    /// If the list exceeds MAX_RECENT_ROMS, the oldest entry is removed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ROM file
    pub fn add<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();

        // Remove existing entry if present
        self.roms.retain(|entry| entry.path != path);

        // Get display name
        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Create new entry
        let entry = RecentRomEntry {
            path: path.to_path_buf(),
            last_accessed: chrono::Local::now().to_rfc3339(),
            display_name,
        };

        // Insert at the beginning (most recent)
        self.roms.insert(0, entry);

        // Trim to maximum size
        if self.roms.len() > MAX_RECENT_ROMS {
            self.roms.truncate(MAX_RECENT_ROMS);
        }
    }

    /// Remove a ROM from the recent list
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ROM file to remove
    pub fn remove<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.roms.retain(|entry| entry.path != path);
    }

    /// Clear all recent ROMs
    pub fn clear(&mut self) {
        self.roms.clear();
    }

    /// Get all recent ROM entries
    ///
    /// # Returns
    ///
    /// Slice of recent ROM entries (most recent first)
    pub fn entries(&self) -> &[RecentRomEntry] {
        &self.roms
    }

    /// Get the most recent ROM path
    ///
    /// # Returns
    ///
    /// Option containing the most recent ROM path, or None if list is empty
    pub fn most_recent(&self) -> Option<&Path> {
        self.roms.first().map(|entry| entry.path.as_path())
    }

    /// Check if the list is empty
    ///
    /// # Returns
    ///
    /// true if the list is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.roms.is_empty()
    }

    /// Get the number of ROMs in the list
    ///
    /// # Returns
    ///
    /// The number of ROMs in the list
    pub fn len(&self) -> usize {
        self.roms.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rom() {
        let mut list = RecentRomsList::new();
        assert!(list.is_empty());

        list.add("test1.nes");
        assert_eq!(list.len(), 1);

        list.add("test2.nes");
        assert_eq!(list.len(), 2);

        // Most recent should be test2.nes
        assert_eq!(list.most_recent().unwrap(), Path::new("test2.nes"));
    }

    #[test]
    fn test_add_duplicate() {
        let mut list = RecentRomsList::new();

        list.add("test1.nes");
        list.add("test2.nes");
        list.add("test1.nes"); // Add duplicate

        assert_eq!(list.len(), 2);

        // test1.nes should be at the top now
        assert_eq!(list.most_recent().unwrap(), Path::new("test1.nes"));
    }

    #[test]
    fn test_max_recent_roms() {
        let mut list = RecentRomsList::new();

        // Add more than MAX_RECENT_ROMS
        for i in 0..15 {
            list.add(format!("test{}.nes", i));
        }

        // Should be capped at MAX_RECENT_ROMS
        assert_eq!(list.len(), MAX_RECENT_ROMS);

        // Most recent should be test14.nes
        assert_eq!(list.most_recent().unwrap(), Path::new("test14.nes"));
    }

    #[test]
    fn test_remove_rom() {
        let mut list = RecentRomsList::new();

        list.add("test1.nes");
        list.add("test2.nes");
        list.add("test3.nes");

        list.remove("test2.nes");

        assert_eq!(list.len(), 2);
        assert!(!list
            .entries()
            .iter()
            .any(|e| e.path == Path::new("test2.nes")));
    }

    #[test]
    fn test_clear() {
        let mut list = RecentRomsList::new();

        list.add("test1.nes");
        list.add("test2.nes");

        list.clear();

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }
}
