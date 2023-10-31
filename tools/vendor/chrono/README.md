[Chrono][docsrs]: Date and Time for Rust
========================================

[![Chrono GitHub Actions][gh-image]][gh-checks]
[![Chrono on crates.io][cratesio-image]][cratesio]
[![Chrono on docs.rs][docsrs-image]][docsrs]
[![Join the chat at https://gitter.im/chrono-rs/chrono][gitter-image]][gitter]

[gh-image]: https://github.com/chronotope/chrono/actions/workflows/test.yml/badge.svg
[gh-checks]: https://github.com/chronotope/chrono/actions?query=workflow%3Atest
[cratesio-image]: https://img.shields.io/crates/v/chrono.svg
[cratesio]: https://crates.io/crates/chrono
[docsrs-image]: https://docs.rs/chrono/badge.svg
[docsrs]: https://docs.rs/chrono
[gitter-image]: https://badges.gitter.im/chrono-rs/chrono.svg
[gitter]: https://gitter.im/chrono-rs/chrono

It aims to be a feature-complete superset of
the [time](https://github.com/rust-lang-deprecated/time) library.
In particular,

* Chrono strictly adheres to ISO 8601.
* Chrono is timezone-aware by default, with separate timezone-naive types.
* Chrono is space-optimal and (while not being the primary goal) reasonably efficient.

There were several previous attempts to bring a good date and time library to Rust,
which Chrono builds upon and should acknowledge:

* [Initial research on
   the wiki](https://github.com/rust-lang/rust-wiki-backup/blob/master/Lib-datetime.md)
* Dietrich Epp's [datetime-rs](https://github.com/depp/datetime-rs)
* Luis de Bethencourt's [rust-datetime](https://github.com/luisbg/rust-datetime)

## Limitations

Only proleptic Gregorian calendar (i.e. extended to support older dates) is supported.
Be very careful if you really have to deal with pre-20C dates, they can be in Julian or others.

Date types are limited in about +/- 262,000 years from the common epoch.
Time types are limited in the nanosecond accuracy.

[Leap seconds are supported in the representation but
Chrono doesn't try to make use of them](https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveTime.html#leap-second-handling).
(The main reason is that leap seconds are not really predictable.)
Almost *every* operation over the possible leap seconds will ignore them.
Consider using `NaiveDateTime` with the implicit TAI (International Atomic Time) scale
if you want.

Chrono inherently does not support an inaccurate or partial date and time representation.
Any operation that can be ambiguous will return `None` in such cases.
For example, "a month later" of 2014-01-30 is not well-defined
and consequently `Utc.ymd_opt(2014, 1, 30).unwrap().with_month(2)` returns `None`.

Non ISO week handling is not yet supported.
For now you can use the [chrono_ext](https://crates.io/crates/chrono_ext)
crate ([sources](https://github.com/bcourtine/chrono-ext/)).

Advanced time zone handling is not yet supported.
For now you can try the [Chrono-tz](https://github.com/chronotope/chrono-tz/) crate instead.
