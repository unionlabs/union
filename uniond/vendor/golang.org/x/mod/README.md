# mod

[![PkgGoDev](https://pkg.go.dev/badge/golang.org/x/mod)](https://pkg.go.dev/golang.org/x/mod)

This repository holds packages for writing tools
that work directly with Go module mechanics.
That is, it is for direct manipulation of Go modules themselves.

It is NOT about supporting general development tools that
need to do things like load packages in module mode.
That use case, where modules are incidental rather than the focus,
should remain in [x/tools](https://pkg.go.dev/golang.org/x/tools),
specifically [x/tools/go/packages](https://pkg.go.dev/golang.org/x/tools/go/packages).

The specific case of loading packages should still be done by
invoking the go command, which remains the single point of
truth for package loading algorithms.
