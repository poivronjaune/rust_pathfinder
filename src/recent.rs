//! Recently used map files, listed in the File menu like Word and Excel do.

use std::fs;
use std::path::{Path, PathBuf};

/// Most files listed, so each one gets a single-digit access key.
pub(crate) const MAX_RECENT_FILES: usize = 9;
/// Longest file name shown in the menu before the middle of its folder is shortened.
const MAX_LABEL_CHARS: usize = 60;

/// Most recently used map files, newest first. The list is kept in a text file in the user's
/// configuration folder, one path per line, so it survives restarts.
pub(crate) struct RecentFiles {
    paths: Vec<PathBuf>,
    store: Option<PathBuf>,
}

impl RecentFiles {
    /// Reads the list saved by an earlier session. A missing or unreadable list starts empty.
    pub(crate) fn load() -> Self {
        let store = store_path();
        let paths = store
            .as_deref()
            .and_then(|store| fs::read_to_string(store).ok())
            .map(|text| parse(&text))
            .unwrap_or_default();
        Self { paths, store }
    }

    /// A list that is never written to disk.
    #[cfg(test)]
    fn in_memory() -> Self {
        Self {
            paths: Vec::new(),
            store: None,
        }
    }

    pub(crate) fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }

    /// Moves `path` to the top of the list, adding it if needed.
    pub(crate) fn add(&mut self, path: &Path) {
        let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        self.paths.retain(|known| !same_file(known, &path));
        self.paths.insert(0, path);
        self.paths.truncate(MAX_RECENT_FILES);
        self.save();
    }

    /// Forgets `path`. Returns whether it was listed.
    pub(crate) fn remove(&mut self, path: &Path) -> bool {
        let count = self.paths.len();
        self.paths.retain(|known| !same_file(known, path));
        let removed = self.paths.len() != count;
        if removed {
            self.save();
        }
        removed
    }

    pub(crate) fn clear(&mut self) {
        self.paths.clear();
        self.save();
    }

    /// Menu text for the entry at `index`, without its access key.
    pub(crate) fn label(&self, index: usize) -> Option<String> {
        self.get(index)
            .map(|path| shorten(&path.display().to_string(), MAX_LABEL_CHARS))
    }

    fn save(&self) {
        let Some(store) = &self.store else {
            return;
        };
        let mut text = String::new();
        for path in &self.paths {
            if let Some(path) = path.to_str() {
                text.push_str(path);
                text.push('\n');
            }
        }
        let saved = match store.parent() {
            Some(folder) => fs::create_dir_all(folder).and_then(|()| fs::write(store, text)),
            None => fs::write(store, text),
        };
        if let Err(error) = saved {
            eprintln!(
                "Unable to save the recent files list to {}: {error}",
                store.display()
            );
        }
    }
}

/// Where the list is kept: `%APPDATA%\rust_pathfinder` on Windows and
/// `$XDG_CONFIG_HOME/rust_pathfinder` (or `~/.config/rust_pathfinder`) elsewhere.
fn store_path() -> Option<PathBuf> {
    let non_empty = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
    let folder = if cfg!(target_os = "windows") {
        non_empty("APPDATA").map(PathBuf::from)
    } else {
        non_empty("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| non_empty("HOME").map(|home| PathBuf::from(home).join(".config")))
    }?;
    Some(folder.join("rust_pathfinder").join("recent_files.txt"))
}

fn parse(text: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let path = PathBuf::from(line);
        if !paths.iter().any(|known| same_file(known, &path)) {
            paths.push(path);
        }
    }
    paths.truncate(MAX_RECENT_FILES);
    paths
}

/// Windows file names ignore case, so `C:\Maps\A.map` and `c:\maps\a.map` are one entry.
fn same_file(a: &Path, b: &Path) -> bool {
    if cfg!(target_os = "windows") {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

/// Shortens `text` to at most `max_chars` characters by replacing the middle with an ellipsis,
/// so the start of the path and the file name both stay visible.
fn shorten(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_owned();
    }
    let kept = max_chars.saturating_sub(1);
    let head = kept / 3;
    let tail = kept - head;
    let mut short: String = chars[..head].iter().collect();
    short.push('\u{2026}');
    short.extend(&chars[chars.len() - tail..]);
    short
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{MAX_RECENT_FILES, RecentFiles, parse, shorten};

    #[test]
    fn newest_file_comes_first_without_duplicates() {
        let mut recent = RecentFiles::in_memory();
        let (a, b) = (Path::new("a.map"), Path::new("b.map"));

        recent.add(a);
        recent.add(b);
        recent.add(a);

        let names: Vec<_> = recent.paths().iter().map(|path| path.file_name()).collect();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], a.file_name());
        assert_eq!(names[1], b.file_name());
        assert!(recent.paths().iter().all(|path| path.is_absolute()));
    }

    #[test]
    fn keeps_only_the_most_recent_files() {
        let mut recent = RecentFiles::in_memory();
        for index in 0..MAX_RECENT_FILES + 3 {
            recent.add(Path::new(&format!("map{index}.map")));
        }

        assert_eq!(recent.paths().len(), MAX_RECENT_FILES);
        let newest = format!("map{}.map", MAX_RECENT_FILES + 2);
        assert!(recent.get(0).unwrap().ends_with(newest));
    }

    #[test]
    fn remove_and_clear_forget_files() {
        let mut recent = RecentFiles::in_memory();
        recent.add(Path::new("a.map"));
        recent.add(Path::new("b.map"));
        let b = recent.get(0).unwrap().to_path_buf();

        assert!(recent.remove(&b));
        assert!(!recent.remove(&b));
        assert_eq!(recent.paths().len(), 1);
        assert!(recent.get(0).unwrap().ends_with("a.map"));

        recent.clear();
        assert!(recent.paths().is_empty());
        assert_eq!(recent.label(0), None);
    }

    #[test]
    fn saved_list_is_read_back_in_order() {
        let text = "\n/maps/one.map\n  /maps/two.map  \n/maps/one.map\n";
        assert_eq!(
            parse(text),
            vec![PathBuf::from("/maps/one.map"), PathBuf::from("/maps/two.map")]
        );
    }

    #[test]
    fn long_labels_keep_both_ends() {
        assert_eq!(shorten("short.map", 20), "short.map");

        let long = format!("/home/user/{}/backyard.map", "folder".repeat(20));
        let short = shorten(&long, 30);
        assert_eq!(short.chars().count(), 30);
        assert!(short.starts_with("/home/user"));
        assert!(short.ends_with("/backyard.map"));
        assert!(short.contains('\u{2026}'));
    }
}
