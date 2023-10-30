# Contributing to `gimli`

Hi! We'd love to have your contributions! If you want help or mentorship, reach
out to us in a GitHub issue, or ping `fitzgen` in `#rust` on `irc.mozilla.org`.

* [Code of Conduct](#coc)
* [Filing an Issue](#issues)
* [Building `gimli`](#building)
* [Testing `gimli`](#testing)
  * [Test Coverage](#coverage)
  * [Using `test-assembler`](#test-assembler)
  * [Fuzzing](#fuzzing)
* [Benchmarking](#benchmarking)
* [Style](#style)

## <a id="coc"></a> Code of Conduct

We abide by the
[Rust Code of Conduct](https://www.rust-lang.org/en-US/conduct.html) and ask
that you do as well.

## <a id="issues"></a> Filing an Issue

Think you've found a bug? File an issue! To help us understand and reproduce the
issue, provide us with:

* The (preferably minimal) test case
* Steps to reproduce the issue using the test case
* The expected result of following those steps
* The actual result of following those steps

Definitely file an issue if you see an unexpected panic originating from within
`gimli`! `gimli` should never panic unless it is explicitly documented to panic
in the specific circumstances provided.

## <a id="building"></a> Building `gimli`

`gimli` should always build on stable `rustc`, but we recommend using
[`rustup`](https://www.rustup.rs/) so you can switch to nightly `rustc` and run
benchmarks.

To build `gimli`:

```
$ cargo build
```

## <a id="testing"></a> Testing `gimli`

Run the tests with `cargo`:

```
$ cargo test
```

### <a id="coverage"></a> Test Coverage

If you have `kcov` installed under linux, then you can generate code coverage
results using the `coverage` script in the root of the repository, and view them
at `target/kcov/index.html`. Otherwise you can create a pull request and view
the coverage results on coveralls.io.

```
$ ./coverage
```

The ideal we aim to reach is having our unit tests exercise every branch in
`gimli`. We allow an exception for branches which propagate errors inside a
`try!(..)` invocation, but we *do* want to exercise the original error paths.

Pull requests adding new code should ensure that this ideal is met.

At the time of writing we have 94% test coverage according to our coveralls.io
continuous integration. That number should generally stay the same or go up ;)
This is a bit subjective, because -.001% is just noise and doesn't matter.

### <a id="test-assembler"></a> Using `test-assembler`

We use the awesome
[`test-assembler`](https://github.com/luser/rust-test-assembler) crate to
construct binary test data. It makes building complex test cases readable.

[Here is an example usage in `gimli`](https://github.com/gimli-rs/gimli/blob/156451f3fe6eeb2fa62b84b362c33fcb176e1171/src/loc.rs#L263)

### <a id="fuzzing"></a> Fuzzing

First, install `cargo fuzz`:

```
$ cargo install cargo-fuzz
```

Optionally, [set up the corpora for our fuzz targets by following these
instructions](https://github.com/gimli-rs/gimli-libfuzzer-corpora/blob/master/README.md#using-these-corpora).

Finally, run a fuzz target! In this case, we are running the `eh_frame` fuzz
target:

```
$ cargo fuzz run eh_frame
```

The fuzz target definitions live in `fuzz/fuzz_targets/*`. You can add new ones
via `cargo fuzz add <my_new_target>`.

## <a id="benchmarking"></a> Benchmarking

The benchmarks require nightly `rustc`, so use `rustup`:

```
$ rustup run nightly cargo bench
```

We aim to be the fastest DWARF library. Period.

Please provide before and after benchmark results with your pull requests. You
may also find [`cargo benchcmp`](https://github.com/BurntSushi/cargo-benchcmp)
handy for comparing results.

Pull requests adding `#[bench]` micro-benchmarks that exercise a new edge case
are very welcome!

## <a id="style"></a> Style

We use `rustfmt` to automatically format and style all of our code.

To install `rustfmt`:

```
$ rustup component add rustfmt-preview
```

To run `rustfmt` on `gimli`:

```
$ cargo fmt
```
