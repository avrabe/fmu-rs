use std::{fs::metadata, path::PathBuf};

pub fn path_exists(path: &str) -> bool {
    metadata(path).is_ok()
}

pub fn path_is_empty(path: &str) -> bool {
    let dir_path_buf = PathBuf::from(path);
    dir_path_buf.read_dir().unwrap().next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn current_path() {
        assert!(path_exists("./"));
    }

    #[test]
    fn bad_path() {
        assert!(!path_exists("./sadsad/"));
    }

    #[test]
    fn path_is_not_empty() {
        assert!(!path_is_empty("./"));
    }

    #[test]
    fn test_path_is_empty() {
        let tmp_dir = TempDir::new("example").unwrap();
        assert!(path_is_empty(tmp_dir.path().to_str().unwrap()));
        tmp_dir.close().unwrap();
    }
}
