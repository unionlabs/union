# [slog](https://pkg.go.dev/log/slog) shim

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sagikazarmark/slog-shim/ci.yaml?style=flat-square)](https://github.com/sagikazarmark/slog-shim/actions/workflows/ci.yaml)
[![go.dev reference](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=flat-square)](https://pkg.go.dev/mod/github.com/sagikazarmark/slog-shim)
![Go Version](https://img.shields.io/badge/go%20version-%3E=1.20-61CFDD.svg?style=flat-square)
[![built with nix](https://img.shields.io/badge/builtwith-nix-7d81f7?style=flat-square)](https://builtwithnix.org)

Go 1.21 introduced a [new structured logging package](https://golang.org/doc/go1.21#slog), `log/slog`, to the standard library.
Although it's been eagerly anticipated by many, widespread adoption isn't expected to occur immediately,
especially since updating to Go 1.21 is a decision that most libraries won't make overnight.

Before this package was added to the standard library, there was an _experimental_ version available at [golang.org/x/exp/slog](https://pkg.go.dev/golang.org/x/exp/slog).
While it's generally advised against using experimental packages in production,
this one served as a sort of backport package for the last few years,
incorporating new features before they were added to the standard library (like `slices`, `maps` or `errors`).

This package serves as a bridge, helping libraries integrate slog in a backward-compatible way without having to immediately update their Go version requirement to 1.21. On Go 1.21 (and above), it acts as a drop-in replacement for `log/slog`, while below 1.21 it falls back to `golang.org/x/exp/slog`.

**How does it achieve backwards compatibility?**

Although there's no consensus on whether dropping support for older Go versions is considered backward compatible, a majority seems to believe it is.
(I don't have scientific proof for this, but it's based on conversations with various individuals across different channels.)

This package adheres to that interpretation of backward compatibility. On Go 1.21, the shim uses type aliases to offer the same API as `slog/log`.
Once a library upgrades its version requirement to Go 1.21, it should be able to discard this shim and use `log/slog` directly.

For older Go versions, the library might become unstable after removing the shim.
However, since those older versions are no longer supported, the promise of backward compatibility remains intact.

## Installation

```shell
go get github.com/sagikazarmark/slog-shim
```

## Usage

Import this package into your library and use it in your public API:

```go
package mylib

import slog "github.com/sagikazarmark/slog-shim"

func New(logger *slog.Logger) MyLib {
    // ...
}
```

When using the library, clients can either use `log/slog` (when on Go 1.21) or `golang.org/x/exp/slog` (below Go 1.21):

```go
package main

import "log/slog"

// OR

import "golang.org/x/exp/slog"

mylib.New(slog.Default())
```

**Make sure consumers are aware that your API behaves differently on different Go versions.**

Once you bump your Go version requirement to Go 1.21, you can drop the shim entirely from your code:

```diff
package mylib

- import slog "github.com/sagikazarmark/slog-shim"
+ import "log/slog"

func New(logger *slog.Logger) MyLib {
    // ...
}
```

## License

The project is licensed under a [BSD-style license](LICENSE).
