/*!
This library provides heavily optimized routines for string search primitives.

# Overview

This section gives a brief high level overview of what this crate offers.

* The top-level module provides routines for searching for 1, 2 or 3 bytes
  in the forward or reverse direction. When searching for more than one byte,
  positions are considered a match if the byte at that position matches any
  of the bytes.
* The [`memmem`] sub-module provides forward and reverse substring search
  routines.

In all such cases, routines operate on `&[u8]` without regard to encoding. This
is exactly what you want when searching either UTF-8 or arbitrary bytes.

# Example: using `memchr`

This example shows how to use `memchr` to find the first occurrence of `z` in
a haystack:

```
use memchr::memchr;

let haystack = b"foo bar baz quuz";
assert_eq!(Some(10), memchr(b'z', haystack));
```

# Example: matching one of three possible bytes

This examples shows how to use `memrchr3` to find occurrences of `a`, `b` or
`c`, starting at the end of the haystack.

```
use memchr::memchr3_iter;

let haystack = b"xyzaxyzbxyzc";

let mut it = memchr3_iter(b'a', b'b', b'c', haystack).rev();
assert_eq!(Some(11), it.next());
assert_eq!(Some(7), it.next());
assert_eq!(Some(3), it.next());
assert_eq!(None, it.next());
```

# Example: iterating over substring matches

This example shows how to use the [`memmem`] sub-module to find occurrences of
a substring in a haystack.

```
use memchr::memmem;

let haystack = b"foo bar foo baz foo";

let mut it = memmem::find_iter(haystack, "foo");
assert_eq!(Some(0), it.next());
assert_eq!(Some(8), it.next());
assert_eq!(Some(16), it.next());
assert_eq!(None, it.next());
```

# Example: repeating a search for the same needle

It may be possible for the overhead of constructing a substring searcher to be
measurable in some workloads. In cases where the same needle is used to search
many haystacks, it is possible to do construction once and thus to avoid it for
subsequent searches. This can be done with a [`memmem::Finder`]:

```
use memchr::memmem;

let finder = memmem::Finder::new("foo");

assert_eq!(Some(4), finder.find(b"baz foo quux"));
assert_eq!(None, finder.find(b"quux baz bar"));
```

# Why use this crate?

At first glance, the APIs provided by this crate might seem weird. Why provide
a dedicated routine like `memchr` for something that could be implemented
clearly and trivially in one line:

```
fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}
```

Or similarly, why does this crate provide substring search routines when Rust's
core library already provides them?

```
fn search(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}
```

The primary reason for both of them to exist is performance. When it comes to
performance, at a high level at least, there are two primary ways to look at
it:

* **Throughput**: For this, think about it as, "given some very large haystack
  and a byte that never occurs in that haystack, how long does it take to
  search through it and determine that it, in fact, does not occur?"
* **Latency**: For this, think about it as, "given a tiny haystack---just a
  few bytes---how long does it take to determine if a byte is in it?"

The `memchr` routine in this crate has _slightly_ worse latency than the
solution presented above, however, its throughput can easily be over an
order of magnitude faster. This is a good general purpose trade off to make.
You rarely lose, but often gain big.

**NOTE:** The name `memchr` comes from the corresponding routine in libc. A key
advantage of using this library is that its performance is not tied to its
quality of implementation in the libc you happen to be using, which can vary
greatly from platform to platform.

But what about substring search? This one is a bit more complicated. The
primary reason for its existence is still indeed performance, but it's also
useful because Rust's core library doesn't actually expose any substring
search routine on arbitrary bytes. The only substring search routine that
exists works exclusively on valid UTF-8.

So if you have valid UTF-8, is there a reason to use this over the standard
library substring search routine? Yes. This routine is faster on almost every
metric, including latency. The natural question then, is why isn't this
implementation in the standard library, even if only for searching on UTF-8?
The reason is that the implementation details for using SIMD in the standard
library haven't quite been worked out yet.

**NOTE:** Currently, only `x86_64` targets have highly accelerated
implementations of substring search. For `memchr`, all targets have
somewhat-accelerated implementations, while only `x86_64` targets have highly
accelerated implementations. This limitation is expected to be lifted once the
standard library exposes a platform independent SIMD API.

# Crate features

* **std** - When enabled (the default), this will permit this crate to use
  features specific to the standard library. Currently, the only thing used
  from the standard library is runtime SIMD CPU feature detection. This means
  that this feature must be enabled to get AVX accelerated routines. When
  `std` is not enabled, this crate will still attempt to use SSE2 accelerated
  routines on `x86_64`.
* **libc** - When enabled (**not** the default), this library will use your
  platform's libc implementation of `memchr` (and `memrchr` on Linux). This
  can be useful on non-`x86_64` targets where the fallback implementation in
  this crate is not as good as the one found in your libc. All other routines
  (e.g., `memchr[23]` and substring search) unconditionally use the
  implementation in this crate.
*/

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
// It's not worth trying to gate all code on just miri, so turn off relevant
// dead code warnings.
#![cfg_attr(miri, allow(dead_code, unused_macros))]

// Supporting 8-bit (or others) would be fine. If you need it, please submit a
// bug report at https://github.com/BurntSushi/memchr
#[cfg(not(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
compile_error!("memchr currently not supported on non-{16,32,64}");

pub use crate::memchr::{
    memchr, memchr2, memchr2_iter, memchr3, memchr3_iter, memchr_iter,
    memrchr, memrchr2, memrchr2_iter, memrchr3, memrchr3_iter, memrchr_iter,
    Memchr, Memchr2, Memchr3,
};

mod cow;
mod memchr;
pub mod memmem;
#[cfg(test)]
mod tests;
