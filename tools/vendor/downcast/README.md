# downcast &emsp; ![Latest Version]

[Latest Version]: https://img.shields.io/crates/v/downcast.svg

A trait (& utilities) for downcasting trait objects back to their original types.

## [link to API documentation](https://docs.rs/downcast)

## example usage

Add to your Cargo.toml:

```toml
[dependencies]
downcast = "0.12"
```

Add to your crate root:

```rust
#[macro_use]
extern crate downcast;
```

* [simple](examples/simple.rs) showcases the most simple usage of this library.
* [with_params](examples/with_params.rs)  showcases how to deal with traits who have type parameters. 
* [sync_service](examples/sync_service.rs)  showcases how to downcast `Arc`-pointers.

## build features

* **std (default)** enables all functionality requiring the standard library (`Downcast::downcast()`).
* **nightly** enables all functionality requiring rust nightly (`Any::type_name()`).

## faq

__Q: I'm getting `the size for values of type XXX cannot be known at compile time` errors, what am i doing wrong?__

A: Make sure you use the corresponding `Any` bound along with the `Downcast` traits. So, `Any` for `Downcast` and `AnySync` for `DowncastSync`.

__Q: Can i cast trait objects to trait objects?__

A: No, that is currently no possible in safe rust - and unsafe solutions are very tricky, as well. If you found a solution, feel free to share it!

__Q: What is the difference between this and the `downcast-rs` crate on crates.io?__

A: At the moment, there isn't one, really.
There was an unfortunate naming clash. You may consider using the other crate, as it is more actively maintained.
This one is considered feature-complete and frozen in functionality.
Hopefully, one day, the Rust language will make downcasting easier and we will need neither of these crates anymore!
