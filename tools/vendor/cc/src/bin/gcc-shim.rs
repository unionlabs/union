#![cfg_attr(test, allow(dead_code))]

use std::env;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::PathBuf;

fn main() {
    let mut args = env::args();
    let program = args.next().expect("Unexpected empty args");

    let out_dir = PathBuf::from(
        env::var_os("GCCTEST_OUT_DIR")
            .unwrap_or_else(|| panic!("{}: GCCTEST_OUT_DIR not found", program)),
    );

    // Find the first nonexistent candidate file to which the program's args can be written.
    let candidate = (0..).find_map(|i| {
        let candidate = out_dir.join(format!("out{}", i));

        if candidate.exists() {
            // If the file exists, commands have already run. Try again.
            None
        } else {
            Some(candidate)
        }
    }).unwrap_or_else(|| panic!("Cannot find the first nonexistent candidate file to which the program's args can be written under out_dir '{}'", out_dir.display()));

    // Create a file and record the args passed to the command.
    let f = File::create(&candidate).unwrap_or_else(|e| {
        panic!(
            "{}: can't create candidate: {}, error: {}",
            program,
            candidate.display(),
            e
        )
    });
    let mut f = io::BufWriter::new(f);

    (|| {
        for arg in args {
            writeln!(f, "{}", arg)?;
        }

        f.flush()?;

        let mut f = f.into_inner()?;
        f.flush()?;
        f.sync_all()
    })()
    .unwrap_or_else(|e| {
        panic!(
            "{}: can't write to candidate: {}, error: {}",
            program,
            candidate.display(),
            e
        )
    });

    // Create a file used by some tests.
    let path = &out_dir.join("libfoo.a");
    File::create(path).unwrap_or_else(|e| {
        panic!(
            "{}: can't create libfoo.a: {}, error: {}",
            program,
            path.display(),
            e
        )
    });
}
