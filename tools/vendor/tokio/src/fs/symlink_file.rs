use crate::fs::asyncify;

use std::io;
use std::path::Path;

/// Creates a new file symbolic link on the filesystem.
///
/// The `dst` path will be a file symbolic link pointing to the `src`
/// path.
///
/// This is an async version of [`std::os::windows::fs::symlink_file`][std]
///
/// [std]: std::os::windows::fs::symlink_file
pub async fn symlink_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref().to_owned();
    let dst = dst.as_ref().to_owned();

    asyncify(move || std::os::windows::fs::symlink_file(src, dst)).await
}
