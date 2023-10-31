# Changelog

## Unreleased

## 0.15.0

### Enhancements

* Addeed more key recognitions
* Exposed `pad_str_with` to public API
* Added `ReadWritePair`
* Support `color256` in `Style::from_dotted_str`

### BREAKING

* Added `ReadWritePair` to `TermTarget` to allow arbitrary read write pairs behave as a term
* Removed `Copy` and `PartialEq` from `TermTarget`

## 0.14.1

### Enhancements

* Added `NO_COLOR` support
* Added some more key recognitions
* Undeprecate `Term::is_term`

## 0.14.0

### Enhancements

* Added emoji support for newer Windows terminals.

### BREAKING

* Made the windows terminal emulation a non default feature (`windows-console-colors`)

## 0.13.0

### Enhancements

* Added `user_attended_stderr` for checking if stderr is a terminal
* Removed `termios` dependency

### Bug Fixes

* Better handling of key recognition on unix
* `Term::terminal_size()` on stderr terms correctly returns stderr term info

### Deprecated

* Deprecate `Term::is_term()` in favor of `Term::features().is_attended()`

### BREAKING

* Remove `Term::want_emoji()` in favor of `Term::features().wants_emoji()`
