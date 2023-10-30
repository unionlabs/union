# crc

[![rust](https://github.com/mrhooray/crc-rs/actions/workflows/rust.yaml/badge.svg)](https://github.com/mrhooray/crc-rs/actions/workflows/rust.yaml)
[![Crate](https://img.shields.io/crates/v/crc.svg)](https://crates.io/crates/crc)
[![Docs](https://docs.rs/crc/badge.svg)](https://docs.rs/crc)
[![License](https://img.shields.io/crates/l/crc.svg?maxAge=2592000)](https://github.com/mrhooray/crc-rs#license)

Rust implementation of CRC(8, 16, 32, 64). MSRV is 1.46.

## Usage
Add `crc` to `Cargo.toml`
```toml
[dependencies]
crc = "2.0"
```

### Compute CRC

```rust
use crc::{Crc, Algorithm, CRC_16_IBM_SDLC, CRC_32_ISCSI};

pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
pub const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

assert_eq!(X25.checksum(b"123456789"), 0x906e);
assert_eq!(CASTAGNOLI.checksum(b"123456789"), 0xe3069283);

// use custom algorithm
const CUSTOM_ALG: Algorithm<u16> = Algorithm {
    width: 16,
    poly: 0x8005,
    init: 0xffff,
    refin: false,
    refout: false,
    xorout: 0x0000,
    check: 0xaee7,
    residue: 0x0000
};
let crc = Crc::<u16>::new(&CUSTOM_ALG);
let mut digest = crc.digest();
digest.update(b"123456789");
assert_eq!(digest.finalize(), 0xaee7);
```

## Benchmark

`cargo bench` with 2.6 GHz Intel Core i7. [Comparison](http://create.stephan-brumme.com/crc32/)
```
crc16          time:   [2.0082 ms 2.0206 ms 2.0367 ms]
               thrpt:  [468.25 MiB/s 471.96 MiB/s 474.89 MiB/s]

crc32          time:   [1.7659 ms 1.7793 ms 1.7952 ms]
               thrpt:  [531.25 MiB/s 535.98 MiB/s 540.05 MiB/s]

crc64          time:   [2.0655 ms 2.0803 ms 2.0973 ms]
               thrpt:  [454.71 MiB/s 458.43 MiB/s 461.72 MiB/s]
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
