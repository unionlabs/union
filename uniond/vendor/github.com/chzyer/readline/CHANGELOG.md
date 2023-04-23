# ChangeLog

### 1.4 - 2016-07-25

* [#60][60] Support dynamic autocompletion
* Fix ANSI parser on Windows
* Fix wrong column width in complete mode on Windows
* Remove dependent package "golang.org/x/crypto/ssh/terminal"

### 1.3 - 2016-05-09

* [#38][38] add SetChildren for prefix completer interface
* [#42][42] improve multiple lines compatibility
* [#43][43] remove sub-package(runes) for gopkg compatibility
* [#46][46] Auto complete with space prefixed line
* [#48][48]	support suspend process (ctrl+Z)
* [#49][49] fix bug that check equals with previous command
* [#53][53] Fix bug which causes integer divide by zero panicking when input buffer is empty

### 1.2 - 2016-03-05

* Add a demo for checking password strength [example/readline-pass-strength](https://github.com/chzyer/readline/blob/master/example/readline-pass-strength/readline-pass-strength.go), , written by [@sahib](https://github.com/sahib)
* [#23][23], support stdin remapping
* [#27][27], add a `UniqueEditLine` to `Config`, which will erase the editing line after user submited it, usually use in IM.
* Add a demo for multiline [example/readline-multiline](https://github.com/chzyer/readline/blob/master/example/readline-multiline/readline-multiline.go) which can submit one SQL by multiple lines.
* Supports performs even stdin/stdout is not a tty.
* Add a new simple apis for single instance, check by [here](https://github.com/chzyer/readline/blob/master/std.go). It need to save history manually if using this api.
* [#28][28], fixes the history is not working as expected.
* [#33][33], vim mode now support `c`, `d`, `x (delete character)`, `r (replace character)`

### 1.1 - 2015-11-20

* [#12][12] Add support for key `<Delete>`/`<Home>`/`<End>`
* Only enter raw mode as needed (calling `Readline()`), program will receive signal(e.g. Ctrl+C) if not interact with `readline`.
* Bugs fixed for `PrefixCompleter`
* Press `Ctrl+D` in empty line will cause `io.EOF` in error, Press `Ctrl+C` in anytime will cause `ErrInterrupt` instead of `io.EOF`, this will privodes a shell-like user experience.
* Customable Interrupt/EOF prompt in `Config`
* [#17][17] Change atomic package to use 32bit function to let it runnable on arm 32bit devices
* Provides a new password user experience(`readline.ReadPasswordEx()`).

### 1.0 - 2015-10-14

* Initial public release.

[12]: https://github.com/chzyer/readline/pull/12
[17]: https://github.com/chzyer/readline/pull/17
[23]: https://github.com/chzyer/readline/pull/23
[27]: https://github.com/chzyer/readline/pull/27
[28]: https://github.com/chzyer/readline/pull/28
[33]: https://github.com/chzyer/readline/pull/33
[38]: https://github.com/chzyer/readline/pull/38
[42]: https://github.com/chzyer/readline/pull/42
[43]: https://github.com/chzyer/readline/pull/43
[46]: https://github.com/chzyer/readline/pull/46
[48]: https://github.com/chzyer/readline/pull/48
[49]: https://github.com/chzyer/readline/pull/49
[53]: https://github.com/chzyer/readline/pull/53
[60]: https://github.com/chzyer/readline/pull/60
