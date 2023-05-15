use std::path::PathBuf;

/// Create a tempdir and copy the passed directories from testdata to the tempdir. Tempdirs are stored in the target
/// directory for debugging purposes. This does not retain symlinks however.
///
/// # Usage
/// ```
/// use crate::testdata::temp_dir_with;
///
/// // creates a directory akin to tmpdir.test_temp_dir_with/{test_temp_dir_with, bins} in the target folder.
/// let dir = temp_dir_with("test_temp_dir_with", &["test_temp_dir_with", "bins"]);
/// std::fs::File::open(dir.into_path().join("test_temp_dir_with").join("foo.bar")).unwrap();
/// ```
pub fn temp_dir_with(dirs: &[&str]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("creating tempdir");
    let testdata = PathBuf::from("src/testdata");
    for dir in dirs {
        let to = tmp.path().to_owned().join(dir);
        let from = testdata.join(dir);
        let options = fs_extra::dir::CopyOptions::default().copy_inside(true);
        fs_extra::dir::copy(from.as_path(), to, &options).unwrap();
    }
    tmp
}
mod tests {
    use super::*;

    #[test]
    fn test_temp_dir_with() {
        let dir = temp_dir_with(&["test_temp_dir_with"]);
        std::fs::File::open(dir.into_path().join("test_temp_dir_with").join("foo.bar")).unwrap();
    }
}
