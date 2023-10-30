# 0.1.9 (June 4, 2022)

- deps: upgrade `tokio-util` dependency to `0.7.x` ([#3762])
- stream: add `StreamExt::map_while` ([#4351])
- stream: add `StreamExt::then` ([#4355])
- stream: add cancel-safety docs to `StreamExt::next` and `try_next` ([#4715])
- stream: expose `Elapsed` error ([#4502])
- stream: expose `Timeout` ([#4601])
- stream: implement `Extend` for `StreamMap` ([#4272])
- sync: add `Clone` to `RecvError` types ([#4560])

[#3762]: https://github.com/tokio-rs/tokio/pull/3762
[#4272]: https://github.com/tokio-rs/tokio/pull/4272
[#4351]: https://github.com/tokio-rs/tokio/pull/4351
[#4355]: https://github.com/tokio-rs/tokio/pull/4355
[#4502]: https://github.com/tokio-rs/tokio/pull/4502
[#4560]: https://github.com/tokio-rs/tokio/pull/4560
[#4601]: https://github.com/tokio-rs/tokio/pull/4601
[#4715]: https://github.com/tokio-rs/tokio/pull/4715

# 0.1.8 (October 29, 2021)

- stream: add `From<Receiver<T>>` impl for receiver streams ([#4080])
- stream: impl `FromIterator` for `StreamMap` ([#4052])
- signal: make windows docs for signal module show up on unix builds ([#3770])

[#3770]: https://github.com/tokio-rs/tokio/pull/3770
[#4052]: https://github.com/tokio-rs/tokio/pull/4052
[#4080]: https://github.com/tokio-rs/tokio/pull/4080

# 0.1.7 (July 7, 2021)

### Fixed

- sync: fix watch wrapper ([#3914])
- time: fix `Timeout::size_hint` ([#3902])

[#3902]: https://github.com/tokio-rs/tokio/pull/3902
[#3914]: https://github.com/tokio-rs/tokio/pull/3914

# 0.1.6 (May 14, 2021)

### Added

- stream: implement `Error` and `Display` for `BroadcastStreamRecvError` ([#3745])

### Fixed

- stream: avoid yielding in `AllFuture` and `AnyFuture` ([#3625])

[#3745]: https://github.com/tokio-rs/tokio/pull/3745
[#3625]: https://github.com/tokio-rs/tokio/pull/3625

# 0.1.5 (March 20, 2021)

### Fixed

- stream: documentation note for throttle `Unpin` ([#3600])

[#3600]: https://github.com/tokio-rs/tokio/pull/3600

# 0.1.4 (March 9, 2021)

Added

- signal: add `Signal` wrapper ([#3510])

Fixed

- stream: remove duplicate `doc_cfg` declaration ([#3561])
- sync: yield initial value in `WatchStream` ([#3576])

[#3510]: https://github.com/tokio-rs/tokio/pull/3510
[#3561]: https://github.com/tokio-rs/tokio/pull/3561
[#3576]: https://github.com/tokio-rs/tokio/pull/3576

# 0.1.3 (February 5, 2021)

Added

 - sync: add wrapper for broadcast and watch ([#3384], [#3504])

[#3384]: https://github.com/tokio-rs/tokio/pull/3384
[#3504]: https://github.com/tokio-rs/tokio/pull/3504

# 0.1.2 (January 12, 2021)

Fixed

 - docs: fix some wrappers missing in documentation ([#3378])

[#3378]: https://github.com/tokio-rs/tokio/pull/3378

# 0.1.1 (January 4, 2021)

Added

 - add `Stream` wrappers ([#3343])

Fixed

 - move `async-stream` to `dev-dependencies` ([#3366])

[#3366]: https://github.com/tokio-rs/tokio/pull/3366
[#3343]: https://github.com/tokio-rs/tokio/pull/3343

# 0.1.0 (December 23, 2020)

 - Initial release
