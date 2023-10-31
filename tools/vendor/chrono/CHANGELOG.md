ChangeLog for Chrono
====================

This documents notable changes to [Chrono](https://github.com/chronotope/chrono)
up to and including version 0.4.19. For later releases, please review the
release notes on [GitHub](https://github.com/chronotope/chrono/releases).

## 0.4.19

* Correct build on solaris/illumos

## 0.4.18

* Restore support for x86_64-fortanix-unknown-sgx

## 0.4.17

* Fix a name resolution error in wasm-bindgen code introduced by removing the dependency on time
  v0.1

## 0.4.16

### Features

* Add %Z specifier to the `FromStr`, similar to the glibc strptime
  (does not set the offset from the timezone name)

* Drop the dependency on time v0.1, which is deprecated, unless the `oldtime`
  feature is active. This feature is active by default in v0.4.16 for backwards
  compatibility, but will likely be removed in v0.5. Code that imports
  `time::Duration` should be switched to import `chrono::Duration` instead to
  avoid breakage.

## 0.4.15

### Fixes

* Correct usage of vec in specific feature combinations (@quodlibetor)

## 0.4.14 **YANKED**

### Features

* Add day and week iterators for `NaiveDate` (@gnzlbg & @robyoung)
* Add a `Month` enum (@hhamana)
* Add `locales`. All format functions can now use locales, see the documentation for the
  `unstable-locales` feature.
* Fix `Local.from_local_datetime` method for wasm

### Improvements

* Added MIN and MAX values for `NaiveTime`, `NaiveDateTime` and `DateTime<Utc>`.

## 0.4.13

### Features

* Add `DurationRound` trait that allows rounding and truncating by `Duration` (@robyoung)

### Internal Improvements

* Code improvements to impl `From` for `js_sys` in wasm to reuse code (@schrieveslaach)

## 0.4.12

### New Methods and impls

* `Duration::abs` to ensure that a duration is just a magnitude (#418 @abreis).

### Compatibility improvements

* impl `From` for `js_sys` in wasm (#424 @schrieveslaach)
* Bump required version of `time` for redox support.

### Bugfixes

* serde modules do a better job with `Option` types (#417 @mwkroening and #429
  @fx-kirin)
* Use js runtime when using wasmbind to get the local offset (#412
  @quodlibetor)

### Internal Improvements

* Migrate to github actions from travis-ci, make the overall CI experience more comprehensible,
  significantly faster and more correct (#439 @quodlibetor)

## 0.4.11

### Improvements

* Support a space or `T` in `FromStr` for `DateTime<Tz>`, meaning that e.g.
  `dt.to_string().parse::<DateTime<Utc>>()` now correctly works on round-trip.
  (@quodlibetor in #378)
* Support "negative UTC" in `parse_from_rfc2822` (@quodlibetor #368 reported in
  #102)
* Support comparisons of DateTimes with different timezones (@dlalic in #375)
* Many documentation improvements

### Bitrot and external integration fixes

* Don't use wasmbind on wasi (@coolreader18 #365)
* Avoid deprecation warnings for `Error::description` (@AnderEnder and
  @quodlibetor #376)

### Internal improvements

* Use Criterion for benchmarks (@quodlibetor)

## 0.4.10

### Compatibility notes

* Putting some functionality behind an `alloc` feature to improve no-std
  support (in #341) means that if you were relying on chrono with
  `no-default-features` *and* using any of the functions that require alloc
  support (i.e. any of the string-generating functions like `to_rfc3339`) you
  will need to add the `alloc` feature in your Cargo.toml.

### Improvements

* `DateTime::parse_from_str` is more than 2x faster in some cases. (@michalsrb
  #358)
* Significant improvements to no-std and alloc support (This should also make
  many format/serialization operations induce zero unnecessary allocations)
  (@CryZe #341)

### Features

* Functions that were accepting `Iterator` of `Item`s (for example
  `format_with_items`) now accept `Iterator` of `Borrow<Item>`, so one can
  use values or references. (@michalsrb #358)
* Add built-in support for structs with nested `Option<Datetime>` etc fields
  (@manifest #302)

### Internal/doc improvements

* Use markdown footnotes on the `strftime` docs page (@qudlibetor #359)
* Migrate from `try!` -> `?` (question mark) because it is now emitting
  deprecation warnings and has been stable since rustc 1.13.0
* Deny dead code

## 0.4.9

### Fixes

* Make Datetime arithmatic adjust their offsets after discovering their new
  timestamps (@quodlibetor #337)
* Put wasm-bindgen related code and dependencies behind a `wasmbind` feature
  gate. (@quodlibetor #335)

## 0.4.8

### Fixes

* Add '0' to single-digit days in rfc2822 date format (@wyhaya #323)
* Correctly pad DelayedFormat (@SamokhinIlya #320)

### Features

* Support `wasm-unknown-unknown` via wasm-bindgen (in addition to
  emscripten/`wasm-unknown-emscripten`). (finished by @evq in #331, initial
  work by @jjpe #287)

## 0.4.7

### Fixes

* Disable libc default features so that CI continues to work on rust 1.13
* Fix panic on negative inputs to timestamp_millis (@cmars #292)
* Make `LocalResult` `Copy/Eq/Hash`

### Features

* Add `std::convert::From` conversions between the different timezone formats
  (@mqudsi #271)
* Add `timestamp_nanos` methods (@jean-airoldie #308)
* Documentation improvements

## 0.4.6

### Maintenance

* Doc improvements -- improve README CI verification, external links
* winapi upgrade to 0.3

## Unreleased

### Features

* Added `NaiveDate::from_weekday_of_month{,_opt}` for getting eg. the 2nd Friday of March 2017.

## 0.4.5

### Features

* Added several more serde deserialization helpers (@novacrazy #258)
* Enabled all features on the playground (@davidtwco #267)
* Derive `Hash` on `FixedOffset` (@LuoZijun #254)
* Improved docs (@storyfeet #261, @quodlibetor #252)

## 0.4.4

### Features

* Added support for parsing nanoseconds without the leading dot (@emschwartz #251)

## 0.4.3

### Features

* Added methods to DateTime/NaiveDateTime to present the stored value as a number
  of nanoseconds since the UNIX epoch (@harkonenbade #247)
* Added a serde serialise/deserialise module for nanosecond timestamps. (@harkonenbade #247)
* Added "Permissive" timezone parsing which allows a numeric timezone to
  be specified without minutes. (@quodlibetor #242)

## 0.4.2

### Deprecations

* More strongly deprecate RustcSerialize: remove it from documentation unless
  the feature is enabled, issue a deprecation warning if the rustc-serialize
  feature is enabled (@quodlibetor #174)

### Features

* Move all uses of the system clock behind a `clock` feature, for use in
  environments where we don't have access to the current time. (@jethrogb #236)
* Implement subtraction of two `Date`s, `Time`s, or `DateTime`s, returning a
  `Duration` (@tobz1000 #237)

## 0.4.1

### Bug Fixes

* Allow parsing timestamps with subsecond precision (@jonasbb)
* RFC2822 allows times to not include the second (@upsuper)

### Features

* New `timestamp_millis` method on `DateTime` and `NaiveDateTim` that returns
  number of milliseconds since the epoch. (@quodlibetor)
* Support exact decimal width on subsecond display for RFC3339 via a new
  `to_rfc3339_opts` method on `DateTime` (@dekellum)
* Use no_std-compatible num dependencies (@cuviper)
* Add `SubsecRound` trait that allows rounding to the nearest second
  (@dekellum)

### Code Hygiene and Docs

* Docs! (@alatiera @kosta @quodlibetor @kennytm)
* Run clippy and various fixes (@quodlibetor)

## 0.4.0 (2017-06-22)

This was originally planned as a minor release but was pushed to a major
release due to the compatibility concern raised.

### Added

- `IsoWeek` has been added for the ISO week without time zone.

- The `+=` and `-=` operators against `time::Duration` are now supported for
  `NaiveDate`, `NaiveTime` and `NaiveDateTime`. (#99)

  (Note that this does not invalidate the eventual deprecation of `time::Duration`.)

- `SystemTime` and `DateTime<Tz>` types can be now converted to each other via `From`.
  Due to the obvious lack of time zone information in `SystemTime`,
  the forward direction is limited to `DateTime<Utc>` and `DateTime<Local>` only.

### Changed

- Intermediate implementation modules have been flattened (#161),
  and `UTC` has been renamed to `Utc` in accordance with the current convention (#148).

  The full list of changes is as follows:

  Before                                   | After
  ---------------------------------------- | ----------------------------
  `chrono::date::Date`                     | `chrono::Date`
  `chrono::date::MIN`                      | `chrono::MIN_DATE`
  `chrono::date::MAX`                      | `chrono::MAX_DATE`
  `chrono::datetime::DateTime`             | `chrono::DateTime`
  `chrono::naive::time::NaiveTime`         | `chrono::naive::NaiveTime`
  `chrono::naive::date::NaiveDate`         | `chrono::naive::NaiveDate`
  `chrono::naive::date::MIN`               | `chrono::naive::MIN_DATE`
  `chrono::naive::date::MAX`               | `chrono::naive::MAX_DATE`
  `chrono::naive::datetime::NaiveDateTime` | `chrono::naive::NaiveDateTime`
  `chrono::offset::utc::UTC`               | `chrono::offset::Utc`
  `chrono::offset::fixed::FixedOffset`     | `chrono::offset::FixedOffset`
  `chrono::offset::local::Local`           | `chrono::offset::Local`
  `chrono::format::parsed::Parsed`         | `chrono::format::Parsed`

  With an exception of `Utc`, this change does not affect any direct usage of
  `chrono::*` or `chrono::prelude::*` types.

- `Datelike::isoweekdate` is replaced by `Datelike::iso_week` which only returns the ISO week.

  The original method used to return a tuple of year number, week number and day of the week,
  but this duplicated the `Datelike::weekday` method and it had been hard to deal with
  the raw year and week number for the ISO week date.
  This change isolates any logic and API for the week date into a separate type.

- `NaiveDateTime` and `DateTime` can now be deserialized from an integral UNIX timestamp. (#125)

  This turns out to be very common input for web-related usages.
  The existing string representation is still supported as well.

- `chrono::serde` and `chrono::naive::serde` modules have been added
  for the serialization utilities. (#125)

  Currently they contain the `ts_seconds` modules that can be used to
  serialize `NaiveDateTime` and `DateTime` values into an integral UNIX timestamp.
  This can be combined with Serde's `[de]serialize_with` attributes
  to fully support the (de)serialization to/from the timestamp.

  For rustc-serialize, there are separate `chrono::TsSeconds` and `chrono::naive::TsSeconds` types
  that are newtype wrappers implementing different (de)serialization logics.
  This is a suboptimal API, however, and it is strongly recommended to migrate to Serde.

### Fixed

- The major version was made to fix the broken Serde dependency issues. (#146, #156, #158, #159)

  The original intention to technically break the dependency was
  to facilitate the use of Serde 1.0 at the expense of temporary breakage.
  Whether this was appropriate or not is quite debatable,
  but it became clear that there are several high-profile crates requiring Serde 0.9
  and it is not feasible to force them to use Serde 1.0 anyway.

  To the end, the new major release was made with some known lower-priority breaking changes.
  0.3.1 is now yanked and any remaining 0.3 users can safely roll back to 0.3.0.

- Various documentation fixes and goodies. (#92, #131, #136)

## 0.3.1 (2017-05-02)

### Added

- `Weekday` now implements `FromStr`, `Serialize` and `Deserialize`. (#113)

  The syntax is identical to `%A`, i.e. either the shortest or the longest form of English names.

### Changed

- Serde 1.0 is now supported. (#142)

  This is technically a breaking change because Serde 0.9 and 1.0 are not compatible,
  but this time we decided not to issue a minor version because
  we have already seen Serde 0.8 and 0.9 compatibility problems even after 0.3.0 and
  a new minor version turned out to be not very helpful for this kind of issues.

### Fixed

- Fixed a bug that the leap second can be mapped wrongly in the local time zone.
  Only occurs when the local time zone is behind UTC. (#130)

## 0.3.0 (2017-02-07)

The project has moved to the [Chronotope](https://github.com/chronotope/) organization.

### Added

- `chrono::prelude` module has been added. All other glob imports are now discouraged.

- `FixedOffset` can be added to or subtracted from any timelike types.

    - `FixedOffset::local_minus_utc` and `FixedOffset::utc_minus_local` methods have been added.
      Note that the old `Offset::local_minus_utc` method is gone; see below.

- Serde support for non-self-describing formats like Bincode is added. (#89)

- Added `Item::Owned{Literal,Space}` variants for owned formatting items. (#76)

- Formatting items and the `Parsed` type have been slightly adjusted so that
  they can be internally extended without breaking any compatibility.

- `Weekday` is now `Hash`able. (#109)

- `ParseError` now implements `Eq` as well as `PartialEq`. (#114)

- More documentation improvements. (#101, #108, #112)

### Changed

- Chrono now only supports Rust 1.13.0 or later (previously: Rust 1.8.0 or later).

- Serde 0.9 is now supported.
  Due to the API difference, support for 0.8 or older is discontinued. (#122)

- Rustc-serialize implementations are now on par with corresponding Serde implementations.
  They both standardize on the `std::fmt::Debug` textual output.

  **This is a silent breaking change (hopefully the last though).**
  You should be prepared for the format change if you depended on rustc-serialize.

- `Offset::local_minus_utc` is now `Offset::fix`, and returns `FixedOffset` instead of a duration.

  This makes every time zone operation operate within a bias less than one day,
  and vastly simplifies many logics.

- `chrono::format::format` now receives `FixedOffset` instead of `time::Duration`.

- The following methods and implementations have been renamed and older names have been *removed*.
  The older names will be reused for the same methods with `std::time::Duration` in the future.

    - `checked_*` → `checked_*_signed` in `Date`, `DateTime`, `NaiveDate` and `NaiveDateTime` types

    - `overflowing_*` → `overflowing_*_signed` in the `NaiveTime` type

    - All subtraction implementations between two time instants have been moved to
      `signed_duration_since`, following the naming in `std::time`.

### Fixed

- Fixed a panic when the `Local` offset receives a leap second. (#123)

### Removed

- Rustc-serialize support for `Date<Tz>` types and all offset types has been dropped.

  These implementations were automatically derived and never had been in a good shape.
  Moreover there are no corresponding Serde implementations, limiting their usefulness.
  In the future they may be revived with more complete implementations.

- The following method aliases deprecated in the 0.2 branch have been removed.

    - `DateTime::num_seconds_from_unix_epoch` (→ `DateTime::timestamp`)
    - `NaiveDateTime::from_num_seconds_from_unix_epoch` (→ `NaiveDateTime::from_timestamp`)
    - `NaiveDateTime::from_num_seconds_from_unix_epoch_opt` (→ `NaiveDateTime::from_timestamp_opt`)
    - `NaiveDateTime::num_seconds_unix_epoch` (→ `NaiveDateTime::timestamp`)

- Formatting items are no longer `Copy`, except for `chrono::format::Pad`.

- `chrono::offset::add_with_leapsecond` has been removed.
  Use a direct addition with `FixedOffset` instead.

## 0.2.25 (2016-08-04)

This is the last version officially supports Rust 1.12.0 or older.

(0.2.24 was accidentally uploaded without a proper check for warnings in the default state,
and replaced by 0.2.25 very shortly. Duh.)

### Added

- Serde 0.8 is now supported. 0.7 also remains supported. (#86)

### Fixed

- The deserialization implementation for rustc-serialize now properly verifies the input.
  All serialization codes are also now thoroughly tested. (#42)

## 0.2.23 (2016-08-03)

### Added

- The documentation was greatly improved for several types,
  and tons of cross-references have been added. (#77, #78, #80, #82)

- `DateTime::timestamp_subsec_{millis,micros,nanos}` methods have been added. (#81)

### Fixed

- When the system time records a leap second,
  the nanosecond component was mistakenly reset to zero. (#84)

- `Local` offset misbehaves in Windows for August and later,
  due to the long-standing libtime bug (dates back to mid-2015).
  Workaround has been implemented. (#85)

## 0.2.22 (2016-04-22)

### Fixed

- `%.6f` and `%.9f` used to print only three digits when the nanosecond part is zero. (#71)
- The documentation for `%+` has been updated to reflect the current status. (#71)

## 0.2.21 (2016-03-29)

### Fixed

- `Fixed::LongWeekdayName` was unable to recognize `"sunday"` (whoops). (#66)

## 0.2.20 (2016-03-06)

### Changed

- `serde` dependency has been updated to 0.7. (#63, #64)

## 0.2.19 (2016-02-05)

### Added

- The documentation for `Date` is made clear about its ambiguity and guarantees.

### Fixed

- `DateTime::date` had been wrong when the local date and the UTC date is in disagreement. (#61)

## 0.2.18 (2016-01-23)

### Fixed

- Chrono no longer pulls a superfluous `rand` dependency. (#57)

## 0.2.17 (2015-11-22)

### Added

- Naive date and time types and `DateTime` now have a `serde` support.
  They serialize as an ISO 8601 / RFC 3339 string just like `Debug`. (#51)

## 0.2.16 (2015-09-06)

### Added

- Added `%.3f`, `%.6f` and `%.9f` specifier for formatting fractional seconds
  up to 3, 6 or 9 decimal digits. This is a natural extension to the existing `%f`.
  Note that this is (not yet) generic, no other value of precision is supported. (#45)

### Changed

- Forbade unsized types from implementing `Datelike` and `Timelike`.
  This does not make a big harm as any type implementing them should be already sized
  to be practical, but this change still can break highly generic codes. (#46)

### Fixed

- Fixed a broken link in the `README.md`. (#41)

## 0.2.15 (2015-07-05)

### Added

- Padding modifiers `%_?`, `%-?` and `%0?` are implemented.
  They are glibc extensions which seem to be reasonably widespread (e.g. Ruby).

- Added `%:z` specifier and corresponding formatting items
  which is essentially the same as `%z` but with a colon.

- Added a new specifier `%.f` which precision adapts from the input.
  This was added as a response to the UX problems in the original nanosecond specifier `%f`.

### Fixed

- `Numeric::Timestamp` specifier (`%s`) was ignoring the time zone offset when provided.

- Improved the documentation and associated tests for `strftime`.

## 0.2.14 (2015-05-15)

### Fixed

- `NaiveDateTime +/- Duration` or `NaiveTime +/- Duration` could have gone wrong
  when the `Duration` to be added is negative and has a fractional second part.
  This was caused by an underflow in the conversion from `Duration` to the parts;
  the lack of tests for this case allowed a bug. (#37)

## 0.2.13 (2015-04-29)

### Added

- The optional dependency on `rustc_serialize` and
  relevant `Rustc{En,De}codable` implementations for supported types has been added.
  This is enabled by the `rustc-serialize` Cargo feature. (#34)

### Changed

- `chrono::Duration` reexport is changed to that of crates.io `time` crate.
  This enables Rust 1.0 beta compatibility.

## 0.2.4 (2015-03-03)

### Fixed

- Clarified the meaning of `Date<Tz>` and fixed unwanted conversion problem
  that only occurs with positive UTC offsets. (#27)

## 0.2.3 (2015-02-27)

### Added

- `DateTime<Tz>` and `Date<Tz>` is now `Copy`/`Send` when `Tz::Offset` is `Copy`/`Send`.
  The implementations for them were mistakenly omitted. (#25)

### Fixed

- `Local::from_utc_datetime` didn't set a correct offset. (#26)

## 0.2.1 (2015-02-21)

### Changed

- `DelayedFormat` no longer conveys a redundant lifetime.

## 0.2.0 (2015-02-19)

### Added

- `Offset` is splitted into `TimeZone` (constructor) and `Offset` (storage) types.
  You would normally see only the former, as the latter is mostly an implementation detail.
  Most importantly, `Local` now can be used to directly construct timezone-aware values.

  Some types (currently, `UTC` and `FixedOffset`) are both `TimeZone` and `Offset`,
  but others aren't (e.g. `Local` is not what is being stored to each `DateTime` values).

- `LocalResult::map` convenience method has been added.

- `TimeZone` now allows a construction of `DateTime` values from UNIX timestamp,
  via `timestamp` and `timestamp_opt` methods.

- `TimeZone` now also has a method for parsing `DateTime`, namely `datetime_from_str`.

- The following methods have been added to all date and time types:

    - `checked_add`
    - `checked_sub`
    - `format_with_items`

- The following methods have been added to all timezone-aware types:

    - `timezone`
    - `with_timezone`
    - `naive_utc`
    - `naive_local`

- `parse_from_str` method has been added to all naive types and `DateTime<FixedOffset>`.

- All naive types and instances of `DateTime` with time zones `UTC`, `Local` and `FixedOffset`
  implement the `FromStr` trait. They parse what `std::fmt::Debug` would print.

- `chrono::format` has been greatly rewritten.

    - The formatting syntax parser is modular now, available at `chrono::format::strftime`.

    - The parser and resolution algorithm is also modular, the former is available at
      `chrono::format::parse` while the latter is available at `chrono::format::parsed`.

    - Explicit support for RFC 2822 and 3339 syntaxes is landed.

    - There is a minor formatting difference with atypical values,
      e.g. for years not between 1 BCE and 9999 CE.

### Changed

- Most uses of `Offset` are converted to `TimeZone`.
  In fact, *all* user-facing code is expected to be `Offset`-free.

- `[Naive]DateTime::*num_seconds_from_unix_epoch*` methods have been renamed to
  simply `timestamp` or `from_timestamp*`. The original names have been deprecated.

### Removed

- `Time` has been removed. This also prompts a related set of methods in `TimeZone`.

  This is in principle possible, but in practice has seen a little use
  because it can only be meaningfully constructed via an existing `DateTime` value.
  This made many operations to `Time` unintuitive or ambiguous,
  so we simply let it go.

  In the case that `Time` is really required, one can use a simpler `NaiveTime`.
  `NaiveTime` and `NaiveDate` can be freely combined and splitted,
  and `TimeZone::from_{local,utc}_datetime` can be used to convert from/to the local time.

- `with_offset` method has been removed. Use `with_timezone` method instead.
  (This is not deprecated since it is an integral part of offset reform.)

## 0.1.14 (2015-01-10)

### Added

- Added a missing `std::fmt::String` impl for `Local`.

## 0.1.13 (2015-01-10)

### Changed

- Most types now implement both `std::fmt::Show` and `std::fmt::String`,
  with the former used for the stricter output and the latter used for more casual output.

### Removed

- `Offset::name` has been replaced by a `std::fmt::String` implementation to `Offset`.

## 0.1.12 (2015-01-08)

### Removed

- `Duration + T` no longer works due to the updated impl reachability rules.
  Use `T + Duration` as a workaround.

## 0.1.4 (2014-12-13)

### Fixed

- Fixed a bug that `Date::and_*` methods with an offset that can change the date are
  off by one day.

## 0.1.3 (2014-11-28)

### Added

- `{Date,Time,DateTime}::with_offset` methods have been added.

- `LocalResult` now implements a common set of traits.

- `LocalResult::and_*` methods have been added.
  They are useful for safely chaining `LocalResult<Date<Off>>` methods
  to make `LocalResult<DateTime<Off>>`.

### Changed

- `Offset::name` now returns `SendStr`.

- `{Date,Time} - Duration` overloadings are now allowed.

## 0.1.2 (2014-11-24)

### Added

- `Duration + Date` overloading is now allowed.

### Changed

- Chrono no longer needs `num` dependency.

## 0.1.0 (2014-11-20)

The initial version that was available to `crates.io`.
