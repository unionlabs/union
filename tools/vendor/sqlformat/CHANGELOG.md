### Version 0.2.0
- Fix extra spaces in string escaping [#13](https://github.com/shssoichiro/sqlformat-rs/pull/13)
- Fix panic on overflowing integer [#14](https://github.com/shssoichiro/sqlformat-rs/pull/14)
- Bump Rust edition to 2021
  - This is technically a breaking change as it bumps the minimum Rust version to 1.56

### Version 0.1.8

- Remove regex dependency
- Remove unused maplit dependency

### Version 0.1.7

- Bump nom to 7.0, which reportedly also fixes some build issues

### Version 0.1.6

- Fix compatibility with Rust 1.44 which was broken in 0.1.5

### Version 0.1.5

- Fix a possible panic on multibyte unicode strings

### Version 0.1.4

- Attempt again to fix the issue some users experience where this crate would fail to compile

### Version 0.1.3

- Fix an issue some users experienced where this crate would fail to compile

### Version 0.1.2

- Rewrite the parser in nom, providing significant performance improvements across the board
- Other significant performance improvement on pathological queries

### Version 0.1.1

- Significant performance improvements

### Version 0.1.0

- Initial release
