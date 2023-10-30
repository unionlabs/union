//! Hello world, via plain syscalls.

#[cfg(all(feature = "std", not(windows)))]
fn main() -> std::io::Result<()> {
    // The message to print. It includes an explicit newline because we're
    // not using `println!`, so we have to include the newline manually.
    let message = "Hello, world!\n";

    // The bytes to print. The `write` syscall operates on byte buffers and
    // returns a byte offset if it writes fewer bytes than requested, so we
    // need the ability to compute substrings at arbitrary byte offsets.
    let mut bytes = message.as_bytes();

    // Safety: See [here] for the safety conditions for calling `stdout`. In
    // this example, the code is inside `main` itself so we know how `stdout`
    // is being used and we know that it's not dropped.
    //
    // [here]: https://docs.rs/rustix/*/rustix/io/fn.stdout.html#safety
    let stdout = unsafe { rustix::io::stdout() };

    while !bytes.is_empty() {
        match rustix::io::write(&stdout, bytes) {
            // `write` can write fewer bytes than requested. In that case,
            // continue writing with the remainder of the bytes.
            Ok(n) => bytes = &bytes[n..],

            // `write` can be interrupted before doing any work; if that
            // happens, retry it.
            Err(rustix::io::Errno::INTR) => (),

            // `write` can also fail for external reasons, such as running out
            // of storage space.
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

#[cfg(any(not(feature = "std"), windows))]
fn main() {
    unimplemented!()
}
