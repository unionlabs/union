# cache-padded

[![Build](https://github.com/smol-rs/cache-padded/workflows/Build%20and%20test/badge.svg)](
https://github.com/smol-rs/cache-padded/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](
https://github.com/smol-rs/cache-padded)
[![Cargo](https://img.shields.io/crates/v/cache-padded.svg)](
https://crates.io/crates/cache-padded)
[![Documentation](https://docs.rs/cache-padded/badge.svg)](
https://docs.rs/cache-padded)

Prevent false sharing by padding and aligning to the length of a cache line.

In concurrent programming, sometimes it is desirable to make sure commonly accessed shared data
is not all placed into the same cache line. Updating an atomic value invalides the whole cache
line it belongs to, which makes the next access to the same cache line slower for other CPU
cores. Use `CachePadded` to ensure updating one piece of data doesn't invalidate other cached
data.

## Size and alignment

Cache lines are assumed to be N bytes long, depending on the architecture:

* On x86-64 and aarch64, N = 128.
* On all others, N = 64.

Note that N is just a reasonable guess and is not guaranteed to match the actual cache line
length of the machine the program is running on.

The size of `CachePadded<T>` is the smallest multiple of N bytes large enough to accommodate
a value of type `T`.

The alignment of `CachePadded<T>` is the maximum of N bytes and the alignment of `T`.

## Examples

Alignment and padding:

```rust
use cache_padded::CachePadded;

let array = [CachePadded::new(1i8), CachePadded::new(2i8)];
let addr1 = &*array[0] as *const i8 as usize;
let addr2 = &*array[1] as *const i8 as usize;

assert!(addr2 - addr1 >= 64);
assert_eq!(addr1 % 64, 0);
assert_eq!(addr2 % 64, 0);
```

When building a concurrent queue with a head and a tail index, it is wise to place indices in
different cache lines so that concurrent threads pushing and popping elements don't invalidate
each other's cache lines:

```rust
use cache_padded::CachePadded;
use std::sync::atomic::AtomicUsize;

struct Queue<T> {
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    buffer: *mut T,
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
