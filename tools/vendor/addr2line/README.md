# addr2line

[![](https://img.shields.io/crates/v/addr2line.svg)](https://crates.io/crates/addr2line)
[![](https://img.shields.io/docsrs/addr2line.svg)](https://docs.rs/addr2line)
[![Coverage Status](https://coveralls.io/repos/github/gimli-rs/addr2line/badge.svg?branch=master)](https://coveralls.io/github/gimli-rs/addr2line?branch=master)

A cross-platform library for retrieving per-address debug information
from files with DWARF debug information.

`addr2line` uses [`gimli`](https://github.com/gimli-rs/gimli) to parse
the debug information, and exposes an interface for finding
the source file, line number, and wrapping function for instruction
addresses within the target program. These lookups can either be
performed programmatically through `Context::find_location` and
`Context::find_frames`, or via the included example binary,
`addr2line` (named and modelled after the equivalent utility from
[GNU binutils](https://sourceware.org/binutils/docs/binutils/addr2line.html)).

# Quickstart
 - Add the [`addr2line` crate](https://crates.io/crates/addr2line) to your `Cargo.toml`
 - Load the file and parse it with [`addr2line::object::read::File::parse`](https://docs.rs/object/*/object/read/struct.File.html#method.parse)
 - Pass the parsed file to [`addr2line::Context::new` ](https://docs.rs/addr2line/*/addr2line/struct.Context.html#method.new)
 - Use [`addr2line::Context::find_location`](https://docs.rs/addr2line/*/addr2line/struct.Context.html#method.find_location)
   or [`addr2line::Context::find_frames`](https://docs.rs/addr2line/*/addr2line/struct.Context.html#method.find_frames)
   to look up debug information for an address

# Performance

`addr2line` optimizes for speed over memory by caching parsed information.
The DWARF information is parsed lazily where possible.

The library aims to perform similarly to equivalent existing tools such
as `addr2line` from binutils, `eu-addr2line` from elfutils, and
`llvm-symbolize` from the llvm project, and in the past some benchmarking
was done that indicates a comparable performance.

## License

Licensed under either of

  * Apache License, Version 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([`LICENSE-MIT`](./LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
