//! This is an example of how to use `dup2` to replace the stdin and stdout
//! file descriptors.

#[cfg(not(windows))]
fn main() {
    use rustix::io::{dup2, pipe};
    use std::io::{BufRead, BufReader};
    use std::mem::forget;

    // Create some new file descriptors that we'll use to replace stdio's file
    // descriptors with.
    let (reader, writer) = pipe().unwrap();

    // Acquire `OwnedFd` instances for stdin and stdout. These APIs are `unsafe`
    // because in general, with low-level APIs like this, libraries can't assume
    // that stdin and stdout will be open or safe to use. It's ok here, because
    // we're directly inside `main`, so we know that stdin and stdout haven't
    // been closed and aren't being used for other purposes.
    let (mut stdin, mut stdout) = unsafe { (rustix::io::take_stdin(), rustix::io::take_stdout()) };

    // Use `dup2` to copy our new file descriptors over the stdio file descriptors.
    //
    // These take their second argument as an `&mut OwnedFd` rather than the
    // usual `impl AsFd` because they conceptually do a `close` on the original
    // file descriptor, which one shouldn't be able to do with just a
    // `BorrowedFd`.
    dup2(&reader, &mut stdin).unwrap();
    dup2(&writer, &mut stdout).unwrap();

    // Then, forget the stdio `OwnedFd`s, because actually dropping them would
    // close them. Here, we want stdin and stdout to remain open for the rest
    // of the program.
    forget(stdin);
    forget(stdout);

    // We can also drop the original file descriptors now, since `dup2` creates
    // new file descriptors with independent lifetimes.
    drop(reader);
    drop(writer);

    // Now we can print to "stdout" in the usual way, and it'll go to our pipe.
    println!("hello, world!");

    // And we can read from stdin, and it'll read from our pipe. It's a little
    // silly that we connected our stdout to our own stdin, but it's just an
    // example :-).
    let mut s = String::new();
    BufReader::new(std::io::stdin()).read_line(&mut s).unwrap();
    assert_eq!(s, "hello, world!\n");
}

#[cfg(windows)]
fn main() {
    unimplemented!()
}
