# Contributing to `libc`

Welcome! If you are reading this document, it means you are interested in contributing
to the `libc` crate.

## Adding an API

Want to use an API which currently isn't bound in `libc`? It's quite easy to add
one!

The internal structure of this crate is designed to minimize the number of
`#[cfg]` attributes in order to easily be able to add new items which apply
to all platforms in the future. As a result, the crate is organized
hierarchically based on platform. Each module has a number of `#[cfg]`'d
children, but only one is ever actually compiled. Each module then reexports all
the contents of its children.

This means that for each platform that libc supports, the path from a
leaf module to the root will contain all bindings for the platform in question.
Consequently, this indicates where an API should be added! Adding an API at a
particular level in the hierarchy means that it is supported on all the child
platforms of that level. For example, when adding a Unix API it should be added
to `src/unix/mod.rs`, but when adding a Linux-only API it should be added to
`src/unix/linux_like/linux/mod.rs`.

If you're not 100% sure at what level of the hierarchy an API should be added
at, fear not! This crate has CI support which tests any binding against all
platforms supported, so you'll see failures if an API is added at the wrong
level or has different signatures across platforms.

New symbol(s) (i.e. functions, constants etc.) should also be added to the
symbols list(s) found in the `libc-test/semver` directory. These lists keep
track of what symbols are public in the libc crate and ensures they remain
available between changes to the crate. If the new symbol(s) are available on
all supported Unixes it should be added to `unix.txt` list<sup>1</sup>,
otherwise they should be added to the OS specific list(s).

With that in mind, the steps for adding a new API are:

1. Determine where in the module hierarchy your API should be added.
2. Add the API, including adding new symbol(s) to the semver lists.
3. Send a PR to this repo.
4. Wait for CI to pass, fixing errors.
5. Wait for a merge!

<sup>1</sup>: Note that this list has nothing to do with any Unix or Posix
standard, it's just a list shared between all OSs that declare `#[cfg(unix)]`.

## Test before you commit

We have two automated tests running on [GitHub Actions](https://github.com/rust-lang/libc/actions):

1. [`libc-test`](https://github.com/gnzlbg/ctest)
  - `cd libc-test && cargo test`
  - Use the `skip_*()` functions in `build.rs` if you really need a workaround.
2. Style checker
  - `rustc ci/style.rs && ./style src`

## Breaking change policy

Sometimes an upstream adds a breaking change to their API e.g. removing outdated items,
changing the type signature, etc. And we probably should follow that change to build the
`libc` crate successfully. It's annoying to do the equivalent of semver-major versioning
for each such change. Instead, we mark the item as deprecated and do the actual change
after a certain period. The steps are:

1. Add `#[deprecated(since = "", note="")]` attribute to the item.
  - The `since` field should have a next version of `libc`
    (e.g., if the current version is `0.2.1`, it should be `0.2.2`).
  - The `note` field should have a reason to deprecate and a tracking issue to call for comments
    (e.g., "We consider removing this as the upstream removed it.
    If you're using it, please comment on #XXX").
2. If we don't see any concerns for a while, do the change actually.

## Supported target policy

When Rust removes a support for a target, the libc crate also may remove the support anytime.

## Releasing your change to crates.io

Now that you've done the amazing job of landing your new API or your new
platform in this crate, the next step is to get that sweet, sweet usage from
crates.io! The only next step is to bump the version of libc and then publish
it. If you'd like to get a release out ASAP you can follow these steps:

1. Increment the patch version number in `Cargo.toml` and `libc-test/Cargo.toml`.
1. Send a PR to this repository. It should [look like this][example-pr], but it'd
   also be nice to fill out the description with a small rationale for the
   release (any rationale is ok though!).
1. Once merged, the release will be tagged and published by one of the libc crate
   maintainers.

[example-pr]: https://github.com/rust-lang/libc/pull/2120
