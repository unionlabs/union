//! Uses the `uds_windows` crate to simulate Unix sockets on Windows.
//!
//! Run with:
//!
//! ```
//! cargo run --example windows-uds
//! ```

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::path::PathBuf;

    use async_io::Async;
    use blocking::Unblock;
    use futures_lite::{future, io, prelude::*};
    use tempfile::tempdir;
    use uds_windows::{UnixListener, UnixStream};

    async fn client(addr: PathBuf) -> io::Result<()> {
        // Connect to the address.
        let stream = Async::new(UnixStream::connect(addr)?)?;
        println!("Connected to {:?}", stream.get_ref().peer_addr()?);

        // Pipe the stream to stdout.
        let mut stdout = Unblock::new(std::io::stdout());
        io::copy(&stream, &mut stdout).await?;
        Ok(())
    }

    let dir = tempdir()?;
    let path = dir.path().join("socket");

    future::block_on(async {
        // Create a listener.
        let listener = Async::new(UnixListener::bind(&path)?)?;
        println!("Listening on {:?}", listener.get_ref().local_addr()?);

        future::try_zip(
            async {
                // Accept the client.
                let (stream, _) = listener.read_with(|l| l.accept()).await?;
                println!("Accepted a client");

                // Send a message, drop the stream, and wait for the client.
                Async::new(stream)?.write_all(b"Hello!\n").await?;
                Ok(())
            },
            client(path),
        )
        .await?;

        Ok(())
    })
}

#[cfg(not(windows))]
fn main() {
    println!("This example works only on Windows!");
}
