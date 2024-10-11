# Flatbuffers Change Log

All major or breaking changes will be documented in this file, as well as any
new features that should be highlighted. Minor fixes or improvements are not
necessarily listed.

## [24.3.25] (March 25 2024)(https://github.com/google/flatbuffers/releases/tag/v24.3.25)

* Fixed license metadata parsing (#8253)
* [C++] Allow string_view in `LookUpByKey` in addition to null-terminated c-style strings (#8203)

## [24.3.7] (March 7 2024)(https://github.com/google/flatbuffers/releases/tag/v24.3.7)

* Just to fix some of the CI build issues from the 24.3.6 release.

## [24.3.6] (March 6 2024)(https://github.com/google/flatbuffers/releases/tag/v24.3.6)

* Fix typescript object API to allow 0 values for null-default scalars (#7864)

## [23.5.26 (May 26 2023)](https://github.com/google/flatbuffers/releases/tag/v23.5.26)

* Mostly bug fixing for 64-bit support
* Adds support for specifying underling type of unions in C++ and TS/JS (#7954)

## [23.5.9 (May 9 2023)](https://github.com/google/flatbuffers/releases/tag/v23.5.9)

* 64-bit support for C++ (#7935)

## [23.5.8 (May 8 2023)](https://github.com/google/flatbuffers/releases/tag/v23.5.8)

* add key_field to compiled tests
* Add golden language directory
* Rework cmake flatc codegeneration (#7938)
* remove defining generated files in test srcs
* Add binary schema reflection (#7932)
* Migrate from rules_nodejs to rules_js/rules_ts (take 2) (#7928)
* `flat_buffers.dart`: mark const variable finals for internal Dart linters
* fixed some windows warnings (#7929)
* inject no long for FBS generation to remove logs in flattests (#7926)
* Revert "Migrate from rules_nodejs to rules_js/rules_ts (#7923)" (#7927)
* Migrate from rules_nodejs to rules_js/rules_ts (#7923)
* Only generate @kotlin.ExperimentalUnsigned annotation on create*Vector methods having an unsigned array type parameter. (#7881)
* additional check for absl::string_view availability (#7897)
* Optionally generate Python type annotations (#7858)
* Replace deprecated command with environment file (#7921)
* drop glibc from runtime dependencies (#7906)
* Make JSON supporting advanced union features (#7869)
* Allow to use functions from `BuildFlatBuffers.cmake` from a flatbuffers installation installed with CMake. (#7912)
* TS/JS: Use TypeError instead of Error when appropriate (#7910)
* Go: make generated code more compliant to "go fmt" (#7907)
* Support file_identifier in Go (#7904)
* Optionally generate type prefixes and suffixes for python code (#7857)
* Go: add test for FinishWithFileIdentifier (#7905)
* Fix go_sample.sh (#7903)
* [TS/JS] Upgrade dependencies (#7889)
* Add a FileWriter interface (#7821)
* TS/JS: Use minvalue from enum if not found (#7888)
* [CS] Verifier (#7850)
* README.md: PyPI case typo (#7880)
* Update go documentation link to point to root module (#7879)
* use Bool for flatbuffers bool instead of Byte (#7876)
* fix using null string in vector (#7872)
* Add `flatbuffers-64` branch to CI for pushes
* made changes to the rust docs so they would compile. new_with_capacity is deprecated should use with_capacity, get_root_as_monster should be root_as_monster (#7871)
* Adding comment for code clarification (#7856)
* ToCamelCase() when kLowerCamel now converts first char to lower. (#7838)
* Fix help output for --java-checkerframework (#7854)
* Update filename to README.md and improve formatting (#7855)
* Update stale.yml
* Updated remaining usages of LICENSE.txt

## [23.3.3 (Mar 3 2023)](https://github.com/google/flatbuffers/releases/tag/v23.3.3)

* Refactoring of `flatc` generators to use an interface (#7797).

* Removed legacy cmake support and set min to 3.8 (#7801).

## [23.1.21 (Jan 21 2023)](https://github.com/google/flatbuffers/releases/tag/v23.1.20)

* Reworked entry points for Typescript/Javascript and compatibility for single
  file build (#7510)

## [23.1.20 (Jan 20 2023)](https://github.com/google/flatbuffers/releases/tag/v23.1.20)

* Removed go.mod files after some versioning issues were being report (#7780).

## [23.1.4 (Jan 4 2023)](https://github.com/google/flatbuffers/releases/tag/v23.1.4)

* Major release! Just kidding, we are continuing the
  [versioning scheme](https://github.com/google/flatbuffers/wiki/Versioning) of
  using a date to signify releases. This results in the first release of the new
  year to bump the tradition major version field.

* Go minimum version is now 1.19 (#7720) with the addition of Go modules.

* Added CI support for Big Endian regression testing (#7707).

* Fixed `getFullyQualifiedName` in typescript to return name delimited by '.'
  instead of '_' (#7730).

* Fixed the versioning scheme to not include leading zeros which are not
  consistently handled by every package manager. Only the last release
  (12.12.06) should have suffered from this.

## [22.12.06 (Dec 06 2022)](https://github.com/google/flatbuffers/releases/tag/v22.12.06)

* Bug fixing release, no major changes.

## [22.10.25 (Oct 25 2022)](https://github.com/google/flatbuffers/releases/tag/v22.10.25)

* Added Nim language support with generator and runtime libraries (#7534).

## [22.9.29 (Sept 29 2022)](https://github.com/google/flatbuffers/releases/tag/v22.9.29)

* Rust soundness fixes to avoid the crate from bing labelled unsafe (#7518).

## [22.9.24 (Sept 24 2022)](https://github.com/google/flatbuffers/releases/tag/v22.9.24)

* 20 Major releases in a row? Nope, we switched to a new
  [versioning scheme](https://github.com/google/flatbuffers/wiki/Versioning)
  that is based on date.

* Python supports fixed size arrays now (#7529).

* Behavior change in how C++ object API uses `UnPackTo`. The original intent of
  this was to reduce allocations by reusing an existing object to pack data
  into. At some point, this logic started to merge the states of the two objects
  instead of clearing the state of the packee. This change goes back to the
  original intention, the packed object is cleared when getting data packed into
  it (#7527).

* Fixed a bug in C++ alignment that was using `sizeof()` instead of the intended
  `AlignOf()` for structs (#7520).

* C# has an
  [official Nuget package](https://www.nuget.org/packages/Google.FlatBuffers)
  now (#7496).

## 2.0.8 (Aug 29 2022)

* Fix for `--keep-prefix` the was generating the wrong include statements for
  C++ (#7469). The bug was introduced in 2.0.7.

* Added the `Verifier::Options` option struct to allow specifying runtime
  configuration settings for the verifier (#7489). This allows to skip verifying
  nested flatbuffers, a on-by-default change that was introduced in 2.0.7. This
  deprecates the existing `Verifier` constructor, which may be removed in a
  future version.

* Refactor of `tests/test.cpp` that lead to ~10% speedup in compilation of the
  entire project (#7487).

## 2.0.7 (Aug 22 2022)

* This is the first version with an explicit change log, so all the previous
  features will not be listed.

* Verifier now checks that buffers are at least the minimum size required to be
  a flatbuffers (12 bytes). This includes nested flatbuffers, which previously
  could be declared valid at size 0.

* Annotated binaries. Given a flatbuffer binary and a schema (or binary schema)
  one can generate an annotated flatbuffer (.afb) to describe each byte in the
  binary with schema metadata and value.

* First binary schema generator (Lua) to generate Lua code via a .bfbs file.
  This is mostly an implementation detail of flatc internals, but will be slowly
  applied to the other language generators.
