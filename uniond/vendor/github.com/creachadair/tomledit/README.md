# tomledit

[![GoDoc](https://img.shields.io/static/v1?label=godoc&message=reference&color=yellowgreen)](https://pkg.go.dev/github.com/creachadair/tomledit)

This repository defines a Go library to parse and manipulate the syntactic
structure of TOML documents.  Unlike other TOML libraries, this one does not
convert values into Go data structures (for that I recommend
[github.com/BurntSushi/toml][toml]).  However, it does preserve the complete
structure of its input, including declaration order and comments, allowing a
TOML document to be read, manipulated, and written back out without loss.

This library is intended to implement the [TOML v1.0.0][spec] specification.

**Handle with care:** This code is a work-in-progress and should not be
considered ready for production use. The package API may still change, and more
test coverage is needed.

[toml]: https://pkg.go.dev/github.com/BurntSushi/toml
[spec]: https://toml.io/en/v1.0.0
