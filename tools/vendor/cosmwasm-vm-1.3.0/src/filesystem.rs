use std::{fs::create_dir_all, path::Path};

#[derive(Debug)]
pub struct MkdirPFailure;

/// An implementation for `mkdir -p`.
///
/// This is a thin wrapper around fs::create_dir_all that
/// hides all OS specific error messages to ensure they don't end up
/// breaking consensus.
pub fn mkdir_p(path: &Path) -> Result<(), MkdirPFailure> {
    create_dir_all(path).map_err(|_e| MkdirPFailure)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn mkdir_p_works() {
        let tmp_root = TempDir::new().unwrap();

        // Can create
        let path = tmp_root.path().join("something");
        assert!(!path.is_dir());
        mkdir_p(&path).unwrap();
        assert!(path.is_dir());

        // Can be called on existing dir
        let path = tmp_root.path().join("something else");
        assert!(!path.is_dir());
        mkdir_p(&path).unwrap();
        assert!(path.is_dir());
        mkdir_p(&path).unwrap(); // no-op
        assert!(path.is_dir());

        // Fails for dir with null
        let path = tmp_root.path().join("something\0with NULL");
        mkdir_p(&path).unwrap_err();
    }
}
