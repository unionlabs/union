clipboard-win
====================

![Build](https://github.com/DoumanAsh/clipboard-win/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/clipboard-win.svg)](https://crates.io/crates/clipboard-win)
[![Docs.rs](https://docs.rs/clipboard-win/badge.svg)](https://docs.rs/clipboard-win/*/x86_64-pc-windows-msvc/clipboard_win/)

This crate provide simple means to operate with Windows clipboard.

# Note keeping Clipboard around:

In Windows [Clipboard](struct.Clipboard.html) opens globally and only one application can set data onto format at the time.

Therefore as soon as operations are finished, user is advised to close [Clipboard](struct.Clipboard.html).

# Clipboard

All read and write access to Windows clipboard requires user to open it.

# Usage

## Manually lock clipboard

```rust
use clipboard_win::{Clipboard, formats, Getter, Setter};

const SAMPLE: &str = "MY loli sample ^^";

let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
formats::Unicode.write_clipboard(&SAMPLE).expect("Write sample");

let mut output = String::new();

assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
assert_eq!(output, SAMPLE);

//Efficiently re-use buffer ;)
output.clear();
assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
assert_eq!(output, SAMPLE);

//Or take the same string twice?
assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
assert_eq!(format!("{0}{0}", SAMPLE), output);

```

## Simplified API

```rust
use clipboard_win::{formats, get_clipboard, set_clipboard};

let text = "my sample ><";

set_clipboard(formats::Unicode, text).expect("To set clipboard");
//Type is necessary as string can be stored in various storages
let result: String = get_clipboard(formats::Unicode).expect("To set clipboard");
assert_eq!(result, text)
```
