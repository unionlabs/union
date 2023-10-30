Version 0.3.6 (2019-08-23)
==========================
* Fix pointless undefined behavior in `Utf16Char.to_ascii_char()` (which is part of ascii feature)
* Widen ascii version requirement to include 1.*
* Add `[u16; 2]` UTF-16 array alternatives to `(u16, Some(u16))` UTF-16 tuple methods
* Add `Utf16Char.is_bmp()`

Version 0.3.5 (2018-10-23)
==========================
* Fix docs.rs build failure

Version 0.3.4 (2018-10-23)
==========================
* Fix UB in UTF-8 validation which lead to invalid codepoints being accepted in release mode
* Add fallible decoding iterator adapters `Utf8CharMerger` and `Utf16CharMerger`
  and slice-based iterators `Utf8CharDecoder` and `Utf16CharDecoder`
* Widen ascii version requirement from 0.8.* to 0.8.0 - 0.10.*
* Implement creating / extending `String`s from `Utf16Char`-producing iterators

Version 0.3.3 (2018-10-16)
==========================
* Fix UTF-8 overlong check. (`from_array()` and `from_slice()` accepted two-byte encodings of ASCII characters >= '@', which includes all letters)
* Implement `FromStr` for `Utf16Char`
* Add `from_str_start()` to `Utf8Char` and `Utf16Char`
* Add `Utf{8,16}Char{s,Indices}`: `str`-based iterators for `Utf8Char` and `Utf16Char` equivalent to `char`'s `Chars` and `CharIndices`.
* Add `StrExt` with functions to create the above iterators.
* Implement `FromIterator` and `Extend` for `Vec<{u8,u16}>` with reference-producing `Utf{8,16}Char` iterators too.
* Add `Utf8CharSplitter` and `Utf16CharSplitter`: `Utf{8,16}Char`-to-`u{8,16}` iterator adapters.
* Add `IterExt`, `iter_bytes()` and `iter_units()` to create the above splitting iterators.
* Add `Utf8Char::from_ascii()`, `Utf16Char::from_bmp()` with `_unchecked` versions of both.
* Add cross-type `PartialEq` and `PartialOrd` implementations.
* Change the `description()` for a few error types.

Version 0.3.2 (2018-08-08)
==========================
* Hide `AsciiExt` deprecation warning and add replacement methods.
* Correct documentation for `U8UtfExt::extra_utf8_bytes()`.
* Fix misspellings in some error descriptions.
* Avoid potentially bad transmutes.

Version 0.3.1 (2017-06-16)
==========================
* Implement `Display` for `Utf8Char` and `Utf16Char`.

Version 0.3.0 (2017-03-29)
==========================
* Replace the "no_std" feature with opt-out "std".
  * Upgrade ascii to v0.8.
  * Make tests compile on stable.
* Remove `CharExt::write_utf{8,16}()` because `encode_utf{8,16}()` has been stabilized.
* Return a proper error from `U16UtfExt::utf16_needs_extra_unit()` instead of `None`.
* Rename `U16UtfExt::utf_is_leading_surrogate()` to `is_utf16_leading_surrogate()`.
* Rename `Utf16Char::from_slice()` to `from_slice_start()`  and `CharExt::from_utf{8,16}_slice()`
  to `from_utf{8,16}_slice_start()` to be consistent with `Utf8Char`.
* Fix a bug where `CharExt::from_slice()` would accept some trailing surrogates
  as standalone codepoints.

Version 0.2.0 (2016-07-24)
==========================
* Change `CharExt::write_utf{8,16}()` to panic instead of returning `None`
  if the slice is too short.
* Fix bug where `CharExt::write_utf8()` and `Utf8Char::to_slice()` could change bytes it shouldn't.
* Rename lots of errors with search and replace:
  * CodePoint -> Codepoint
  * Several -> Multiple
* Update the ascii feature to use [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html) v0.7.
* Support `#[no_std]`; see 70e090ee for differences.
* Ungate impls of `AsciiExt`. (doesn't require ascii or nightly)
* Make the tests compile (and pass) again.
  (They still require nightly).

Version 0.1.* (2016-04-07)
==========================
First release.
