# 0.2.14

* add support for [RustyHermit](https://github.com/hermitcore/libhermit-rs), a Rust-based unikernel [#41](https://github.com/softprops/atty/pull/41)

# 0.2.13

* support older versions of rust that do now support 2018 edition

# 0.2.12

* Redox is now in the unix family so redox cfg is no longer needed [#35](https://github.com/softprops/atty/pull/35)

# 0.2.11

* fix msys detection with `winapi@0.3.5` [#28](https://github.com/softprops/atty/pull/28)

# 0.2.10

* fix wasm regression [#27](https://github.com/softprops/atty/pull/27)

# 0.2.9

* Fix fix pty detection [#25](https://github.com/softprops/atty/pull/25)

# 0.2.8

* Fix an inverted condition on MinGW [#22](https://github.com/softprops/atty/pull/22)

# 0.2.7

* Change `||` to `&&` for whether MSYS is a tty [#24](https://github.com/softprops/atty/pull/24/)

# 0.2.6

* updated winapi dependency to [0.3](https://retep998.github.io/blog/winapi-0.3/) [#18](https://github.com/softprops/atty/pull/18)

# 0.2.5

* added support for Wasm compile targets [#17](https://github.com/softprops/atty/pull/17)

# 0.2.4

* added support for Wasm compile targets [#17](https://github.com/softprops/atty/pull/17)

# 0.2.3

* added support for Redox OS [#14](https://github.com/softprops/atty/pull/14)

# 0.2.2

* use target specific dependencies [#11](https://github.com/softprops/atty/pull/11)
* Add tty detection for MSYS terminals [#12](https://github.com/softprops/atty/pull/12)

# 0.2.1

* fix windows bug

# 0.2.0

* support for various stream types

# 0.1.2

* windows support (with automated testing)
* automated code coverage

# 0.1.1

* bumped libc dep from `0.1` to `0.2`

# 0.1.0

* initial release
