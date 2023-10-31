//! io-lifetimes provides safe, convenient, and portable ways to temporarily
//! view an I/O resource as a `File`, `Socket`, or other types.

#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

use io_lifetimes::AsFilelike;
use std::fs::File;
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    let stdout = stdout();

    // With `AsFilelike`, any type implementing `AsFd`/`AsHandle` can be viewed
    // as any type supporting `FromFilelike`, so you can call `File` methods on
    // `Stdout` or other things.
    //
    // Whether or not you can actually do this is up to the OS, of course. In
    // this case, Unix can do this, but it appears Windows can't.
    let metadata = stdout.as_filelike_view::<File>().metadata()?;

    if metadata.is_file() {
        println!("stdout is a file!");
    } else {
        println!("stdout is not a file!");
    }

    Ok(())
}
