# `mac_address`

[![crates.io](https://img.shields.io/crates/v/mac_address.svg)](https://crates.io/crates/mac_address)
[![Released API docs](https://docs.rs/mac_address/badge.svg)](https://docs.rs/mac_address)

`mac_address` provides a cross platform way to retrieve the [MAC address](https://en.wikipedia.org/wiki/MAC_address) of network hardware.

Supported platforms: Linux, Windows, MacOS, FreeBSD, OpenBSD

## Example

```rust
extern crate mac_address;

use mac_address::get_mac_address;

fn main() {
    match get_mac_address() {
        Ok(Some(ma)) => {
            println!("MAC addr = {}", ma);
            println!("bytes = {:?}", ma.bytes());
        }
        Ok(None) => println!("No MAC address found."),
        Err(e) => println!("{:?}", e),
    }
}
```

## License

`mac_address` is licensed under both MIT and Apache 2.0
