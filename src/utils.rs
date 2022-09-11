use std::{fs::metadata, path::PathBuf};

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn current_path() {
        assert_eq!(path_exists("./"), true);
    }

    #[test]
    fn bad_path() {
        assert_eq!(path_exists("./sadsad/"), false);
    }

    #[test]
    fn path_is_not_empty() {
        assert_eq!(path_is_empty("./"), false);
    }

    #[test]
    fn test_path_is_empty() {
        let tmp_dir = TempDir::new("example").unwrap();
        assert!(path_is_empty(tmp_dir.path().to_str().unwrap()));
        tmp_dir.close().unwrap();
    }
}

pub fn path_exists(path: &str) -> bool {
    metadata(path).is_ok()
}

pub fn path_is_empty(path: &str) -> bool {
    let dir_path_buf = PathBuf::from(path);
    dir_path_buf.read_dir().unwrap().next().is_none()
}
