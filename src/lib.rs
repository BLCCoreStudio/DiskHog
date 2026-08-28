#![forbid(unsafe_code)]

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

impl EntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "dir",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub size: u64,
    pub kind: EntryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanOptions {
    pub include_files: bool,
    pub include_dirs: bool,
    pub max_depth: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_files: true,
            include_dirs: true,
            max_depth: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    pub entries: Vec<Entry>,
    pub issues: Vec<ScanIssue>,
}

pub fn scan(path: impl AsRef<Path>, options: ScanOptions) -> io::Result<ScanReport> {
    let path = path.as_ref();
    let metadata = fs::symlink_metadata(path)?;

    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the root path must not be a symbolic link",
        ));
    }

    let mut entries = Vec::new();
    let mut issues = Vec::new();

    if metadata.is_file() {
        if options.include_files && depth_is_visible(0, options.max_depth) {
            entries.push(Entry {
                path: path.to_path_buf(),
                size: disk_usage_bytes(&metadata),
                kind: EntryKind::File,
            });
        }
    } else if metadata.is_dir() {
        scan_directory(path, 0, options, &mut entries, &mut issues);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the root path is neither a regular file nor a directory",
        ));
    }

    entries.sort_by(|left, right| {
        right
            .size
            .cmp(&left.size)
            .then_with(|| left.path.cmp(&right.path))
    });

    Ok(ScanReport { entries, issues })
}

fn scan_directory(
    path: &Path,
    depth: usize,
    options: ScanOptions,
    entries: &mut Vec<Entry>,
    issues: &mut Vec<ScanIssue>,
) -> u64 {
    let directory = match fs::read_dir(path) {
        Ok(directory) => directory,
        Err(error) => {
            issues.push(ScanIssue {
                path: path.to_path_buf(),
                message: error.to_string(),
            });
            return 0;
        }
    };

    let mut total = 0_u64;

    for item in directory {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                issues.push(ScanIssue {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                });
                continue;
            }
        };

        let child_path = item.path();
        let metadata = match fs::symlink_metadata(&child_path) {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(ScanIssue {
                    path: child_path,
                    message: error.to_string(),
                });
                continue;
            }
        };

        if metadata.file_type().is_symlink() {
            continue;
        }

        let child_depth = depth.saturating_add(1);

        if metadata.is_file() {
            let size = disk_usage_bytes(&metadata);
            total = total.saturating_add(size);

            if options.include_files && depth_is_visible(child_depth, options.max_depth) {
                entries.push(Entry {
                    path: child_path,
                    size,
                    kind: EntryKind::File,
                });
            }
        } else if metadata.is_dir() {
            let size = scan_directory(&child_path, child_depth, options, entries, issues);
            total = total.saturating_add(size);

            if options.include_dirs && depth_is_visible(child_depth, options.max_depth) {
                entries.push(Entry {
                    path: child_path,
                    size,
                    kind: EntryKind::Directory,
                });
            }
        }
    }

    total
}

fn depth_is_visible(depth: usize, max_depth: Option<usize>) -> bool {
    match max_depth {
        Some(max_depth) => depth <= max_depth,
        None => true,
    }
}

#[cfg(unix)]
fn disk_usage_bytes(metadata: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;

    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn disk_usage_bytes(metadata: &Metadata) -> u64 {
    metadata.len()
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit_index = 0_usize;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{value:.1} {}", UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "diskhog-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn human_size_uses_binary_units() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1024 * 1024), "1.0 MiB");
        assert_eq!(human_size(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn scan_sorts_largest_first() {
        let root = TestDir::new();
        fs::write(root.0.join("small.bin"), vec![0_u8; 1024]).unwrap();
        fs::write(root.0.join("large.bin"), vec![0_u8; 64 * 1024]).unwrap();

        let report = scan(
            &root.0,
            ScanOptions {
                include_files: true,
                include_dirs: false,
                max_depth: None,
            },
        )
        .unwrap();

        assert_eq!(report.entries.len(), 2);
        assert!(report.entries[0].size >= report.entries[1].size);
        assert!(report.entries[0].path.ends_with("large.bin"));
    }

    #[test]
    fn depth_limits_display_but_not_directory_totals() {
        let root = TestDir::new();
        let nested = root.0.join("one").join("two");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("payload.bin"), vec![1_u8; 8192]).unwrap();

        let report = scan(
            &root.0,
            ScanOptions {
                include_files: false,
                include_dirs: true,
                max_depth: Some(1),
            },
        )
        .unwrap();

        assert_eq!(report.entries.len(), 1);
        assert!(report.entries[0].path.ends_with("one"));
        assert!(report.entries[0].size > 0);
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_links_are_not_followed() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new();
        let nested = root.0.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("payload.bin"), vec![1_u8; 8192]).unwrap();
        symlink(&root.0, nested.join("loop")).unwrap();

        let report = scan(&root.0, ScanOptions::default()).unwrap();

        assert!(report.entries.len() < 10);
        assert!(report
            .entries
            .iter()
            .all(|entry| !entry.path.ends_with("loop")));
    }
}
