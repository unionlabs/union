# 1.20.1 (July 25, 2022)

### Fixed

- chore: fix version detection in build script ([#4860])

[#4860]: https://github.com/tokio-rs/tokio/pull/4860

# 1.20.0 (July 12, 2022)

### Added
- tokio: add `track_caller` to public APIs ([#4772], [#4791], [#4793], [#4806], [#4808])
- sync: Add `has_changed` method to `watch::Ref` ([#4758])

### Changed

- time: remove `src/time/driver/wheel/stack.rs` ([#4766])
- rt: clean up arguments passed to basic scheduler ([#4767])
- net: be more specific about winapi features ([#4764])
- tokio: use const initialized thread locals where possible ([#4677])
- task: various small improvements to LocalKey ([#4795])

### Documented

- fs: warn about performance pitfall ([#4762])
- chore: fix spelling ([#4769])
- sync: document spurious failures in oneshot ([#4777])
- sync: add warning for watch in non-Send futures ([#4741])
- chore: fix typo ([#4798])

### Unstable

- joinset: rename `join_one` to `join_next` ([#4755])
- rt: unhandled panic config for current thread rt ([#4770])

[#4677]: https://github.com/tokio-rs/tokio/pull/4677
[#4741]: https://github.com/tokio-rs/tokio/pull/4741
[#4755]: https://github.com/tokio-rs/tokio/pull/4755
[#4758]: https://github.com/tokio-rs/tokio/pull/4758
[#4762]: https://github.com/tokio-rs/tokio/pull/4762
[#4764]: https://github.com/tokio-rs/tokio/pull/4764
[#4766]: https://github.com/tokio-rs/tokio/pull/4766
[#4767]: https://github.com/tokio-rs/tokio/pull/4767
[#4769]: https://github.com/tokio-rs/tokio/pull/4769
[#4770]: https://github.com/tokio-rs/tokio/pull/4770
[#4772]: https://github.com/tokio-rs/tokio/pull/4772
[#4777]: https://github.com/tokio-rs/tokio/pull/4777
[#4791]: https://github.com/tokio-rs/tokio/pull/4791
[#4793]: https://github.com/tokio-rs/tokio/pull/4793
[#4795]: https://github.com/tokio-rs/tokio/pull/4795
[#4798]: https://github.com/tokio-rs/tokio/pull/4798
[#4806]: https://github.com/tokio-rs/tokio/pull/4806
[#4808]: https://github.com/tokio-rs/tokio/pull/4808

# 1.19.2 (June 6, 2022)

This release fixes another bug in `Notified::enable`. ([#4751])

[#4751]: https://github.com/tokio-rs/tokio/pull/4751

# 1.19.1 (June 5, 2022)

This release fixes a bug in `Notified::enable`. ([#4747])

[#4747]: https://github.com/tokio-rs/tokio/pull/4747

# 1.19.0 (June 3, 2022)

### Added

- runtime: add `is_finished` method for `JoinHandle` and `AbortHandle` ([#4709])
- runtime: make global queue and event polling intervals configurable ([#4671])
- sync: add `Notified::enable` ([#4705])
- sync: add `watch::Sender::send_if_modified` ([#4591])
- sync: add resubscribe method to broadcast::Receiver ([#4607])
- net: add `take_error` to `TcpSocket` and `TcpStream` ([#4739])

### Changed

- io: refactor out usage of Weak in the io handle ([#4656])

### Fixed

- macros: avoid starvation in `join!` and `try_join!` ([#4624])

### Documented

- runtime: clarify semantics of tasks outliving `block_on` ([#4729])
- time: fix example for `MissedTickBehavior::Burst` ([#4713])

### Unstable

- metrics: correctly update atomics in `IoDriverMetrics` ([#4725])
- metrics: fix compilation with unstable, process, and rt, but without net ([#4682])
- task: add `#[track_caller]` to `JoinSet`/`JoinMap` ([#4697])
- task: add `Builder::{spawn_on, spawn_local_on, spawn_blocking_on}` ([#4683])
- task: add `consume_budget` for cooperative scheduling ([#4498])
- task: add `join_set::Builder` for configuring `JoinSet` tasks ([#4687])
- task: update return value of `JoinSet::join_one` ([#4726])

[#4498]: https://github.com/tokio-rs/tokio/pull/4498
[#4591]: https://github.com/tokio-rs/tokio/pull/4591
[#4607]: https://github.com/tokio-rs/tokio/pull/4607
[#4624]: https://github.com/tokio-rs/tokio/pull/4624
[#4656]: https://github.com/tokio-rs/tokio/pull/4656
[#4671]: https://github.com/tokio-rs/tokio/pull/4671
[#4682]: https://github.com/tokio-rs/tokio/pull/4682
[#4683]: https://github.com/tokio-rs/tokio/pull/4683
[#4687]: https://github.com/tokio-rs/tokio/pull/4687
[#4697]: https://github.com/tokio-rs/tokio/pull/4697
[#4705]: https://github.com/tokio-rs/tokio/pull/4705
[#4709]: https://github.com/tokio-rs/tokio/pull/4709
[#4713]: https://github.com/tokio-rs/tokio/pull/4713
[#4725]: https://github.com/tokio-rs/tokio/pull/4725
[#4726]: https://github.com/tokio-rs/tokio/pull/4726
[#4729]: https://github.com/tokio-rs/tokio/pull/4729
[#4739]: https://github.com/tokio-rs/tokio/pull/4739

# 1.18.2 (May 5, 2022)

Add missing features for the `winapi` dependency. ([#4663])

[#4663]: https://github.com/tokio-rs/tokio/pull/4663

# 1.18.1 (May 2, 2022)

The 1.18.0 release broke the build for targets without 64-bit atomics when
building with `tokio_unstable`. This release fixes that. ([#4649])

[#4649]: https://github.com/tokio-rs/tokio/pull/4649

# 1.18.0 (April 27, 2022)

This release adds a number of new APIs in `tokio::net`, `tokio::signal`, and
`tokio::sync`. In addition, it adds new unstable APIs to `tokio::task` (`Id`s
for uniquely identifying a task, and `AbortHandle` for remotely cancelling a
task), as well as a number of bugfixes.

### Fixed

- blocking: add missing `#[track_caller]` for `spawn_blocking` ([#4616])
- macros: fix `select` macro to process 64 branches ([#4519])
- net: fix `try_io` methods not calling Mio's `try_io` internally ([#4582])
- runtime: recover when OS fails to spawn a new thread ([#4485])

### Added

- net: add `UdpSocket::peer_addr` ([#4611])
- net: add `try_read_buf` method for named pipes ([#4626])
- signal: add `SignalKind` `Hash`/`Eq` impls and `c_int` conversion ([#4540])
- signal: add support for signals up to `SIGRTMAX` ([#4555])
- sync: add `watch::Sender::send_modify` method ([#4310])
- sync: add `broadcast::Receiver::len` method ([#4542])
- sync: add `watch::Receiver::same_channel` method ([#4581])
- sync: implement `Clone` for `RecvError` types ([#4560])

### Changed

- update `mio` to 0.8.1 ([#4582])
- macros: rename `tokio::select!`'s internal `util` module ([#4543])
- runtime: use `Vec::with_capacity` when building runtime ([#4553])

### Documented

- improve docs for `tokio_unstable` ([#4524])
- runtime: include more documentation for thread_pool/worker ([#4511])
- runtime: update `Handle::current`'s docs to mention `EnterGuard` ([#4567])
- time: clarify platform specific timer resolution ([#4474])
- signal: document that `Signal::recv` is cancel-safe ([#4634])
- sync: `UnboundedReceiver` close docs ([#4548])

### Unstable

The following changes only apply when building with `--cfg tokio_unstable`:

- task: add `task::Id` type ([#4630])
- task: add `AbortHandle` type for cancelling tasks in a `JoinSet` ([#4530],
  [#4640])
- task: fix missing `doc(cfg(...))` attributes for `JoinSet` ([#4531])
- task: fix broken link in `AbortHandle` RustDoc ([#4545])
- metrics: add initial IO driver metrics ([#4507])


[#4616]: https://github.com/tokio-rs/tokio/pull/4616
[#4519]: https://github.com/tokio-rs/tokio/pull/4519
[#4582]: https://github.com/tokio-rs/tokio/pull/4582
[#4485]: https://github.com/tokio-rs/tokio/pull/4485
[#4613]: https://github.com/tokio-rs/tokio/pull/4613
[#4611]: https://github.com/tokio-rs/tokio/pull/4611
[#4626]: https://github.com/tokio-rs/tokio/pull/4626
[#4540]: https://github.com/tokio-rs/tokio/pull/4540
[#4555]: https://github.com/tokio-rs/tokio/pull/4555
[#4310]: https://github.com/tokio-rs/tokio/pull/4310
[#4542]: https://github.com/tokio-rs/tokio/pull/4542
[#4581]: https://github.com/tokio-rs/tokio/pull/4581
[#4560]: https://github.com/tokio-rs/tokio/pull/4560
[#4631]: https://github.com/tokio-rs/tokio/pull/4631
[#4582]: https://github.com/tokio-rs/tokio/pull/4582
[#4543]: https://github.com/tokio-rs/tokio/pull/4543
[#4553]: https://github.com/tokio-rs/tokio/pull/4553
[#4524]: https://github.com/tokio-rs/tokio/pull/4524
[#4511]: https://github.com/tokio-rs/tokio/pull/4511
[#4567]: https://github.com/tokio-rs/tokio/pull/4567
[#4474]: https://github.com/tokio-rs/tokio/pull/4474
[#4634]: https://github.com/tokio-rs/tokio/pull/4634
[#4548]: https://github.com/tokio-rs/tokio/pull/4548
[#4630]: https://github.com/tokio-rs/tokio/pull/4630
[#4530]: https://github.com/tokio-rs/tokio/pull/4530
[#4640]: https://github.com/tokio-rs/tokio/pull/4640
[#4531]: https://github.com/tokio-rs/tokio/pull/4531
[#4545]: https://github.com/tokio-rs/tokio/pull/4545
[#4507]: https://github.com/tokio-rs/tokio/pull/4507

# 1.17.0 (February 16, 2022)

This release updates the minimum supported Rust version (MSRV) to 1.49, the
`mio` dependency to v0.8, and the (optional) `parking_lot` dependency to v0.12.
Additionally, it contains several bug fixes, as well as internal refactoring and
performance improvements.

### Fixed

- time: prevent panicking in `sleep` with large durations ([#4495])
- time: eliminate potential panics in `Instant` arithmetic on platforms where
  `Instant::now` is not monotonic ([#4461])
- io: fix `DuplexStream` not participating in cooperative yielding ([#4478])
- rt: fix potential double panic when dropping a `JoinHandle` ([#4430])

### Changed

- update minimum supported Rust version to 1.49 ([#4457])
- update `parking_lot` dependency to v0.12.0 ([#4459])
- update `mio` dependency to v0.8 ([#4449])
- rt: remove an unnecessary lock in the blocking pool ([#4436])
- rt: remove an unnecessary enum in the basic scheduler ([#4462])
- time: use bit manipulation instead of modulo to improve performance ([#4480])
- net: use `std::future::Ready` instead of our own `Ready` future ([#4271])
- replace deprecated `atomic::spin_loop_hint` with `hint::spin_loop` ([#4491])
- fix miri failures in intrusive linked lists ([#4397]) 

### Documented

- io: add an example for `tokio::process::ChildStdin` ([#4479])

### Unstable

The following changes only apply when building with `--cfg tokio_unstable`:

- task: fix missing location information in `tracing` spans generated by
  `spawn_local` ([#4483])
- task: add `JoinSet` for managing sets of tasks ([#4335])
- metrics: fix compilation error on MIPS ([#4475])
- metrics: fix compilation error on arm32v7 ([#4453])

[#4495]: https://github.com/tokio-rs/tokio/pull/4495
[#4461]: https://github.com/tokio-rs/tokio/pull/4461
[#4478]: https://github.com/tokio-rs/tokio/pull/4478
[#4430]: https://github.com/tokio-rs/tokio/pull/4430
[#4457]: https://github.com/tokio-rs/tokio/pull/4457
[#4459]: https://github.com/tokio-rs/tokio/pull/4459
[#4449]: https://github.com/tokio-rs/tokio/pull/4449
[#4462]: https://github.com/tokio-rs/tokio/pull/4462
[#4436]: https://github.com/tokio-rs/tokio/pull/4436
[#4480]: https://github.com/tokio-rs/tokio/pull/4480
[#4271]: https://github.com/tokio-rs/tokio/pull/4271
[#4491]: https://github.com/tokio-rs/tokio/pull/4491
[#4397]: https://github.com/tokio-rs/tokio/pull/4397
[#4479]: https://github.com/tokio-rs/tokio/pull/4479
[#4483]: https://github.com/tokio-rs/tokio/pull/4483
[#4335]: https://github.com/tokio-rs/tokio/pull/4335
[#4475]: https://github.com/tokio-rs/tokio/pull/4475
[#4453]: https://github.com/tokio-rs/tokio/pull/4453

# 1.16.1 (January 28, 2022)

This release fixes a bug in [#4428] with the change [#4437].

[#4428]: https://github.com/tokio-rs/tokio/pull/4428
[#4437]: https://github.com/tokio-rs/tokio/pull/4437

# 1.16.0 (January 27, 2022)

Fixes a soundness bug in `io::Take` ([#4428]). The unsoundness is exposed when
leaking memory in the given `AsyncRead` implementation and then overwriting the
supplied buffer:

```rust
impl AsyncRead for Buggy {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>
    ) -> Poll<Result<()>> {
      let new_buf = vec![0; 5].leak();
      *buf = ReadBuf::new(new_buf);
      buf.put_slice(b"hello");
      Poll::Ready(Ok(()))
    }
}
```

Also, this release includes improvements to the multi-threaded scheduler that
can increase throughput by up to 20% in some cases ([#4383]).

### Fixed

- io: **soundness** don't expose uninitialized memory when using `io::Take` in edge case ([#4428])
- fs: ensure `File::write` results in a `write` syscall when the runtime shuts down ([#4316])
- process: drop pipe after child exits in `wait_with_output` ([#4315])
- rt: improve error message when spawning a thread fails ([#4398])
- rt: reduce false-positive thread wakups in the multi-threaded scheduler ([#4383])
- sync: don't inherit `Send` from `parking_lot::*Guard` ([#4359])

### Added

- net: `TcpSocket::linger()` and `set_linger()` ([#4324])
- net: impl `UnwindSafe` for socket types ([#4384])
- rt: impl `UnwindSafe` for `JoinHandle` ([#4418])
- sync: `watch::Receiver::has_changed()` ([#4342])
- sync: `oneshot::Receiver::blocking_recv()` ([#4334])
- sync: `RwLock` blocking operations ([#4425])

### Unstable

The following changes only apply when building with `--cfg tokio_unstable`

- rt: **breaking change** overhaul runtime metrics API ([#4373])

[#4428]: https://github.com/tokio-rs/tokio/pull/4428
[#4316]: https://github.com/tokio-rs/tokio/pull/4316
[#4315]: https://github.com/tokio-rs/tokio/pull/4315
[#4398]: https://github.com/tokio-rs/tokio/pull/4398
[#4383]: https://github.com/tokio-rs/tokio/pull/4383
[#4359]: https://github.com/tokio-rs/tokio/pull/4359
[#4324]: https://github.com/tokio-rs/tokio/pull/4324
[#4384]: https://github.com/tokio-rs/tokio/pull/4384
[#4418]: https://github.com/tokio-rs/tokio/pull/4418
[#4342]: https://github.com/tokio-rs/tokio/pull/4342
[#4334]: https://github.com/tokio-rs/tokio/pull/4334
[#4425]: https://github.com/tokio-rs/tokio/pull/4425
[#4373]: https://github.com/tokio-rs/tokio/pull/4373

# 1.15.0 (December 15, 2021)

### Fixed

- io: add cooperative yielding support to `io::empty()` ([#4300])
- time: make timeout robust against budget-depleting tasks ([#4314])

### Changed

- update minimum supported Rust version to 1.46.

### Added

- time: add `Interval::reset()` ([#4248])
- io: add explicit lifetimes to `AsyncFdReadyGuard` ([#4267])
- process: add `Command::as_std()` ([#4295])

### Added (unstable)

- tracing: instrument `tokio::sync` types ([#4302])

[#4302]: https://github.com/tokio-rs/tokio/pull/4302
[#4300]: https://github.com/tokio-rs/tokio/pull/4300
[#4295]: https://github.com/tokio-rs/tokio/pull/4295
[#4267]: https://github.com/tokio-rs/tokio/pull/4267
[#4248]: https://github.com/tokio-rs/tokio/pull/4248
[#4314]: https://github.com/tokio-rs/tokio/pull/4314

# 1.14.0 (November 15, 2021)

### Fixed

- macros: fix compiler errors when using `mut` patterns in `select!` ([#4211])
- sync: fix a data race between `oneshot::Sender::send` and awaiting a
  `oneshot::Receiver` when the oneshot has been closed ([#4226])
- sync: make `AtomicWaker` panic safe ([#3689])
- runtime: fix basic scheduler dropping tasks outside a runtime context
  ([#4213])

### Added

- stats: add `RuntimeStats::busy_duration_total` ([#4179], [#4223])

### Changed

- io: updated `copy` buffer size to match `std::io::copy` ([#4209])

### Documented

-  io: rename buffer to file in doc-test ([#4230])
-  sync: fix Notify example ([#4212])

[#4211]: https://github.com/tokio-rs/tokio/pull/4211
[#4226]: https://github.com/tokio-rs/tokio/pull/4226
[#3689]: https://github.com/tokio-rs/tokio/pull/3689
[#4213]: https://github.com/tokio-rs/tokio/pull/4213
[#4179]: https://github.com/tokio-rs/tokio/pull/4179
[#4223]: https://github.com/tokio-rs/tokio/pull/4223
[#4209]: https://github.com/tokio-rs/tokio/pull/4209
[#4230]: https://github.com/tokio-rs/tokio/pull/4230
[#4212]: https://github.com/tokio-rs/tokio/pull/4212

# 1.13.1 (November 15, 2021)

### Fixed

- sync: fix a data race between `oneshot::Sender::send` and awaiting a
  `oneshot::Receiver` when the oneshot has been closed ([#4226])

[#4226]: https://github.com/tokio-rs/tokio/pull/4226

# 1.13.0 (October 29, 2021)

### Fixed

- sync: fix `Notify` to clone the waker before locking its waiter list ([#4129])
- tokio: add riscv32 to non atomic64 architectures ([#4185])

### Added

- net: add `poll_{recv,send}_ready` methods to `udp` and `uds_datagram` ([#4131])
- net: add `try_*`, `readable`, `writable`, `ready`, and `peer_addr` methods to split halves ([#4120])
- sync: add `blocking_lock` to `Mutex` ([#4130])
- sync: add `watch::Sender::send_replace` ([#3962], [#4195])
- sync: expand `Debug` for `Mutex<T>` impl to unsized `T` ([#4134])
- tracing: instrument time::Sleep ([#4072])
- tracing: use structured location fields for spawned tasks ([#4128])

### Changed

- io: add assert in `copy_bidirectional` that `poll_write` is sensible ([#4125])
- macros: use qualified syntax when polling in `select!` ([#4192])
- runtime: handle `block_on` wakeups better ([#4157])
- task: allocate callback on heap immediately in debug mode ([#4203])
- tokio: assert platform-minimum requirements at build time ([#3797])

### Documented

- docs: conversion of doc comments to indicative mood ([#4174])
- docs: add returning on the first error example for `try_join!` ([#4133])
- docs: fixing broken links in `tokio/src/lib.rs` ([#4132])
- signal: add example with background listener ([#4171])
- sync: add more oneshot examples ([#4153])
- time: document `Interval::tick` cancel safety ([#4152])

[#3797]: https://github.com/tokio-rs/tokio/pull/3797
[#3962]: https://github.com/tokio-rs/tokio/pull/3962
[#4072]: https://github.com/tokio-rs/tokio/pull/4072
[#4120]: https://github.com/tokio-rs/tokio/pull/4120
[#4125]: https://github.com/tokio-rs/tokio/pull/4125
[#4128]: https://github.com/tokio-rs/tokio/pull/4128
[#4129]: https://github.com/tokio-rs/tokio/pull/4129
[#4130]: https://github.com/tokio-rs/tokio/pull/4130
[#4131]: https://github.com/tokio-rs/tokio/pull/4131
[#4132]: https://github.com/tokio-rs/tokio/pull/4132
[#4133]: https://github.com/tokio-rs/tokio/pull/4133
[#4134]: https://github.com/tokio-rs/tokio/pull/4134
[#4152]: https://github.com/tokio-rs/tokio/pull/4152
[#4153]: https://github.com/tokio-rs/tokio/pull/4153
[#4157]: https://github.com/tokio-rs/tokio/pull/4157
[#4171]: https://github.com/tokio-rs/tokio/pull/4171
[#4174]: https://github.com/tokio-rs/tokio/pull/4174
[#4185]: https://github.com/tokio-rs/tokio/pull/4185
[#4192]: https://github.com/tokio-rs/tokio/pull/4192
[#4195]: https://github.com/tokio-rs/tokio/pull/4195
[#4203]: https://github.com/tokio-rs/tokio/pull/4203

# 1.12.0 (September 21, 2021)

### Fixed

- mpsc: ensure `try_reserve` error is consistent with `try_send` ([#4119])
- mpsc: use `spin_loop_hint` instead of `yield_now` ([#4115])
- sync: make `SendError` field public ([#4097])

### Added

- io: add POSIX AIO on FreeBSD ([#4054])
- io: add convenience method `AsyncSeekExt::rewind` ([#4107])
- runtime: add tracing span for `block_on` futures ([#4094])
- runtime: callback when a worker parks and unparks ([#4070])
- sync: implement `try_recv` for mpsc channels ([#4113])

### Documented

- docs: clarify CPU-bound tasks on Tokio ([#4105])
- mpsc: document spurious failures on `poll_recv` ([#4117])
- mpsc: document that `PollSender` impls `Sink` ([#4110])
- task: document non-guarantees of `yield_now` ([#4091])
- time: document paused time details better ([#4061], [#4103])

[#4027]: https://github.com/tokio-rs/tokio/pull/4027
[#4054]: https://github.com/tokio-rs/tokio/pull/4054
[#4061]: https://github.com/tokio-rs/tokio/pull/4061
[#4070]: https://github.com/tokio-rs/tokio/pull/4070
[#4091]: https://github.com/tokio-rs/tokio/pull/4091
[#4094]: https://github.com/tokio-rs/tokio/pull/4094
[#4097]: https://github.com/tokio-rs/tokio/pull/4097
[#4103]: https://github.com/tokio-rs/tokio/pull/4103
[#4105]: https://github.com/tokio-rs/tokio/pull/4105
[#4107]: https://github.com/tokio-rs/tokio/pull/4107
[#4110]: https://github.com/tokio-rs/tokio/pull/4110
[#4113]: https://github.com/tokio-rs/tokio/pull/4113
[#4115]: https://github.com/tokio-rs/tokio/pull/4115
[#4117]: https://github.com/tokio-rs/tokio/pull/4117
[#4119]: https://github.com/tokio-rs/tokio/pull/4119

# 1.11.0 (August 31, 2021)

### Fixed

 - time: don't panic when Instant is not monotonic ([#4044])
 - io: fix panic in `fill_buf` by not calling `poll_fill_buf` twice ([#4084])

### Added

 - watch: add `watch::Sender::subscribe` ([#3800])
 - process: add `from_std` to `ChildStd*` ([#4045])
 - stats: initial work on runtime stats ([#4043])

### Changed

 - tracing: change span naming to new console convention ([#4042])
 - io: speed-up waking by using uninitialized array ([#4055], [#4071], [#4075])

### Documented

 - time: make Sleep examples easier to find ([#4040])

[#3800]: https://github.com/tokio-rs/tokio/pull/3800
[#4040]: https://github.com/tokio-rs/tokio/pull/4040
[#4042]: https://github.com/tokio-rs/tokio/pull/4042
[#4043]: https://github.com/tokio-rs/tokio/pull/4043
[#4044]: https://github.com/tokio-rs/tokio/pull/4044
[#4045]: https://github.com/tokio-rs/tokio/pull/4045
[#4055]: https://github.com/tokio-rs/tokio/pull/4055
[#4071]: https://github.com/tokio-rs/tokio/pull/4071
[#4075]: https://github.com/tokio-rs/tokio/pull/4075
[#4084]: https://github.com/tokio-rs/tokio/pull/4084

# 1.10.1 (August 24, 2021)

### Fixed

 - runtime: fix leak in UnownedTask ([#4063])

[#4063]: https://github.com/tokio-rs/tokio/pull/4063

# 1.10.0 (August 12, 2021)

### Added

 - io: add `(read|write)_f(32|64)[_le]` methods ([#4022])
 - io: add `fill_buf` and `consume` to `AsyncBufReadExt` ([#3991])
 - process: add `Child::raw_handle()` on windows ([#3998])

### Fixed

 - doc: fix non-doc builds with `--cfg docsrs` ([#4020])
 - io: flush eagerly in `io::copy` ([#4001])
 - runtime: a debug assert was sometimes triggered during shutdown ([#4005])
 - sync: use `spin_loop_hint` instead of `yield_now` in mpsc ([#4037])
 - tokio: the test-util feature depends on rt, sync, and time ([#4036])

### Changes

 - runtime: reorganize parts of the runtime ([#3979], [#4005])
 - signal: make windows docs for signal module show up on unix builds ([#3770])
 - task: quickly send task to heap on debug mode ([#4009])

### Documented

 - io: document cancellation safety of `AsyncBufReadExt` ([#3997])
 - sync: document when `watch::send` fails ([#4021])

[#3770]: https://github.com/tokio-rs/tokio/pull/3770
[#3979]: https://github.com/tokio-rs/tokio/pull/3979
[#3991]: https://github.com/tokio-rs/tokio/pull/3991
[#3997]: https://github.com/tokio-rs/tokio/pull/3997
[#3998]: https://github.com/tokio-rs/tokio/pull/3998
[#4001]: https://github.com/tokio-rs/tokio/pull/4001
[#4005]: https://github.com/tokio-rs/tokio/pull/4005
[#4009]: https://github.com/tokio-rs/tokio/pull/4009
[#4020]: https://github.com/tokio-rs/tokio/pull/4020
[#4021]: https://github.com/tokio-rs/tokio/pull/4021
[#4022]: https://github.com/tokio-rs/tokio/pull/4022
[#4036]: https://github.com/tokio-rs/tokio/pull/4036
[#4037]: https://github.com/tokio-rs/tokio/pull/4037

# 1.9.0 (July 22, 2021)

### Added

 - net: allow customized I/O operations for `TcpStream` ([#3888])
 - sync: add getter for the mutex from a guard ([#3928])
 - task: expose nameable future for `TaskLocal::scope` ([#3273])

### Fixed

 - Fix leak if output of future panics on drop ([#3967])
 - Fix leak in `LocalSet` ([#3978])

### Changes

 - runtime: reorganize parts of the runtime ([#3909], [#3939], [#3950], [#3955], [#3980])
 - sync: clean up `OnceCell` ([#3945])
 - task: remove mutex in `JoinError` ([#3959])

[#3273]: https://github.com/tokio-rs/tokio/pull/3273
[#3888]: https://github.com/tokio-rs/tokio/pull/3888
[#3909]: https://github.com/tokio-rs/tokio/pull/3909
[#3928]: https://github.com/tokio-rs/tokio/pull/3928
[#3934]: https://github.com/tokio-rs/tokio/pull/3934
[#3939]: https://github.com/tokio-rs/tokio/pull/3939
[#3945]: https://github.com/tokio-rs/tokio/pull/3945
[#3950]: https://github.com/tokio-rs/tokio/pull/3950
[#3955]: https://github.com/tokio-rs/tokio/pull/3955
[#3959]: https://github.com/tokio-rs/tokio/pull/3959
[#3967]: https://github.com/tokio-rs/tokio/pull/3967
[#3978]: https://github.com/tokio-rs/tokio/pull/3978
[#3980]: https://github.com/tokio-rs/tokio/pull/3980

# 1.8.3 (July 26, 2021)

This release backports two fixes from 1.9.0

### Fixed

 - Fix leak if output of future panics on drop ([#3967])
 - Fix leak in `LocalSet` ([#3978])

[#3967]: https://github.com/tokio-rs/tokio/pull/3967
[#3978]: https://github.com/tokio-rs/tokio/pull/3978

# 1.8.2 (July 19, 2021)

Fixes a missed edge case from 1.8.1.

### Fixed

- runtime: drop canceled future on next poll (#3965)

# 1.8.1 (July 6, 2021)

Forward ports 1.5.1 fixes.

### Fixed

- runtime: remotely abort tasks on `JoinHandle::abort` ([#3934])

[#3934]: https://github.com/tokio-rs/tokio/pull/3934

# 1.8.0 (July 2, 2021)

### Added

- io: add `get_{ref,mut}` methods to `AsyncFdReadyGuard` and `AsyncFdReadyMutGuard` ([#3807])
- io: efficient implementation of vectored writes for `BufWriter` ([#3163])
- net: add ready/try methods to `NamedPipe{Client,Server}` ([#3866], [#3899])
- sync: add `watch::Receiver::borrow_and_update` ([#3813])
- sync: implement `From<T>` for `OnceCell<T>` ([#3877])
- time: allow users to specify Interval behaviour when delayed ([#3721])

### Added (unstable)

- rt: add `tokio::task::Builder` ([#3881])

### Fixed

- net: handle HUP event with `UnixStream` ([#3898])

### Documented

- doc: document cancellation safety ([#3900])
- time: add wait alias to sleep ([#3897])
- time: document auto-advancing behaviour of runtime ([#3763])

[#3163]: https://github.com/tokio-rs/tokio/pull/3163
[#3721]: https://github.com/tokio-rs/tokio/pull/3721
[#3763]: https://github.com/tokio-rs/tokio/pull/3763
[#3807]: https://github.com/tokio-rs/tokio/pull/3807
[#3813]: https://github.com/tokio-rs/tokio/pull/3813
[#3866]: https://github.com/tokio-rs/tokio/pull/3866
[#3877]: https://github.com/tokio-rs/tokio/pull/3877
[#3881]: https://github.com/tokio-rs/tokio/pull/3881
[#3897]: https://github.com/tokio-rs/tokio/pull/3897
[#3898]: https://github.com/tokio-rs/tokio/pull/3898
[#3899]: https://github.com/tokio-rs/tokio/pull/3899
[#3900]: https://github.com/tokio-rs/tokio/pull/3900

# 1.7.2 (July 6, 2021)

Forward ports 1.5.1 fixes.

### Fixed

- runtime: remotely abort tasks on `JoinHandle::abort` ([#3934])

[#3934]: https://github.com/tokio-rs/tokio/pull/3934

# 1.7.1 (June 18, 2021)

### Fixed

- runtime: fix early task shutdown during runtime shutdown ([#3870])

[#3870]: https://github.com/tokio-rs/tokio/pull/3870

# 1.7.0 (June 15, 2021)

### Added

- net: add named pipes on windows ([#3760])
- net: add `TcpSocket` from `std::net::TcpStream` conversion ([#3838])
- sync: add `receiver_count` to `watch::Sender` ([#3729])
- sync: export `sync::notify::Notified` future publicly ([#3840])
- tracing: instrument task wakers ([#3836])

### Fixed

- macros: suppress `clippy::default_numeric_fallback` lint in generated code ([#3831])
- runtime: immediately drop new tasks when runtime is shut down ([#3752])
- sync: deprecate unused `mpsc::RecvError` type ([#3833])

### Documented

- io: clarify EOF condition for `AsyncReadExt::read_buf` ([#3850])
- io: clarify limits on return values of `AsyncWrite::poll_write` ([#3820])
- sync: add examples to Semaphore ([#3808])

[#3729]: https://github.com/tokio-rs/tokio/pull/3729
[#3752]: https://github.com/tokio-rs/tokio/pull/3752
[#3760]: https://github.com/tokio-rs/tokio/pull/3760
[#3808]: https://github.com/tokio-rs/tokio/pull/3808
[#3820]: https://github.com/tokio-rs/tokio/pull/3820
[#3831]: https://github.com/tokio-rs/tokio/pull/3831
[#3833]: https://github.com/tokio-rs/tokio/pull/3833
[#3836]: https://github.com/tokio-rs/tokio/pull/3836
[#3838]: https://github.com/tokio-rs/tokio/pull/3838
[#3840]: https://github.com/tokio-rs/tokio/pull/3840
[#3850]: https://github.com/tokio-rs/tokio/pull/3850

# 1.6.3 (July 6, 2021)

Forward ports 1.5.1 fixes.

### Fixed

- runtime: remotely abort tasks on `JoinHandle::abort` ([#3934])

[#3934]: https://github.com/tokio-rs/tokio/pull/3934

# 1.6.2 (June 14, 2021)

### Fixes

- test: sub-ms `time:advance` regression introduced in 1.6 ([#3852])

[#3852]: https://github.com/tokio-rs/tokio/pull/3852

# 1.6.1 (May 28, 2021)

This release reverts [#3518] because it doesn't work on some kernels due to
a kernel bug. ([#3803])

[#3518]: https://github.com/tokio-rs/tokio/issues/3518
[#3803]: https://github.com/tokio-rs/tokio/issues/3803

# 1.6.0 (May 14, 2021)

### Added

- fs: try doing a non-blocking read before punting to the threadpool ([#3518])
- io: add `write_all_buf` to `AsyncWriteExt` ([#3737])
- io: implement `AsyncSeek` for `BufReader`, `BufWriter`, and `BufStream` ([#3491])
- net: support non-blocking vectored I/O ([#3761])
- sync: add `mpsc::Sender::{reserve_owned, try_reserve_owned}` ([#3704])
- sync: add a `MutexGuard::map` method that returns a `MappedMutexGuard` ([#2472])
- time: add getter for Interval's period ([#3705])

### Fixed

- io: wake pending writers on `DuplexStream` close ([#3756])
- process: avoid redundant effort to reap orphan processes ([#3743])
- signal: use `std::os::raw::c_int` instead of `libc::c_int` on public API ([#3774])
- sync: preserve permit state in `notify_waiters` ([#3660])
- task: update `JoinHandle` panic message ([#3727])
- time: prevent `time::advance` from going too far ([#3712])

### Documented

- net: hide `net::unix::datagram` module from docs ([#3775])
- process: updated example ([#3748])
- sync: `Barrier` doc should use task, not thread ([#3780])
- task: update documentation on `block_in_place` ([#3753])

[#2472]: https://github.com/tokio-rs/tokio/pull/2472
[#3491]: https://github.com/tokio-rs/tokio/pull/3491
[#3518]: https://github.com/tokio-rs/tokio/pull/3518
[#3660]: https://github.com/tokio-rs/tokio/pull/3660
[#3704]: https://github.com/tokio-rs/tokio/pull/3704
[#3705]: https://github.com/tokio-rs/tokio/pull/3705
[#3712]: https://github.com/tokio-rs/tokio/pull/3712
[#3727]: https://github.com/tokio-rs/tokio/pull/3727
[#3737]: https://github.com/tokio-rs/tokio/pull/3737
[#3743]: https://github.com/tokio-rs/tokio/pull/3743
[#3748]: https://github.com/tokio-rs/tokio/pull/3748
[#3753]: https://github.com/tokio-rs/tokio/pull/3753
[#3756]: https://github.com/tokio-rs/tokio/pull/3756
[#3761]: https://github.com/tokio-rs/tokio/pull/3761
[#3774]: https://github.com/tokio-rs/tokio/pull/3774
[#3775]: https://github.com/tokio-rs/tokio/pull/3775
[#3780]: https://github.com/tokio-rs/tokio/pull/3780

# 1.5.1 (July 6, 2021)

### Fixed

- runtime: remotely abort tasks on `JoinHandle::abort` ([#3934])

[#3934]: https://github.com/tokio-rs/tokio/pull/3934

# 1.5.0 (April 12, 2021)

### Added

- io: add `AsyncSeekExt::stream_position` ([#3650])
- io: add `AsyncWriteExt::write_vectored` ([#3678])
- io: add a `copy_bidirectional` utility ([#3572])
- net: implement `IntoRawFd` for `TcpSocket` ([#3684])
- sync: add `OnceCell` ([#3591])
- sync: add `OwnedRwLockReadGuard` and `OwnedRwLockWriteGuard` ([#3340])
- sync: add `Semaphore::is_closed` ([#3673])
- sync: add `mpsc::Sender::capacity` ([#3690])
- sync: allow configuring `RwLock` max reads ([#3644])
- task: add `sync_scope` for `LocalKey` ([#3612])

### Fixed

- chore: try to avoid `noalias` attributes on intrusive linked list ([#3654])
- rt: fix panic in `JoinHandle::abort()` when called from other threads ([#3672])
- sync: don't panic in `oneshot::try_recv` ([#3674])
- sync: fix notifications getting dropped on receiver drop ([#3652])
- sync: fix `Semaphore` permit overflow calculation ([#3644])

### Documented

- io: clarify requirements of `AsyncFd` ([#3635])
- runtime: fix unclear docs for `{Handle,Runtime}::block_on` ([#3628])
- sync: document that `Semaphore` is fair ([#3693])
- sync: improve doc on blocking mutex ([#3645])

[#3340]: https://github.com/tokio-rs/tokio/pull/3340
[#3572]: https://github.com/tokio-rs/tokio/pull/3572
[#3591]: https://github.com/tokio-rs/tokio/pull/3591
[#3612]: https://github.com/tokio-rs/tokio/pull/3612
[#3628]: https://github.com/tokio-rs/tokio/pull/3628
[#3635]: https://github.com/tokio-rs/tokio/pull/3635
[#3644]: https://github.com/tokio-rs/tokio/pull/3644
[#3645]: https://github.com/tokio-rs/tokio/pull/3645
[#3650]: https://github.com/tokio-rs/tokio/pull/3650
[#3652]: https://github.com/tokio-rs/tokio/pull/3652
[#3654]: https://github.com/tokio-rs/tokio/pull/3654
[#3672]: https://github.com/tokio-rs/tokio/pull/3672
[#3673]: https://github.com/tokio-rs/tokio/pull/3673
[#3674]: https://github.com/tokio-rs/tokio/pull/3674
[#3678]: https://github.com/tokio-rs/tokio/pull/3678
[#3684]: https://github.com/tokio-rs/tokio/pull/3684
[#3690]: https://github.com/tokio-rs/tokio/pull/3690
[#3693]: https://github.com/tokio-rs/tokio/pull/3693

# 1.4.0 (March 20, 2021)

### Added

- macros: introduce biased argument for `select!` ([#3603])
- runtime: add `Handle::block_on` ([#3569])

### Fixed

- runtime: avoid unnecessary polling of `block_on` future ([#3582])
- runtime: fix memory leak/growth when creating many runtimes ([#3564])
- runtime: mark `EnterGuard` with `must_use` ([#3609])

### Documented

- chore: mention fix for building docs in contributing guide ([#3618])
- doc: add link to `PollSender` ([#3613])
- doc: alias sleep to delay ([#3604])
- sync: improve `Mutex` FIFO explanation ([#3615])
- timer: fix double newline in module docs ([#3617])

[#3564]: https://github.com/tokio-rs/tokio/pull/3564
[#3613]: https://github.com/tokio-rs/tokio/pull/3613
[#3618]: https://github.com/tokio-rs/tokio/pull/3618
[#3617]: https://github.com/tokio-rs/tokio/pull/3617
[#3582]: https://github.com/tokio-rs/tokio/pull/3582
[#3615]: https://github.com/tokio-rs/tokio/pull/3615
[#3603]: https://github.com/tokio-rs/tokio/pull/3603
[#3609]: https://github.com/tokio-rs/tokio/pull/3609
[#3604]: https://github.com/tokio-rs/tokio/pull/3604
[#3569]: https://github.com/tokio-rs/tokio/pull/3569

# 1.3.0 (March 9, 2021)

### Added

- coop: expose an `unconstrained()` opt-out ([#3547])
- net: add `into_std` for net types without it ([#3509])
- sync: add `same_channel` method to `mpsc::Sender` ([#3532])
- sync: add `{try_,}acquire_many_owned` to `Semaphore` ([#3535])
- sync: add back `RwLockWriteGuard::map` and `RwLockWriteGuard::try_map` ([#3348])

### Fixed

- sync: allow `oneshot::Receiver::close` after successful `try_recv` ([#3552])
- time: do not panic on `timeout(Duration::MAX)` ([#3551])

### Documented

- doc: doc aliases for pre-1.0 function names ([#3523])
- io: fix typos ([#3541])
- io: note the EOF behaviour of `read_until` ([#3536])
- io: update `AsyncRead::poll_read` doc ([#3557])
- net: update `UdpSocket` splitting doc ([#3517])
- runtime: add link to `LocalSet` on `new_current_thread` ([#3508])
- runtime: update documentation of thread limits ([#3527])
- sync: do not recommend `join_all` for `Barrier` ([#3514])
- sync: documentation for `oneshot` ([#3592])
- sync: rename `notify` to `notify_one` ([#3526])
- time: fix typo in `Sleep` doc ([#3515])
- time: sync `interval.rs` and `time/mod.rs` docs ([#3533])

[#3348]: https://github.com/tokio-rs/tokio/pull/3348
[#3508]: https://github.com/tokio-rs/tokio/pull/3508
[#3509]: https://github.com/tokio-rs/tokio/pull/3509
[#3514]: https://github.com/tokio-rs/tokio/pull/3514
[#3515]: https://github.com/tokio-rs/tokio/pull/3515
[#3517]: https://github.com/tokio-rs/tokio/pull/3517
[#3523]: https://github.com/tokio-rs/tokio/pull/3523
[#3526]: https://github.com/tokio-rs/tokio/pull/3526
[#3527]: https://github.com/tokio-rs/tokio/pull/3527
[#3532]: https://github.com/tokio-rs/tokio/pull/3532
[#3533]: https://github.com/tokio-rs/tokio/pull/3533
[#3535]: https://github.com/tokio-rs/tokio/pull/3535
[#3536]: https://github.com/tokio-rs/tokio/pull/3536
[#3541]: https://github.com/tokio-rs/tokio/pull/3541
[#3547]: https://github.com/tokio-rs/tokio/pull/3547
[#3551]: https://github.com/tokio-rs/tokio/pull/3551
[#3552]: https://github.com/tokio-rs/tokio/pull/3552
[#3557]: https://github.com/tokio-rs/tokio/pull/3557
[#3592]: https://github.com/tokio-rs/tokio/pull/3592

# 1.2.0 (February 5, 2021)

### Added

- signal: make `Signal::poll_recv` method public ([#3383])

### Fixed

- time: make `test-util` paused time fully deterministic ([#3492])

### Documented

- sync: link to new broadcast and watch wrappers ([#3504])

[#3383]: https://github.com/tokio-rs/tokio/pull/3383
[#3492]: https://github.com/tokio-rs/tokio/pull/3492
[#3504]: https://github.com/tokio-rs/tokio/pull/3504

# 1.1.1 (January 29, 2021)

Forward ports 1.0.3 fix.

### Fixed
- io: memory leak during shutdown ([#3477]).

# 1.1.0 (January 22, 2021)

### Added

- net: add `try_read_buf` and `try_recv_buf` ([#3351])
- mpsc: Add `Sender::try_reserve` function ([#3418])
- sync: add `RwLock` `try_read` and `try_write` methods ([#3400])
- io: add `ReadBuf::inner_mut` ([#3443])

### Changed

- macros: improve `select!` error message ([#3352])
- io: keep track of initialized bytes in `read_to_end` ([#3426])
- runtime: consolidate errors for context missing ([#3441])

### Fixed

- task: wake `LocalSet` on `spawn_local` ([#3369])
- sync: fix panic in broadcast::Receiver drop ([#3434])

### Documented
- stream: link to new `Stream` wrappers in `tokio-stream` ([#3343])
- docs: mention that `test-util` feature is not enabled with full ([#3397])
- process: add documentation to process::Child fields ([#3437])
- io: clarify `AsyncFd` docs about changes of the inner fd ([#3430])
- net: update datagram docs on splitting ([#3448])
- time: document that `Sleep` is not `Unpin` ([#3457])
- sync: add link to `PollSemaphore` ([#3456])
- task: add `LocalSet` example ([#3438])
- sync: improve bounded `mpsc` documentation ([#3458])

[#3343]: https://github.com/tokio-rs/tokio/pull/3343
[#3351]: https://github.com/tokio-rs/tokio/pull/3351
[#3352]: https://github.com/tokio-rs/tokio/pull/3352
[#3369]: https://github.com/tokio-rs/tokio/pull/3369
[#3397]: https://github.com/tokio-rs/tokio/pull/3397
[#3400]: https://github.com/tokio-rs/tokio/pull/3400
[#3418]: https://github.com/tokio-rs/tokio/pull/3418
[#3426]: https://github.com/tokio-rs/tokio/pull/3426
[#3430]: https://github.com/tokio-rs/tokio/pull/3430
[#3434]: https://github.com/tokio-rs/tokio/pull/3434
[#3437]: https://github.com/tokio-rs/tokio/pull/3437
[#3438]: https://github.com/tokio-rs/tokio/pull/3438
[#3441]: https://github.com/tokio-rs/tokio/pull/3441
[#3443]: https://github.com/tokio-rs/tokio/pull/3443
[#3448]: https://github.com/tokio-rs/tokio/pull/3448
[#3456]: https://github.com/tokio-rs/tokio/pull/3456
[#3457]: https://github.com/tokio-rs/tokio/pull/3457
[#3458]: https://github.com/tokio-rs/tokio/pull/3458

# 1.0.3 (January 28, 2021)

### Fixed
- io: memory leak during shutdown ([#3477]).

[#3477]: https://github.com/tokio-rs/tokio/pull/3477

# 1.0.2 (January 14, 2021)

### Fixed
- io: soundness in `read_to_end` ([#3428]).

[#3428]: https://github.com/tokio-rs/tokio/pull/3428

# 1.0.1 (December 25, 2020)

This release fixes a soundness hole caused by the combination of `RwLockWriteGuard::map`
and `RwLockWriteGuard::downgrade` by removing the `map` function. This is a breaking
change, but breaking changes are allowed under our semver policy when they are required
to fix a soundness hole. (See [this RFC][semver] for more.)

Note that we have chosen not to do a deprecation cycle or similar because Tokio 1.0.0 was
released two days ago, and therefore the impact should be minimal.

Due to the soundness hole, we have also yanked Tokio version 1.0.0.

### Removed

- sync: remove `RwLockWriteGuard::map` and `RwLockWriteGuard::try_map` ([#3345])

### Fixed

- docs: remove stream feature from docs ([#3335])

[semver]: https://github.com/rust-lang/rfcs/blob/master/text/1122-language-semver.md#soundness-changes
[#3335]: https://github.com/tokio-rs/tokio/pull/3335
[#3345]: https://github.com/tokio-rs/tokio/pull/3345

# 1.0.0 (December 23, 2020)

Commit to the API and long-term support.

### Fixed

- sync: spurious wakeup in `watch` ([#3234]).

### Changed

- io: rename `AsyncFd::with_io()` to `try_io()` ([#3306])
- fs: avoid OS specific `*Ext` traits in favor of conditionally defining the fn ([#3264]).
- fs: `Sleep` is `!Unpin` ([#3278]).
- net: pass `SocketAddr` by value ([#3125]).
- net: `TcpStream::poll_peek` takes `ReadBuf` ([#3259]).
- rt: rename `runtime::Builder::max_threads()` to `max_blocking_threads()` ([#3287]).
- time: require `current_thread` runtime when calling `time::pause()` ([#3289]).

### Removed

- remove `tokio::prelude` ([#3299]).
- io: remove `AsyncFd::with_poll()` ([#3306]).
- net: remove `{Tcp,Unix}Stream::shutdown()` in favor of `AsyncWrite::shutdown()` ([#3298]).
- stream: move all stream utilities to `tokio-stream` until `Stream` is added to
  `std` ([#3277]).
- sync: mpsc `try_recv()` due to unexpected behavior ([#3263]).
- tracing: make unstable as `tracing-core` is not 1.0 yet ([#3266]).

### Added

- fs: `poll_*` fns to `DirEntry` ([#3308]).
- io: `poll_*` fns to `io::Lines`, `io::Split` ([#3308]).
- io: `_mut` method variants to `AsyncFd` ([#3304]).
- net: `poll_*` fns to `UnixDatagram` ([#3223]).
- net: `UnixStream` readiness and non-blocking ops ([#3246]).
- sync: `UnboundedReceiver::blocking_recv()` ([#3262]).
- sync: `watch::Sender::borrow()` ([#3269]).
- sync: `Semaphore::close()` ([#3065]).
- sync: `poll_recv` fns to `mpsc::Receiver`, `mpsc::UnboundedReceiver` ([#3308]).
- time: `poll_tick` fn to `time::Interval` ([#3316]).

[#3065]: https://github.com/tokio-rs/tokio/pull/3065
[#3125]: https://github.com/tokio-rs/tokio/pull/3125
[#3223]: https://github.com/tokio-rs/tokio/pull/3223
[#3234]: https://github.com/tokio-rs/tokio/pull/3234
[#3246]: https://github.com/tokio-rs/tokio/pull/3246
[#3259]: https://github.com/tokio-rs/tokio/pull/3259
[#3262]: https://github.com/tokio-rs/tokio/pull/3262
[#3263]: https://github.com/tokio-rs/tokio/pull/3263
[#3264]: https://github.com/tokio-rs/tokio/pull/3264
[#3266]: https://github.com/tokio-rs/tokio/pull/3266
[#3269]: https://github.com/tokio-rs/tokio/pull/3269
[#3277]: https://github.com/tokio-rs/tokio/pull/3277
[#3278]: https://github.com/tokio-rs/tokio/pull/3278
[#3287]: https://github.com/tokio-rs/tokio/pull/3287
[#3289]: https://github.com/tokio-rs/tokio/pull/3289
[#3298]: https://github.com/tokio-rs/tokio/pull/3298
[#3299]: https://github.com/tokio-rs/tokio/pull/3299
[#3304]: https://github.com/tokio-rs/tokio/pull/3304
[#3306]: https://github.com/tokio-rs/tokio/pull/3306
[#3308]: https://github.com/tokio-rs/tokio/pull/3308
[#3316]: https://github.com/tokio-rs/tokio/pull/3316

# 0.3.6 (December 14, 2020)

### Fixed

- rt: fix deadlock in shutdown ([#3228])
- rt: fix panic in task abort when off rt ([#3159])
- sync: make `add_permits` panic with usize::MAX >> 3 permits ([#3188])
- time: Fix race condition in timer drop ([#3229])
- watch: fix spurious wakeup ([#3244])

### Added

- example: add back udp-codec example ([#3205])
- net: add `TcpStream::into_std` ([#3189])

[#3159]: https://github.com/tokio-rs/tokio/pull/3159
[#3188]: https://github.com/tokio-rs/tokio/pull/3188
[#3189]: https://github.com/tokio-rs/tokio/pull/3189
[#3205]: https://github.com/tokio-rs/tokio/pull/3205
[#3228]: https://github.com/tokio-rs/tokio/pull/3228
[#3229]: https://github.com/tokio-rs/tokio/pull/3229
[#3244]: https://github.com/tokio-rs/tokio/pull/3244

# 0.3.5 (November 30, 2020)

### Fixed

- rt: fix `shutdown_timeout(0)` ([#3196]).
- time: fixed race condition with small sleeps ([#3069]).

### Added

- io: `AsyncFd::with_interest()` ([#3167]).
- signal: `CtrlC` stream on windows ([#3186]).

[#3069]: https://github.com/tokio-rs/tokio/pull/3069
[#3167]: https://github.com/tokio-rs/tokio/pull/3167
[#3186]: https://github.com/tokio-rs/tokio/pull/3186
[#3196]: https://github.com/tokio-rs/tokio/pull/3196

# 0.3.4 (November 18, 2020)

### Fixed

- stream: `StreamMap` `Default` impl bound ([#3093]).
- io: `AsyncFd::into_inner()` should deregister the FD ([#3104]).

### Changed

- meta: `parking_lot` feature enabled with `full` ([#3119]).

### Added

- io: `AsyncWrite` vectored writes ([#3149]).
- net: TCP/UDP readiness and non-blocking ops ([#3130], [#2743], [#3138]).
- net: TCP socket option (linger, send/recv buf size) ([#3145], [#3143]).
- net: PID field in `UCred` with solaris/illumos ([#3085]).
- rt: `runtime::Handle` allows spawning onto a runtime ([#3079]).
- sync: `Notify::notify_waiters()` ([#3098]).
- sync: `acquire_many()`, `try_acquire_many()` to `Semaphore` ([#3067]).

[#2743]: https://github.com/tokio-rs/tokio/pull/2743
[#3067]: https://github.com/tokio-rs/tokio/pull/3067
[#3079]: https://github.com/tokio-rs/tokio/pull/3079
[#3085]: https://github.com/tokio-rs/tokio/pull/3085
[#3093]: https://github.com/tokio-rs/tokio/pull/3093
[#3098]: https://github.com/tokio-rs/tokio/pull/3098
[#3104]: https://github.com/tokio-rs/tokio/pull/3104
[#3119]: https://github.com/tokio-rs/tokio/pull/3119
[#3130]: https://github.com/tokio-rs/tokio/pull/3130
[#3138]: https://github.com/tokio-rs/tokio/pull/3138
[#3143]: https://github.com/tokio-rs/tokio/pull/3143
[#3145]: https://github.com/tokio-rs/tokio/pull/3145
[#3149]: https://github.com/tokio-rs/tokio/pull/3149

# 0.3.3 (November 2, 2020)

Fixes a soundness hole by adding a missing `Send` bound to
`Runtime::spawn_blocking()`.

### Fixed

- rt: include missing `Send`, fixing soundness hole ([#3089]).
- tracing: avoid huge trace span names ([#3074]).

### Added

- net: `TcpSocket::reuseport()`, `TcpSocket::set_reuseport()` ([#3083]).
- net: `TcpSocket::reuseaddr()` ([#3093]).
- net: `TcpSocket::local_addr()` ([#3093]).
- net: add pid to `UCred` ([#2633]).

[#2633]: https://github.com/tokio-rs/tokio/pull/2633
[#3074]: https://github.com/tokio-rs/tokio/pull/3074
[#3083]: https://github.com/tokio-rs/tokio/pull/3083
[#3089]: https://github.com/tokio-rs/tokio/pull/3089
[#3093]: https://github.com/tokio-rs/tokio/pull/3093

# 0.3.2 (October 27, 2020)

Adds `AsyncFd` as a replacement for v0.2's `PollEvented`.

### Fixed

- io: fix a potential deadlock when shutting down the I/O driver ([#2903]).
- sync: `RwLockWriteGuard::downgrade()` bug ([#2957]).

### Added

- io: `AsyncFd` for receiving readiness events on raw FDs ([#2903]).
- net: `poll_*` function on `UdpSocket` ([#2981]).
- net: `UdpSocket::take_error()` ([#3051]).
- sync: `oneshot::Sender::poll_closed()` ([#3032]).

[#2903]: https://github.com/tokio-rs/tokio/pull/2903
[#2957]: https://github.com/tokio-rs/tokio/pull/2957
[#2981]: https://github.com/tokio-rs/tokio/pull/2981
[#3032]: https://github.com/tokio-rs/tokio/pull/3032
[#3051]: https://github.com/tokio-rs/tokio/pull/3051

# 0.3.1 (October 21, 2020)

This release fixes an use-after-free in the IO driver. Additionally, the `read_buf`
and `write_buf` methods have been added back to the IO traits, as the bytes crate
is now on track to reach version 1.0 together with Tokio.

### Fixed

- net: fix use-after-free ([#3019]).
- fs: ensure buffered data is written on shutdown ([#3009]).

### Added

- io: `copy_buf()` ([#2884]).
- io: `AsyncReadExt::read_buf()`, `AsyncReadExt::write_buf()` for working with
  `Buf`/`BufMut` ([#3003]).
- rt: `Runtime::spawn_blocking()` ([#2980]).
- sync: `watch::Sender::is_closed()` ([#2991]).

[#2884]: https://github.com/tokio-rs/tokio/pull/2884
[#2980]: https://github.com/tokio-rs/tokio/pull/2980
[#2991]: https://github.com/tokio-rs/tokio/pull/2991
[#3003]: https://github.com/tokio-rs/tokio/pull/3003
[#3009]: https://github.com/tokio-rs/tokio/pull/3009
[#3019]: https://github.com/tokio-rs/tokio/pull/3019

# 0.3.0 (October 15, 2020)

This represents a 1.0 beta release. APIs are polished and future-proofed. APIs
not included for 1.0 stabilization have been removed.

Biggest changes are:

- I/O driver internal rewrite. The windows implementation includes significant
  changes.
- Runtime API is polished, especially with how it interacts with feature flag
  combinations.
- Feature flags are simplified
  - `rt-core` and `rt-util` are combined to `rt`
  - `rt-threaded` is renamed to `rt-multi-thread` to match builder API
  - `tcp`, `udp`, `uds`, `dns` are combined to `net`.
  - `parking_lot` is included with `full`

### Changes

- meta: Minimum supported Rust version is now 1.45.
- io: `AsyncRead` trait now takes `ReadBuf` in order to safely handle reading
  into uninitialized memory ([#2758]).
- io: Internal I/O driver storage is now able to compact ([#2757]).
- rt: `Runtime::block_on` now takes `&self` ([#2782]).
- sync: `watch` reworked to decouple receiving a change notification from
  receiving the value ([#2814], [#2806]).
- sync: `Notify::notify` is renamed to `notify_one` ([#2822]).
- process: `Child::kill` is now an `async fn` that cleans zombies ([#2823]).
- sync: use `const fn` constructors as possible ([#2833], [#2790])
- signal: reduce cross-thread notification ([#2835]).
- net: tcp,udp,uds types support operations with `&self` ([#2828], [#2919], [#2934]).
- sync: blocking `mpsc` channel supports `send` with `&self` ([#2861]).
- time: rename `delay_for` and `delay_until` to `sleep` and `sleep_until` ([#2826]).
- io: upgrade to `mio` 0.7 ([#2893]).
- io: `AsyncSeek` trait is tweaked ([#2885]).
- fs: `File` operations take `&self` ([#2930]).
- rt: runtime API, and `#[tokio::main]` macro polish ([#2876])
- rt: `Runtime::enter` uses an RAII guard instead of a closure ([#2954]).
- net: the `from_std` function on all sockets no longer sets socket into non-blocking mode ([#2893])

### Added

- sync: `map` function to lock guards ([#2445]).
- sync: `blocking_recv` and `blocking_send` fns to `mpsc` for use outside of Tokio ([#2685]).
- rt: `Builder::thread_name_fn` for configuring thread names ([#1921]).
- fs: impl `FromRawFd` and `FromRawHandle` for `File` ([#2792]).
- process: `Child::wait` and `Child::try_wait` ([#2796]).
- rt: support configuring thread keep-alive duration ([#2809]).
- rt: `task::JoinHandle::abort` forcibly cancels a spawned task ([#2474]).
- sync: `RwLock` write guard to read guard downgrading ([#2733]).
- net: add `poll_*` functions that take `&self` to all net types ([#2845])
- sync: `get_mut()` for `Mutex`, `RwLock` ([#2856]).
- sync: `mpsc::Sender::closed()` waits for `Receiver` half to close ([#2840]).
- sync: `mpsc::Sender::is_closed()` returns true if `Receiver` half is closed ([#2726]).
- stream: `iter` and `iter_mut` to `StreamMap` ([#2890]).
- net: implement `AsRawSocket` on windows ([#2911]).
- net: `TcpSocket` creates a socket without binding or listening ([#2920]).

### Removed

- io: vectored ops are removed from `AsyncRead`, `AsyncWrite` traits ([#2882]).
- io: `mio` is removed from the public API. `PollEvented` and` Registration` are
  removed ([#2893]).
- io: remove `bytes` from public API. `Buf` and `BufMut` implementation are
  removed ([#2908]).
- time: `DelayQueue` is moved to `tokio-util` ([#2897]).

### Fixed

- io: `stdout` and `stderr` buffering on windows ([#2734]).

[#1921]: https://github.com/tokio-rs/tokio/pull/1921
[#2445]: https://github.com/tokio-rs/tokio/pull/2445
[#2474]: https://github.com/tokio-rs/tokio/pull/2474
[#2685]: https://github.com/tokio-rs/tokio/pull/2685
[#2726]: https://github.com/tokio-rs/tokio/pull/2726
[#2733]: https://github.com/tokio-rs/tokio/pull/2733
[#2734]: https://github.com/tokio-rs/tokio/pull/2734
[#2757]: https://github.com/tokio-rs/tokio/pull/2757
[#2758]: https://github.com/tokio-rs/tokio/pull/2758
[#2782]: https://github.com/tokio-rs/tokio/pull/2782
[#2790]: https://github.com/tokio-rs/tokio/pull/2790
[#2792]: https://github.com/tokio-rs/tokio/pull/2792
[#2796]: https://github.com/tokio-rs/tokio/pull/2796
[#2806]: https://github.com/tokio-rs/tokio/pull/2806
[#2809]: https://github.com/tokio-rs/tokio/pull/2809
[#2814]: https://github.com/tokio-rs/tokio/pull/2814
[#2822]: https://github.com/tokio-rs/tokio/pull/2822
[#2823]: https://github.com/tokio-rs/tokio/pull/2823
[#2826]: https://github.com/tokio-rs/tokio/pull/2826
[#2828]: https://github.com/tokio-rs/tokio/pull/2828
[#2833]: https://github.com/tokio-rs/tokio/pull/2833
[#2835]: https://github.com/tokio-rs/tokio/pull/2835
[#2840]: https://github.com/tokio-rs/tokio/pull/2840
[#2845]: https://github.com/tokio-rs/tokio/pull/2845
[#2856]: https://github.com/tokio-rs/tokio/pull/2856
[#2861]: https://github.com/tokio-rs/tokio/pull/2861
[#2876]: https://github.com/tokio-rs/tokio/pull/2876
[#2882]: https://github.com/tokio-rs/tokio/pull/2882
[#2885]: https://github.com/tokio-rs/tokio/pull/2885
[#2890]: https://github.com/tokio-rs/tokio/pull/2890
[#2893]: https://github.com/tokio-rs/tokio/pull/2893
[#2897]: https://github.com/tokio-rs/tokio/pull/2897
[#2908]: https://github.com/tokio-rs/tokio/pull/2908
[#2911]: https://github.com/tokio-rs/tokio/pull/2911
[#2919]: https://github.com/tokio-rs/tokio/pull/2919
[#2920]: https://github.com/tokio-rs/tokio/pull/2920
[#2930]: https://github.com/tokio-rs/tokio/pull/2930
[#2934]: https://github.com/tokio-rs/tokio/pull/2934
[#2954]: https://github.com/tokio-rs/tokio/pull/2954

# 0.2.22 (July 21, 2020)

### Fixes

- docs: misc improvements ([#2572], [#2658], [#2663], [#2656], [#2647], [#2630], [#2487], [#2621],
  [#2624], [#2600], [#2623], [#2622], [#2577], [#2569], [#2589], [#2575], [#2540], [#2564], [#2567],
  [#2520], [#2521], [#2493])
- rt: allow calls to `block_on` inside calls to `block_in_place` that are
  themselves inside `block_on` ([#2645])
- net: fix non-portable behavior when dropping `TcpStream` `OwnedWriteHalf` ([#2597])
- io: improve stack usage by allocating large buffers on directly on the heap
  ([#2634])
- io: fix unsound pin projection in `AsyncReadExt::read_buf` and
  `AsyncWriteExt::write_buf` ([#2612])
- io: fix unnecessary zeroing for `AsyncRead` implementors ([#2525])
- io: Fix `BufReader` not correctly forwarding `poll_write_buf` ([#2654])
- io: fix panic in `AsyncReadExt::read_line` ([#2541])

### Changes

- coop: returning `Poll::Pending` no longer decrements the task budget ([#2549])

### Added

- io: little-endian variants of `AsyncReadExt` and `AsyncWriteExt` methods
  ([#1915])
- task: add [`tracing`] instrumentation to spawned tasks ([#2655])
- sync: allow unsized types in `Mutex` and `RwLock` (via `default` constructors)
  ([#2615])
- net: add `ToSocketAddrs` implementation for `&[SocketAddr]` ([#2604])
- fs: add `OpenOptionsExt` for `OpenOptions` ([#2515])
- fs: add `DirBuilder` ([#2524])

[`tracing`]: https://crates.io/crates/tracing
[#1915]: https://github.com/tokio-rs/tokio/pull/1915
[#2487]: https://github.com/tokio-rs/tokio/pull/2487
[#2493]: https://github.com/tokio-rs/tokio/pull/2493
[#2515]: https://github.com/tokio-rs/tokio/pull/2515
[#2520]: https://github.com/tokio-rs/tokio/pull/2520
[#2521]: https://github.com/tokio-rs/tokio/pull/2521
[#2524]: https://github.com/tokio-rs/tokio/pull/2524
[#2525]: https://github.com/tokio-rs/tokio/pull/2525
[#2540]: https://github.com/tokio-rs/tokio/pull/2540
[#2541]: https://github.com/tokio-rs/tokio/pull/2541
[#2549]: https://github.com/tokio-rs/tokio/pull/2549
[#2564]: https://github.com/tokio-rs/tokio/pull/2564
[#2567]: https://github.com/tokio-rs/tokio/pull/2567
[#2569]: https://github.com/tokio-rs/tokio/pull/2569
[#2572]: https://github.com/tokio-rs/tokio/pull/2572
[#2575]: https://github.com/tokio-rs/tokio/pull/2575
[#2577]: https://github.com/tokio-rs/tokio/pull/2577
[#2589]: https://github.com/tokio-rs/tokio/pull/2589
[#2597]: https://github.com/tokio-rs/tokio/pull/2597
[#2600]: https://github.com/tokio-rs/tokio/pull/2600
[#2604]: https://github.com/tokio-rs/tokio/pull/2604
[#2612]: https://github.com/tokio-rs/tokio/pull/2612
[#2615]: https://github.com/tokio-rs/tokio/pull/2615
[#2621]: https://github.com/tokio-rs/tokio/pull/2621
[#2622]: https://github.com/tokio-rs/tokio/pull/2622
[#2623]: https://github.com/tokio-rs/tokio/pull/2623
[#2624]: https://github.com/tokio-rs/tokio/pull/2624
[#2630]: https://github.com/tokio-rs/tokio/pull/2630
[#2634]: https://github.com/tokio-rs/tokio/pull/2634
[#2645]: https://github.com/tokio-rs/tokio/pull/2645
[#2647]: https://github.com/tokio-rs/tokio/pull/2647
[#2654]: https://github.com/tokio-rs/tokio/pull/2654
[#2655]: https://github.com/tokio-rs/tokio/pull/2655
[#2656]: https://github.com/tokio-rs/tokio/pull/2656
[#2658]: https://github.com/tokio-rs/tokio/pull/2658
[#2663]: https://github.com/tokio-rs/tokio/pull/2663

# 0.2.21 (May 13, 2020)

### Fixes

- macros: disambiguate built-in `#[test]` attribute in macro expansion ([#2503])
- rt: `LocalSet` and task budgeting ([#2462]).
- rt: task budgeting with `block_in_place` ([#2502]).
- sync: release `broadcast` channel memory without sending a value ([#2509]).
- time: notify when resetting a `Delay` to a time in the past ([#2290])

### Added

- io: `get_mut`, `get_ref`, and `into_inner` to `Lines` ([#2450]).
- io: `mio::Ready` argument to `PollEvented` ([#2419]).
- os: illumos support ([#2486]).
- rt: `Handle::spawn_blocking` ([#2501]).
- sync: `OwnedMutexGuard` for `Arc<Mutex<T>>` ([#2455]).

[#2290]: https://github.com/tokio-rs/tokio/pull/2290
[#2419]: https://github.com/tokio-rs/tokio/pull/2419
[#2450]: https://github.com/tokio-rs/tokio/pull/2450
[#2455]: https://github.com/tokio-rs/tokio/pull/2455
[#2462]: https://github.com/tokio-rs/tokio/pull/2462
[#2486]: https://github.com/tokio-rs/tokio/pull/2486
[#2501]: https://github.com/tokio-rs/tokio/pull/2501
[#2502]: https://github.com/tokio-rs/tokio/pull/2502
[#2503]: https://github.com/tokio-rs/tokio/pull/2503
[#2509]: https://github.com/tokio-rs/tokio/pull/2509

# 0.2.20 (April 28, 2020)

### Fixes

- sync: `broadcast` closing the channel no longer requires capacity ([#2448]).
- rt: regression when configuring runtime with `max_threads` less than number of CPUs ([#2457]).

[#2448]: https://github.com/tokio-rs/tokio/pull/2448
[#2457]: https://github.com/tokio-rs/tokio/pull/2457

# 0.2.19 (April 24, 2020)

### Fixes

- docs: misc improvements ([#2400], [#2405], [#2414], [#2420], [#2423], [#2426], [#2427], [#2434], [#2436], [#2440]).
- rt: support `block_in_place` in more contexts ([#2409], [#2410]).
- stream: no panic in `merge()` and `chain()` when using `size_hint()` ([#2430]).
- task: include visibility modifier when defining a task-local ([#2416]).

### Added

- rt: `runtime::Handle::block_on` ([#2437]).
- sync: owned `Semaphore` permit ([#2421]).
- tcp: owned split ([#2270]).

[#2270]: https://github.com/tokio-rs/tokio/pull/2270
[#2400]: https://github.com/tokio-rs/tokio/pull/2400
[#2405]: https://github.com/tokio-rs/tokio/pull/2405
[#2409]: https://github.com/tokio-rs/tokio/pull/2409
[#2410]: https://github.com/tokio-rs/tokio/pull/2410
[#2414]: https://github.com/tokio-rs/tokio/pull/2414
[#2416]: https://github.com/tokio-rs/tokio/pull/2416
[#2420]: https://github.com/tokio-rs/tokio/pull/2420
[#2421]: https://github.com/tokio-rs/tokio/pull/2421
[#2423]: https://github.com/tokio-rs/tokio/pull/2423
[#2426]: https://github.com/tokio-rs/tokio/pull/2426
[#2427]: https://github.com/tokio-rs/tokio/pull/2427
[#2430]: https://github.com/tokio-rs/tokio/pull/2430
[#2434]: https://github.com/tokio-rs/tokio/pull/2434
[#2436]: https://github.com/tokio-rs/tokio/pull/2436
[#2437]: https://github.com/tokio-rs/tokio/pull/2437
[#2440]: https://github.com/tokio-rs/tokio/pull/2440

# 0.2.18 (April 12, 2020)

### Fixes

- task: `LocalSet` was incorrectly marked as `Send` ([#2398])
- io: correctly report `WriteZero` failure in `write_int` ([#2334])

[#2334]: https://github.com/tokio-rs/tokio/pull/2334
[#2398]: https://github.com/tokio-rs/tokio/pull/2398

# 0.2.17 (April 9, 2020)

### Fixes

- rt: bug in work-stealing queue ([#2387])

### Changes

- rt: threadpool uses logical CPU count instead of physical by default ([#2391])

[#2387]: https://github.com/tokio-rs/tokio/pull/2387
[#2391]: https://github.com/tokio-rs/tokio/pull/2391

# 0.2.16 (April 3, 2020)

### Fixes

- sync: fix a regression where `Mutex`, `Semaphore`, and `RwLock` futures no
  longer implement `Sync` ([#2375])
- fs: fix `fs::copy` not copying file permissions ([#2354])

### Added

- time: added `deadline` method to `delay_queue::Expired` ([#2300])
- io: added `StreamReader` ([#2052])

[#2052]: https://github.com/tokio-rs/tokio/pull/2052
[#2300]: https://github.com/tokio-rs/tokio/pull/2300
[#2354]: https://github.com/tokio-rs/tokio/pull/2354
[#2375]: https://github.com/tokio-rs/tokio/pull/2375

# 0.2.15 (April 2, 2020)

### Fixes

- rt: fix queue regression ([#2362]).

### Added

- sync: Add disarm to `mpsc::Sender` ([#2358]).

[#2358]: https://github.com/tokio-rs/tokio/pull/2358
[#2362]: https://github.com/tokio-rs/tokio/pull/2362

# 0.2.14 (April 1, 2020)

### Fixes

- rt: concurrency bug in scheduler ([#2273]).
- rt: concurrency bug with shell runtime ([#2333]).
- test-util: correct pause/resume of time ([#2253]).
- time: `DelayQueue` correct wakeup after `insert` ([#2285]).

### Added

- io: impl `RawFd`, `AsRawHandle` for std io types ([#2335]).
- rt: automatic cooperative task yielding ([#2160], [#2343], [#2349]).
- sync: `RwLock::into_inner` ([#2321]).

### Changed

- sync: semaphore, mutex internals rewritten to avoid allocations ([#2325]).

[#2160]: https://github.com/tokio-rs/tokio/pull/2160
[#2253]: https://github.com/tokio-rs/tokio/pull/2253
[#2273]: https://github.com/tokio-rs/tokio/pull/2273
[#2285]: https://github.com/tokio-rs/tokio/pull/2285
[#2321]: https://github.com/tokio-rs/tokio/pull/2321
[#2325]: https://github.com/tokio-rs/tokio/pull/2325
[#2333]: https://github.com/tokio-rs/tokio/pull/2333
[#2335]: https://github.com/tokio-rs/tokio/pull/2335
[#2343]: https://github.com/tokio-rs/tokio/pull/2343
[#2349]: https://github.com/tokio-rs/tokio/pull/2349

# 0.2.13 (February 28, 2020)

### Fixes

- macros: unresolved import in `pin!` ([#2281]).

[#2281]: https://github.com/tokio-rs/tokio/pull/2281

# 0.2.12 (February 27, 2020)

### Fixes

- net: `UnixStream::poll_shutdown` should call `shutdown(Write)` ([#2245]).
- process: Wake up read and write on `EPOLLERR` ([#2218]).
- rt: potential deadlock when using `block_in_place` and shutting down the
  runtime ([#2119]).
- rt: only detect number of CPUs if `core_threads` not specified ([#2238]).
- sync: reduce `watch::Receiver` struct size ([#2191]).
- time: succeed when setting delay of `$MAX-1` ([#2184]).
- time: avoid having to poll `DelayQueue` after inserting new delay ([#2217]).

### Added

- macros: `pin!` variant that assigns to identifier and pins ([#2274]).
- net: impl `Stream` for `Listener` types ([#2275]).
- rt: `Runtime::shutdown_timeout` waits for runtime to shutdown for specified
  duration ([#2186]).
- stream: `StreamMap` merges streams and can insert / remove streams at
  runtime ([#2185]).
- stream: `StreamExt::skip()` skips a fixed number of items ([#2204]).
- stream: `StreamExt::skip_while()` skips items based on a predicate ([#2205]).
- sync: `Notify` provides basic `async` / `await` task notification ([#2210]).
- sync: `Mutex::into_inner` retrieves guarded data ([#2250]).
- sync: `mpsc::Sender::send_timeout` sends, waiting for up to specified duration
  for channel capacity ([#2227]).
- time: impl `Ord` and `Hash` for `Instant` ([#2239]).

[#2119]: https://github.com/tokio-rs/tokio/pull/2119
[#2184]: https://github.com/tokio-rs/tokio/pull/2184
[#2185]: https://github.com/tokio-rs/tokio/pull/2185
[#2186]: https://github.com/tokio-rs/tokio/pull/2186
[#2191]: https://github.com/tokio-rs/tokio/pull/2191
[#2204]: https://github.com/tokio-rs/tokio/pull/2204
[#2205]: https://github.com/tokio-rs/tokio/pull/2205
[#2210]: https://github.com/tokio-rs/tokio/pull/2210
[#2217]: https://github.com/tokio-rs/tokio/pull/2217
[#2218]: https://github.com/tokio-rs/tokio/pull/2218
[#2227]: https://github.com/tokio-rs/tokio/pull/2227
[#2238]: https://github.com/tokio-rs/tokio/pull/2238
[#2239]: https://github.com/tokio-rs/tokio/pull/2239
[#2245]: https://github.com/tokio-rs/tokio/pull/2245
[#2250]: https://github.com/tokio-rs/tokio/pull/2250
[#2274]: https://github.com/tokio-rs/tokio/pull/2274
[#2275]: https://github.com/tokio-rs/tokio/pull/2275

# 0.2.11 (January 27, 2020)

### Fixes

- docs: misc fixes and tweaks ([#2155], [#2103], [#2027], [#2167], [#2175]).
- macros: handle generics in `#[tokio::main]` method ([#2177]).
- sync: `broadcast` potential lost notifications ([#2135]).
- rt: improve "no runtime" panic messages ([#2145]).

### Added

- optional support for using `parking_lot` internally ([#2164]).
- fs: `fs::copy`, an async version of `std::fs::copy` ([#2079]).
- macros: `select!` waits for the first branch to complete ([#2152]).
- macros: `join!` waits for all branches to complete ([#2158]).
- macros: `try_join!` waits for all branches to complete or the first error ([#2169]).
- macros: `pin!` pins a value to the stack ([#2163]).
- net: `ReadHalf::poll()` and `ReadHalf::poll_peak` ([#2151])
- stream: `StreamExt::timeout()` sets a per-item max duration ([#2149]).
- stream: `StreamExt::fold()` applies a function, producing a single value. ([#2122]).
- sync: impl `Eq`, `PartialEq` for `oneshot::RecvError` ([#2168]).
- task: methods for inspecting the `JoinError` cause ([#2051]).

[#2027]: https://github.com/tokio-rs/tokio/pull/2027
[#2051]: https://github.com/tokio-rs/tokio/pull/2051
[#2079]: https://github.com/tokio-rs/tokio/pull/2079
[#2103]: https://github.com/tokio-rs/tokio/pull/2103
[#2122]: https://github.com/tokio-rs/tokio/pull/2122
[#2135]: https://github.com/tokio-rs/tokio/pull/2135
[#2145]: https://github.com/tokio-rs/tokio/pull/2145
[#2149]: https://github.com/tokio-rs/tokio/pull/2149
[#2151]: https://github.com/tokio-rs/tokio/pull/2151
[#2152]: https://github.com/tokio-rs/tokio/pull/2152
[#2155]: https://github.com/tokio-rs/tokio/pull/2155
[#2158]: https://github.com/tokio-rs/tokio/pull/2158
[#2163]: https://github.com/tokio-rs/tokio/pull/2163
[#2164]: https://github.com/tokio-rs/tokio/pull/2164
[#2167]: https://github.com/tokio-rs/tokio/pull/2167
[#2168]: https://github.com/tokio-rs/tokio/pull/2168
[#2169]: https://github.com/tokio-rs/tokio/pull/2169
[#2175]: https://github.com/tokio-rs/tokio/pull/2175
[#2177]: https://github.com/tokio-rs/tokio/pull/2177

# 0.2.10 (January 21, 2020)

### Fixes

- `#[tokio::main]` when `rt-core` feature flag is not enabled ([#2139]).
- remove `AsyncBufRead` from `BufStream` impl block ([#2108]).
- potential undefined behavior when implementing `AsyncRead` incorrectly ([#2030]).

### Added

- `BufStream::with_capacity` ([#2125]).
- impl `From` and `Default` for `RwLock` ([#2089]).
- `io::ReadHalf::is_pair_of` checks if provided `WriteHalf` is for the same
  underlying object ([#1762], [#2144]).
- `runtime::Handle::try_current()` returns a handle to the current runtime ([#2118]).
- `stream::empty()` returns an immediately ready empty stream ([#2092]).
- `stream::once(val)` returns a stream that yields a single value: `val` ([#2094]).
- `stream::pending()` returns a stream that never becomes ready ([#2092]).
- `StreamExt::chain()` sequences a second stream after the first completes ([#2093]).
- `StreamExt::collect()` transform a stream into a collection ([#2109]).
- `StreamExt::fuse` ends the stream after the first `None` ([#2085]).
- `StreamExt::merge` combines two streams, yielding values as they become ready ([#2091]).
- Task-local storage ([#2126]).

[#1762]: https://github.com/tokio-rs/tokio/pull/1762
[#2030]: https://github.com/tokio-rs/tokio/pull/2030
[#2085]: https://github.com/tokio-rs/tokio/pull/2085
[#2089]: https://github.com/tokio-rs/tokio/pull/2089
[#2091]: https://github.com/tokio-rs/tokio/pull/2091
[#2092]: https://github.com/tokio-rs/tokio/pull/2092
[#2093]: https://github.com/tokio-rs/tokio/pull/2093
[#2094]: https://github.com/tokio-rs/tokio/pull/2094
[#2108]: https://github.com/tokio-rs/tokio/pull/2108
[#2109]: https://github.com/tokio-rs/tokio/pull/2109
[#2118]: https://github.com/tokio-rs/tokio/pull/2118
[#2125]: https://github.com/tokio-rs/tokio/pull/2125
[#2126]: https://github.com/tokio-rs/tokio/pull/2126
[#2139]: https://github.com/tokio-rs/tokio/pull/2139
[#2144]: https://github.com/tokio-rs/tokio/pull/2144

# 0.2.9 (January 9, 2020)

### Fixes

- `AsyncSeek` impl for `File` ([#1986]).
- rt: shutdown deadlock in `threaded_scheduler` ([#2074], [#2082]).
- rt: memory ordering when dropping `JoinHandle` ([#2044]).
- docs: misc API documentation fixes and improvements.

[#1986]: https://github.com/tokio-rs/tokio/pull/1986
[#2044]: https://github.com/tokio-rs/tokio/pull/2044
[#2074]: https://github.com/tokio-rs/tokio/pull/2074
[#2082]: https://github.com/tokio-rs/tokio/pull/2082

# 0.2.8 (January 7, 2020)

### Fixes

- depend on new version of `tokio-macros`.

# 0.2.7 (January 7, 2020)

### Fixes

- potential deadlock when dropping `basic_scheduler` Runtime.
- calling `spawn_blocking` from within a `spawn_blocking` ([#2006]).
- storing a `Runtime` instance in a thread-local ([#2011]).
- miscellaneous documentation fixes.
- rt: fix `Waker::will_wake` to return true when tasks match ([#2045]).
- test-util: `time::advance` runs pending tasks before changing the time ([#2059]).

### Added

- `net::lookup_host` maps a `T: ToSocketAddrs` to a stream of `SocketAddrs` ([#1870]).
- `process::Child` fields are made public to match `std` ([#2014]).
- impl `Stream` for `sync::broadcast::Receiver` ([#2012]).
- `sync::RwLock` provides an asynchronous read-write lock ([#1699]).
- `runtime::Handle::current` returns the handle for the current runtime ([#2040]).
- `StreamExt::filter` filters stream values according to a predicate ([#2001]).
- `StreamExt::filter_map` simultaneously filter and map stream values ([#2001]).
- `StreamExt::try_next` convenience for streams of `Result<T, E>` ([#2005]).
- `StreamExt::take` limits a stream to a specified number of values ([#2025]).
- `StreamExt::take_while` limits a stream based on a predicate ([#2029]).
- `StreamExt::all` tests if every element of the stream matches a predicate ([#2035]).
- `StreamExt::any` tests if any element of the stream matches a predicate ([#2034]).
- `task::LocalSet.await` runs spawned tasks until the set is idle ([#1971]).
- `time::DelayQueue::len` returns the number entries in the queue ([#1755]).
- expose runtime options from the `#[tokio::main]` and `#[tokio::test]` ([#2022]).

[#1699]: https://github.com/tokio-rs/tokio/pull/1699
[#1755]: https://github.com/tokio-rs/tokio/pull/1755
[#1870]: https://github.com/tokio-rs/tokio/pull/1870
[#1971]: https://github.com/tokio-rs/tokio/pull/1971
[#2001]: https://github.com/tokio-rs/tokio/pull/2001
[#2005]: https://github.com/tokio-rs/tokio/pull/2005
[#2006]: https://github.com/tokio-rs/tokio/pull/2006
[#2011]: https://github.com/tokio-rs/tokio/pull/2011
[#2012]: https://github.com/tokio-rs/tokio/pull/2012
[#2014]: https://github.com/tokio-rs/tokio/pull/2014
[#2022]: https://github.com/tokio-rs/tokio/pull/2022
[#2025]: https://github.com/tokio-rs/tokio/pull/2025
[#2029]: https://github.com/tokio-rs/tokio/pull/2029
[#2034]: https://github.com/tokio-rs/tokio/pull/2034
[#2035]: https://github.com/tokio-rs/tokio/pull/2035
[#2040]: https://github.com/tokio-rs/tokio/pull/2040
[#2045]: https://github.com/tokio-rs/tokio/pull/2045
[#2059]: https://github.com/tokio-rs/tokio/pull/2059

# 0.2.6 (December 19, 2019)

### Fixes

- `fs::File::seek` API regression ([#1991]).

[#1991]: https://github.com/tokio-rs/tokio/pull/1991

# 0.2.5 (December 18, 2019)

### Added

- `io::AsyncSeek` trait ([#1924]).
- `Mutex::try_lock` ([#1939])
- `mpsc::Receiver::try_recv` and `mpsc::UnboundedReceiver::try_recv` ([#1939]).
- `writev` support for `TcpStream` ([#1956]).
- `time::throttle` for throttling streams ([#1949]).
- implement `Stream` for `time::DelayQueue` ([#1975]).
- `sync::broadcast` provides a fan-out channel ([#1943]).
- `sync::Semaphore` provides an async semaphore ([#1973]).
- `stream::StreamExt` provides stream utilities ([#1962]).

### Fixes

- deadlock risk while shutting down the runtime ([#1972]).
- panic while shutting down the runtime ([#1978]).
- `sync::MutexGuard` debug output ([#1961]).
- misc doc improvements ([#1933], [#1934], [#1940], [#1942]).

### Changes

- runtime threads are configured with `runtime::Builder::core_threads` and
  `runtime::Builder::max_threads`. `runtime::Builder::num_threads` is
  deprecated ([#1977]).

[#1924]: https://github.com/tokio-rs/tokio/pull/1924
[#1933]: https://github.com/tokio-rs/tokio/pull/1933
[#1934]: https://github.com/tokio-rs/tokio/pull/1934
[#1939]: https://github.com/tokio-rs/tokio/pull/1939
[#1940]: https://github.com/tokio-rs/tokio/pull/1940
[#1942]: https://github.com/tokio-rs/tokio/pull/1942
[#1943]: https://github.com/tokio-rs/tokio/pull/1943
[#1949]: https://github.com/tokio-rs/tokio/pull/1949
[#1956]: https://github.com/tokio-rs/tokio/pull/1956
[#1961]: https://github.com/tokio-rs/tokio/pull/1961
[#1962]: https://github.com/tokio-rs/tokio/pull/1962
[#1972]: https://github.com/tokio-rs/tokio/pull/1972
[#1973]: https://github.com/tokio-rs/tokio/pull/1973
[#1975]: https://github.com/tokio-rs/tokio/pull/1975
[#1977]: https://github.com/tokio-rs/tokio/pull/1977
[#1978]: https://github.com/tokio-rs/tokio/pull/1978

# 0.2.4 (December 6, 2019)

### Fixes

- `sync::Mutex` deadlock when `lock()` future is dropped early ([#1898]).

[#1898]: https://github.com/tokio-rs/tokio/pull/1898

# 0.2.3 (December 6, 2019)

### Added

- read / write integers using `AsyncReadExt` and `AsyncWriteExt` ([#1863]).
- `read_buf` / `write_buf` for reading / writing `Buf` / `BufMut` ([#1881]).
- `TcpStream::poll_peek` - pollable API for performing TCP peek ([#1864]).
- `sync::oneshot::error::TryRecvError` provides variants to detect the error
  kind ([#1874]).
- `LocalSet::block_on` accepts `!'static` task ([#1882]).
- `task::JoinError` is now `Sync` ([#1888]).
- impl conversions between `tokio::time::Instant` and
  `std::time::Instant` ([#1904]).

### Fixes

- calling `spawn_blocking` after runtime shutdown ([#1875]).
- `LocalSet` drop infinite loop ([#1892]).
- `LocalSet` hang under load ([#1905]).
- improved documentation ([#1865], [#1866], [#1868], [#1874], [#1876], [#1911]).

[#1863]: https://github.com/tokio-rs/tokio/pull/1863
[#1864]: https://github.com/tokio-rs/tokio/pull/1864
[#1865]: https://github.com/tokio-rs/tokio/pull/1865
[#1866]: https://github.com/tokio-rs/tokio/pull/1866
[#1868]: https://github.com/tokio-rs/tokio/pull/1868
[#1874]: https://github.com/tokio-rs/tokio/pull/1874
[#1875]: https://github.com/tokio-rs/tokio/pull/1875
[#1876]: https://github.com/tokio-rs/tokio/pull/1876
[#1881]: https://github.com/tokio-rs/tokio/pull/1881
[#1882]: https://github.com/tokio-rs/tokio/pull/1882
[#1888]: https://github.com/tokio-rs/tokio/pull/1888
[#1892]: https://github.com/tokio-rs/tokio/pull/1892
[#1904]: https://github.com/tokio-rs/tokio/pull/1904
[#1905]: https://github.com/tokio-rs/tokio/pull/1905
[#1911]: https://github.com/tokio-rs/tokio/pull/1911

# 0.2.2 (November 29, 2019)

### Fixes

- scheduling with `basic_scheduler` ([#1861]).
- update `spawn` panic message to specify that a task scheduler is required ([#1839]).
- API docs example for `runtime::Builder` to include a task scheduler ([#1841]).
- general documentation ([#1834]).
- building on illumos/solaris ([#1772]).
- panic when dropping `LocalSet` ([#1843]).
- API docs mention the required Cargo features for `Builder::{basic, threaded}_scheduler` ([#1858]).

### Added

- impl `Stream` for `signal::unix::Signal` ([#1849]).
- API docs for platform specific behavior of `signal::ctrl_c` and `signal::unix::Signal` ([#1854]).
- API docs for `signal::unix::Signal::{recv, poll_recv}` and `signal::windows::CtrlBreak::{recv, poll_recv}` ([#1854]).
- `File::into_std` and `File::try_into_std` methods ([#1856]).

[#1772]: https://github.com/tokio-rs/tokio/pull/1772
[#1834]: https://github.com/tokio-rs/tokio/pull/1834
[#1839]: https://github.com/tokio-rs/tokio/pull/1839
[#1841]: https://github.com/tokio-rs/tokio/pull/1841
[#1843]: https://github.com/tokio-rs/tokio/pull/1843
[#1849]: https://github.com/tokio-rs/tokio/pull/1849
[#1854]: https://github.com/tokio-rs/tokio/pull/1854
[#1856]: https://github.com/tokio-rs/tokio/pull/1856
[#1858]: https://github.com/tokio-rs/tokio/pull/1858
[#1861]: https://github.com/tokio-rs/tokio/pull/1861

# 0.2.1 (November 26, 2019)

### Fixes

- API docs for `TcpListener::incoming`, `UnixListener::incoming` ([#1831]).

### Added

- `tokio::task::LocalSet` provides a strategy for spawning `!Send` tasks ([#1733]).
- export `tokio::time::Elapsed` ([#1826]).
- impl `AsRawFd`, `AsRawHandle` for `tokio::fs::File` ([#1827]).

[#1733]: https://github.com/tokio-rs/tokio/pull/1733
[#1826]: https://github.com/tokio-rs/tokio/pull/1826
[#1827]: https://github.com/tokio-rs/tokio/pull/1827
[#1831]: https://github.com/tokio-rs/tokio/pull/1831

# 0.2.0 (November 26, 2019)

A major breaking change. Most implementation and APIs have changed one way or
another. This changelog entry contains a highlight

### Changed

- APIs are updated to use `async / await`.
- most `tokio-*` crates are collapsed into this crate.
- Scheduler is rewritten.
- `tokio::spawn` returns a `JoinHandle`.
- A single I/O / timer is used per runtime.
- I/O driver uses a concurrent slab for allocating state.
- components are made available via feature flag.
- Use `bytes` 0.5
- `tokio::codec` is moved to `tokio-util`.

### Removed

- Standalone `timer` and `net` drivers are removed, use `Runtime` instead
- `current_thread` runtime is removed, use `tokio::runtime::Runtime` with
  `basic_scheduler` instead.

# 0.1.21 (May 30, 2019)

### Changed

- Bump `tokio-trace-core` version to 0.2 ([#1111]).

[#1111]: https://github.com/tokio-rs/tokio/pull/1111

# 0.1.20 (May 14, 2019)

### Added

- `tokio::runtime::Builder::panic_handler` allows configuring handling
  panics on the runtime ([#1055]).

[#1055]: https://github.com/tokio-rs/tokio/pull/1055

# 0.1.19 (April 22, 2019)

### Added

- Re-export `tokio::sync::Mutex` primitive ([#964]).

[#964]: https://github.com/tokio-rs/tokio/pull/964

# 0.1.18 (March 22, 2019)

### Added

- `TypedExecutor` re-export and implementations ([#993]).

[#993]: https://github.com/tokio-rs/tokio/pull/993

# 0.1.17 (March 13, 2019)

### Added

- Propagate trace subscriber in the runtime ([#966]).

[#966]: https://github.com/tokio-rs/tokio/pull/966

# 0.1.16 (March 1, 2019)

### Fixed

- async-await: track latest nightly changes ([#940]).

### Added

- `sync::Watch`, a single value broadcast channel ([#922]).
- Async equivalent of read / write file helpers being added to `std` ([#896]).

[#896]: https://github.com/tokio-rs/tokio/pull/896
[#922]: https://github.com/tokio-rs/tokio/pull/922
[#940]: https://github.com/tokio-rs/tokio/pull/940

# 0.1.15 (January 24, 2019)

### Added

- Re-export tokio-sync APIs ([#839]).
- Stream enumerate combinator ([#832]).

[#832]: https://github.com/tokio-rs/tokio/pull/832
[#839]: https://github.com/tokio-rs/tokio/pull/839

# 0.1.14 (January 6, 2019)

- Use feature flags to break up the crate, allowing users to pick & choose
  components ([#808]).
- Export `UnixDatagram` and `UnixDatagramFramed` ([#772]).

[#772]: https://github.com/tokio-rs/tokio/pull/772
[#808]: https://github.com/tokio-rs/tokio/pull/808

# 0.1.13 (November 21, 2018)

- Fix `Runtime::reactor()` when no tasks are spawned ([#721]).
- `runtime::Builder` no longer uses deprecated methods ([#749]).
- Provide `after_start` and `before_stop` configuration settings for
  `Runtime` ([#756]).
- Implement throttle stream combinator ([#736]).

[#721]: https://github.com/tokio-rs/tokio/pull/721
[#736]: https://github.com/tokio-rs/tokio/pull/736
[#749]: https://github.com/tokio-rs/tokio/pull/749
[#756]: https://github.com/tokio-rs/tokio/pull/756

# 0.1.12 (October 23, 2018)

- runtime: expose `keep_alive` on runtime builder ([#676]).
- runtime: create a reactor per worker thread ([#660]).
- codec: fix panic in `LengthDelimitedCodec` ([#682]).
- io: re-export `tokio_io::io::read` function ([#689]).
- runtime: check for executor re-entry in more places ([#708]).

[#660]: https://github.com/tokio-rs/tokio/pull/660
[#676]: https://github.com/tokio-rs/tokio/pull/676
[#682]: https://github.com/tokio-rs/tokio/pull/682
[#689]: https://github.com/tokio-rs/tokio/pull/689
[#708]: https://github.com/tokio-rs/tokio/pull/708

# 0.1.11 (September 28, 2018)

- Fix `tokio-async-await` dependency ([#675]).

[#675]: https://github.com/tokio-rs/tokio/pull/675

# 0.1.10 (September 27, 2018)

- Fix minimal versions

# 0.1.9 (September 27, 2018)

- Experimental async/await improvements ([#661]).
- Re-export `TaskExecutor` from `tokio-current-thread` ([#652]).
- Improve `Runtime` builder API ([#645]).
- `tokio::run` panics when called from the context of an executor
  ([#646]).
- Introduce `StreamExt` with a `timeout` helper ([#573]).
- Move `length_delimited` into `tokio` ([#575]).
- Re-organize `tokio::net` module ([#548]).
- Re-export `tokio-current-thread::spawn` in current_thread runtime
  ([#579]).

[#548]: https://github.com/tokio-rs/tokio/pull/548
[#573]: https://github.com/tokio-rs/tokio/pull/573
[#575]: https://github.com/tokio-rs/tokio/pull/575
[#579]: https://github.com/tokio-rs/tokio/pull/579
[#645]: https://github.com/tokio-rs/tokio/pull/645
[#646]: https://github.com/tokio-rs/tokio/pull/646
[#652]: https://github.com/tokio-rs/tokio/pull/652
[#661]: https://github.com/tokio-rs/tokio/pull/661

# 0.1.8 (August 23, 2018)

- Extract tokio::executor::current_thread to a sub crate ([#370])
- Add `Runtime::block_on` ([#398])
- Add `runtime::current_thread::block_on_all` ([#477])
- Misc documentation improvements ([#450])
- Implement `std::error::Error` for error types ([#501])

[#370]: https://github.com/tokio-rs/tokio/pull/370
[#398]: https://github.com/tokio-rs/tokio/pull/398
[#450]: https://github.com/tokio-rs/tokio/pull/450
[#477]: https://github.com/tokio-rs/tokio/pull/477
[#501]: https://github.com/tokio-rs/tokio/pull/501

# 0.1.7 (June 6, 2018)

- Add `Runtime::block_on` for concurrent runtime ([#391]).
- Provide handle to `current_thread::Runtime` that allows spawning tasks from
  other threads ([#340]).
- Provide `clock::now()`, a configurable source of time ([#381]).

[#340]: https://github.com/tokio-rs/tokio/pull/340
[#381]: https://github.com/tokio-rs/tokio/pull/381
[#391]: https://github.com/tokio-rs/tokio/pull/391

# 0.1.6 (May 2, 2018)

- Add asynchronous filesystem APIs ([#323]).
- Add "current thread" runtime variant ([#308]).
- `CurrentThread`: Expose inner `Park` instance.
- Improve fairness of `CurrentThread` executor ([#313]).

[#308]: https://github.com/tokio-rs/tokio/pull/308
[#313]: https://github.com/tokio-rs/tokio/pull/313
[#323]: https://github.com/tokio-rs/tokio/pull/323

# 0.1.5 (March 30, 2018)

- Provide timer API ([#266])

[#266]: https://github.com/tokio-rs/tokio/pull/266

# 0.1.4 (March 22, 2018)

- Fix build on FreeBSD ([#218])
- Shutdown the Runtime when the handle is dropped ([#214])
- Set Runtime thread name prefix for worker threads ([#232])
- Add builder for Runtime ([#234])
- Extract TCP and UDP types into separate crates ([#224])
- Optionally support futures 0.2.

[#214]: https://github.com/tokio-rs/tokio/pull/214
[#218]: https://github.com/tokio-rs/tokio/pull/218
[#224]: https://github.com/tokio-rs/tokio/pull/224
[#232]: https://github.com/tokio-rs/tokio/pull/232
[#234]: https://github.com/tokio-rs/tokio/pull/234

# 0.1.3 (March 09, 2018)

- Fix `CurrentThread::turn` to block on idle ([#212]).

[#212]: https://github.com/tokio-rs/tokio/pull/212

# 0.1.2 (March 09, 2018)

- Introduce Tokio Runtime ([#141])
- Provide `CurrentThread` for more flexible usage of current thread executor ([#141]).
- Add Lio for platforms that support it ([#142]).
- I/O resources now lazily bind to the reactor ([#160]).
- Extract Reactor to dedicated crate ([#169])
- Add facade to sub crates and add prelude ([#166]).
- Switch TCP/UDP fns to poll\_ -> Poll<...> style ([#175])

[#141]: https://github.com/tokio-rs/tokio/pull/141
[#142]: https://github.com/tokio-rs/tokio/pull/142
[#160]: https://github.com/tokio-rs/tokio/pull/160
[#166]: https://github.com/tokio-rs/tokio/pull/166
[#169]: https://github.com/tokio-rs/tokio/pull/169
[#175]: https://github.com/tokio-rs/tokio/pull/175

# 0.1.1 (February 09, 2018)

- Doc fixes

# 0.1.0 (February 07, 2018)

- Initial crate released based on [RFC](https://github.com/tokio-rs/tokio-rfcs/pull/3).
