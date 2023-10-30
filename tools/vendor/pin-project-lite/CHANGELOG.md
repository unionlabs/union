# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.2.9] - 2022-04-26

- Improve compile time of `pin_project!` calls. (#71, thanks @nnethercote)

## [0.2.8] - 2021-12-31

- Fix handling of trailing commas in `PinnedDrop` impl. ([#64](https://github.com/taiki-e/pin-project-lite/pull/64), thanks @Michael-J-Ward)

## [0.2.7] - 2021-06-26

- [Support custom Drop implementation.](https://github.com/taiki-e/pin-project-lite/pull/25) See [#25](https://github.com/taiki-e/pin-project-lite/pull/25) for details.

## [0.2.6] - 2021-03-04

- Support item attributes in any order. ([#57](https://github.com/taiki-e/pin-project-lite/pull/57), thanks @SabrinaJewson)

## [0.2.5] - 2021-03-02

- [Prepare for removal of `safe_packed_borrows` lint.](https://github.com/taiki-e/pin-project-lite/pull/55) See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

## [0.2.4] - 2021-01-11

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- Add `project_replace`. ([#43](https://github.com/taiki-e/pin-project-lite/pull/43), thanks @Marwes)

## [0.2.3] - 2021-01-09

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Suppress `clippy::unknown_clippy_lints` lint in generated code.](https://github.com/taiki-e/pin-project-lite/pull/47)

## [0.2.2] - 2021-01-09

**Note:** This release has been yanked.** See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Suppress `clippy::ref_option_ref` lint in generated code.](https://github.com/taiki-e/pin-project-lite/pull/45)

## [0.2.1] - 2021-01-05

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- Exclude unneeded files from crates.io.

## [0.2.0] - 2020-11-13

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [`pin_project!` macro now supports enums.](https://github.com/taiki-e/pin-project-lite/pull/28)

  To use `pin_project!` on enums, you need to name the projection type returned from the method.

  ```rust
  use pin_project_lite::pin_project;
  use std::pin::Pin;

  pin_project! {
      #[project = EnumProj]
      enum Enum<T, U> {
          Variant { #[pin] pinned: T, unpinned: U },
      }
  }

  impl<T, U> Enum<T, U> {
      fn method(self: Pin<&mut Self>) {
          match self.project() {
              EnumProj::Variant { pinned, unpinned } => {
                  let _: Pin<&mut T> = pinned;
                  let _: &mut U = unpinned;
              }
          }
      }
  }
  ```

- [Support naming the projection types.](https://github.com/taiki-e/pin-project-lite/pull/28)

  By passing an attribute with the same name as the method, you can name the projection type returned from the method:

  ```rust
  use pin_project_lite::pin_project;
  use std::pin::Pin;

  pin_project! {
      #[project = StructProj]
      struct Struct<T> {
          #[pin]
          field: T,
      }
  }

  fn func<T>(x: Pin<&mut Struct<T>>) {
      let StructProj { field } = x.project();
      let _: Pin<&mut T> = field;
  }
  ```

## [0.1.12] - 2021-03-02

- [Prepare for removal of `safe_packed_borrows` lint.](https://github.com/taiki-e/pin-project-lite/pull/55) See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

## [0.1.11] - 2020-10-20

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- Suppress `clippy::redundant_pub_crate` lint in generated code.

- Documentation improvements.

## [0.1.10] - 2020-10-01

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- Suppress `drop_bounds` lint, which will be added to rustc in the future. See [taiki-e/pin-project#272](https://github.com/taiki-e/pin-project/issues/272) for more details.

## [0.1.9] - 2020-09-29

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Fix trailing comma support in generics.](https://github.com/taiki-e/pin-project-lite/pull/32)

## [0.1.8] - 2020-09-26

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Fix compatibility of generated code with `forbid(future_incompatible)`.](https://github.com/taiki-e/pin-project-lite/pull/30)

  Note: This does not guarantee compatibility with `forbid(future_incompatible)` in the future.
  If rustc adds a new lint, we may not be able to keep this.

## [0.1.7] - 2020-06-04

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Support `?Sized` bounds in where clauses.](https://github.com/taiki-e/pin-project-lite/pull/22)

- [Fix lifetime inference error when an associated type is used in fields.](https://github.com/taiki-e/pin-project-lite/pull/20)

- Suppress `clippy::used_underscore_binding` lint in generated code.

- Documentation improvements.

## [0.1.6] - 2020-05-31

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Support lifetime bounds in where clauses.](https://github.com/taiki-e/pin-project-lite/pull/18)

- Documentation improvements.

## [0.1.5] - 2020-05-07

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Support overwriting the name of `core` crate.](https://github.com/taiki-e/pin-project-lite/pull/14)

## [0.1.4] - 2020-01-20

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Support ?Sized bounds in generic parameters.](https://github.com/taiki-e/pin-project-lite/pull/9)

## [0.1.3] - 2020-01-20

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [Support lifetime bounds in generic parameters.](https://github.com/taiki-e/pin-project-lite/pull/7)

## [0.1.2] - 2020-01-05

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- Support recognizing default generic parameters. ([#6](https://github.com/taiki-e/pin-project-lite/pull/6), thanks @kennytm)

## [0.1.1] - 2019-11-15

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

- [`pin_project!` macro now determines the visibility of the projection type/method is based on the original type.](https://github.com/taiki-e/pin-project-lite/pull/5)

## [0.1.0] - 2019-10-22

**Note:** This release has been yanked. See [#55](https://github.com/taiki-e/pin-project-lite/pull/55) for details.

Initial release

[Unreleased]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.9...HEAD
[0.2.9]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/taiki-e/pin-project-lite/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.11...v0.2.0
[0.1.12]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.11...v0.1.12
[0.1.11]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/pin-project-lite/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/pin-project-lite/releases/tag/v0.1.0
