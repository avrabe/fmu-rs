use std::fs::metadata;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_path() {
        assert_eq!(path_exists("./"), true);
    }

    #[test]
    fn bad_path() {
        assert_eq!(path_exists("./sadsad/"), false);
    }
}

pub fn path_exists(path: &str) -> bool {
    metadata(path).is_ok()
}
