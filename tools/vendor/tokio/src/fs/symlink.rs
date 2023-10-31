use crate::fs::asyncify;

use std::io;
use std::path::Path;

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
///
/// This is an async version of [`std::os::unix::fs::symlink`][std]
///
/// [std]: std::os::unix::fs::symlink
pub async fn symlink(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref().to_owned();
    let dst = dst.as_ref().to_owned();

    asyncify(move || std::os::unix::fs::symlink(src, dst)).await
}
