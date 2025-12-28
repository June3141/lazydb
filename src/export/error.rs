//! Export error types with user-friendly messages

use std::path::PathBuf;

/// Export operation errors
#[derive(Debug)]
pub enum ExportError {
    /// Failed to create or write to the file
    FileWriteError {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Directory does not exist
    DirectoryNotFound { path: PathBuf },
    /// Permission denied when writing file
    PermissionDenied { path: PathBuf },
    /// Disk full or quota exceeded
    DiskFull { path: PathBuf },
    /// Encoding error
    EncodingError { message: String },
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::FileWriteError { path, source } => {
                write!(
                    f,
                    "Failed to write file '{}': {} (Check file permissions and disk space)",
                    path.display(),
                    source
                )
            }
            ExportError::DirectoryNotFound { path } => {
                write!(
                    f,
                    "Directory not found: '{}' (Create the directory first or choose a different location)",
                    path.display()
                )
            }
            ExportError::PermissionDenied { path } => {
                write!(
                    f,
                    "Permission denied: Cannot write to '{}' (Check file/folder permissions)",
                    path.display()
                )
            }
            ExportError::DiskFull { path } => {
                write!(
                    f,
                    "Disk full: Cannot write to '{}' (Free up disk space and try again)",
                    path.display()
                )
            }
            ExportError::EncodingError { message } => {
                write!(f, "Encoding error: {} (Try using UTF-8 encoding)", message)
            }
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::FileWriteError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl ExportError {
    /// Create an ExportError from an io::Error, categorizing it appropriately
    pub fn from_io_error(err: std::io::Error, path: PathBuf) -> Self {
        use std::io::ErrorKind;

        match err.kind() {
            ErrorKind::NotFound => ExportError::DirectoryNotFound { path },
            ErrorKind::PermissionDenied => ExportError::PermissionDenied { path },
            // StorageFull is not stable yet, so we check the raw_os_error
            _ if Self::is_disk_full_error(&err) => ExportError::DiskFull { path },
            _ => ExportError::FileWriteError { path, source: err },
        }
    }

    /// Check if the error is a disk full error
    fn is_disk_full_error(err: &std::io::Error) -> bool {
        // ENOSPC on Unix, ERROR_DISK_FULL on Windows
        #[cfg(unix)]
        {
            err.raw_os_error() == Some(28) // ENOSPC
        }
        #[cfg(windows)]
        {
            err.raw_os_error() == Some(112) // ERROR_DISK_FULL
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = err;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_file_write_error_displays_path_and_hint() {
        let err = ExportError::FileWriteError {
            path: PathBuf::from("/tmp/test.csv"),
            source: io::Error::other("write failed"),
        };
        let display = err.to_string();

        assert!(
            display.contains("/tmp/test.csv"),
            "Expected path in message, got: {}",
            display
        );
        assert!(
            display.contains("permission") || display.contains("disk"),
            "Expected hint about permissions or disk, got: {}",
            display
        );
    }

    #[test]
    fn test_directory_not_found_displays_user_friendly_message() {
        let err = ExportError::DirectoryNotFound {
            path: PathBuf::from("/nonexistent/dir/file.csv"),
        };
        let display = err.to_string();

        assert!(
            display.contains("Directory not found") || display.contains("ディレクトリ"),
            "Expected directory not found message, got: {}",
            display
        );
        assert!(
            display.contains("/nonexistent/dir/file.csv"),
            "Expected path in message, got: {}",
            display
        );
        assert!(
            display.contains("Create") || display.contains("作成"),
            "Expected hint to create directory, got: {}",
            display
        );
    }

    #[test]
    fn test_permission_denied_displays_user_friendly_message() {
        let err = ExportError::PermissionDenied {
            path: PathBuf::from("/protected/file.csv"),
        };
        let display = err.to_string();

        assert!(
            display.contains("Permission denied") || display.contains("権限"),
            "Expected permission denied message, got: {}",
            display
        );
        assert!(
            display.contains("/protected/file.csv"),
            "Expected path in message, got: {}",
            display
        );
    }

    #[test]
    fn test_disk_full_displays_user_friendly_message() {
        let err = ExportError::DiskFull {
            path: PathBuf::from("/tmp/large_file.csv"),
        };
        let display = err.to_string();

        assert!(
            display.contains("Disk full") || display.contains("ディスク"),
            "Expected disk full message, got: {}",
            display
        );
        assert!(
            display.contains("Free up") || display.contains("空き"),
            "Expected hint to free up space, got: {}",
            display
        );
    }

    #[test]
    fn test_encoding_error_displays_user_friendly_message() {
        let err = ExportError::EncodingError {
            message: "invalid UTF-8 sequence".to_string(),
        };
        let display = err.to_string();

        assert!(
            display.contains("Encoding") || display.contains("エンコーディング"),
            "Expected encoding error message, got: {}",
            display
        );
        assert!(
            display.contains("UTF-8"),
            "Expected UTF-8 hint, got: {}",
            display
        );
    }

    #[test]
    fn test_from_io_error_categorizes_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file or directory");
        let err = ExportError::from_io_error(io_err, PathBuf::from("/test/path"));

        assert!(matches!(err, ExportError::DirectoryNotFound { .. }));
    }

    #[test]
    fn test_from_io_error_categorizes_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err = ExportError::from_io_error(io_err, PathBuf::from("/test/path"));

        assert!(matches!(err, ExportError::PermissionDenied { .. }));
    }

    #[test]
    fn test_from_io_error_fallback_to_file_write_error() {
        let io_err = io::Error::other("some other error");
        let err = ExportError::from_io_error(io_err, PathBuf::from("/test/path"));

        assert!(matches!(err, ExportError::FileWriteError { .. }));
    }
}
