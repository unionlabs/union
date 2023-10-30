tempdir
=======

A Rust library for creating a temporary directory and deleting its entire
contents when the directory is dropped.

[![Build Status](https://travis-ci.org/rust-lang-nursery/tempdir.svg?branch=master)](https://travis-ci.org/rust-lang-nursery/tempdir)
[![Build status](https://ci.appveyor.com/api/projects/status/2mp24396db5t4hul/branch/master?svg=true)](https://ci.appveyor.com/project/rust-lang-libs/tempdir/branch/master)

[Documentation](https://doc.rust-lang.org/tempdir)

## Deprecation Note

The `tempdir` crate is being merged into [`tempfile`](https://github.com/Stebalien/tempfile). Please see [this issue](https://github.com/Stebalien/tempfile/issues/43) to track progress and direct new issues and pull requests to `tempfile`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tempdir = "0.3"
```

and this to your crate root:

```rust
extern crate tempdir;
```

## Example

This sample method does the following:

1. Create a temporary directory in the default location with the given prefix.
2. Determine a file path in the directory and print it out.
3. Create a file inside the temp folder.
4. Write to the file and sync it to disk.
5. Close the directory, deleting the contents in the process.

```rust
use std::io::{self, Write};
use std::fs::File;
use tempdir::TempDir;

fn write_temp_folder_with_files() -> io::Result<()> {
    let dir = TempDir::new("my_directory_prefix")?;
    let file_path = dir.path().join("foo.txt");
    println!("{:?}", file_path);

    let mut f = File::create(file_path)?;
    f.write_all(b"Hello, world!")?;
    f.sync_all()?;
    dir.close()?;

    Ok(())
}
```

**Note:** Closing the directory is actually optional, as it would be done on
drop. The benefit of closing here is that it allows possible errors to be
handled.
