# polling

[![Build](https://github.com/smol-rs/polling/workflows/Build%20and%20test/badge.svg)](
https://github.com/smol-rs/polling/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](
https://github.com/smol-rs/polling)
[![Cargo](https://img.shields.io/crates/v/polling.svg)](
https://crates.io/crates/polling)
[![Documentation](https://docs.rs/polling/badge.svg)](
https://docs.rs/polling)

Portable interface to epoll, kqueue, event ports, and wepoll.

Supported platforms:
- [epoll](https://en.wikipedia.org/wiki/Epoll): Linux, Android
- [kqueue](https://en.wikipedia.org/wiki/Kqueue): macOS, iOS, FreeBSD, NetBSD, OpenBSD,
  DragonFly BSD
- [event ports](https://illumos.org/man/port_create): illumos, Solaris
- [poll](https://en.wikipedia.org/wiki/Poll_(Unix)): VxWorks, Fuchsia, other Unix systems
- [wepoll](https://github.com/piscisaureus/wepoll): Windows

Polling is done in oneshot mode, which means interest in I/O events needs to be reset after
an event is delivered if we're interested in the next event of the same kind.

Only one thread can be waiting for I/O events at a time.

## Examples

```rust,no_run
use polling::{Event, Poller};
use std::net::TcpListener;

// Create a TCP listener.
let socket = TcpListener::bind("127.0.0.1:8000")?;
socket.set_nonblocking(true)?;
let key = 7; // Arbitrary key identifying the socket.

// Create a poller and register interest in readability on the socket.
let poller = Poller::new()?;
poller.add(&socket, Event::readable(key))?;

// The event loop.
let mut events = Vec::new();
loop {
    // Wait for at least one I/O event.
    events.clear();
    poller.wait(&mut events, None)?;

    for ev in &events {
        if ev.key == key {
            // Perform a non-blocking accept operation.
            socket.accept()?;
            // Set interest in the next readability event.
            poller.modify(&socket, Event::readable(key))?;
        }
    }
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
