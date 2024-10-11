This repository factors out an opinionated selection of internal packages and functionality from the Go standard
library. Currently this consists mostly of packages and testing code from within the Go tool implementation.

This repo is [primarily maintained](https://github.com/rogpeppe/go-internal/graphs/contributors) by long-time
[Go contributors](https://github.com/golang/go/contributors) who are also currently
[maintaining CUE](https://github.com/cue-lang/cue/graphs/contributors) (which is primarily written in Go
and which relies upon several of the packages here).

Contributions are welcome, but please open an issue for discussion first.

## Packages

Included are the following:

- goproxytest: a GOPROXY implementation designed for test use.
- gotooltest: Use the Go tool inside test scripts (see testscript below)
- imports: list of known architectures and OSs, and support for reading import statements.
- par: do work in parallel.
- testenv: information on the current testing environment.
- testscript: script-based testing based on txtar files
- txtar: simple text-based file archives for testing.

Note that most users of `txtar` should use https://pkg.go.dev/golang.org/x/tools/txtar instead.
Our package will be replaced by it once https://github.com/golang/go/issues/59264 is fixed.

### testscript

The most popular package here is the [testscript](https://pkg.go.dev/github.com/rogpeppe/go-internal/testscript) package:
 * Provides a shell-like test environment that is very nicely tuned for testing Go CLI commands.
 * Extracted from the core Go team's internal testscript package ([cmd/go/internal/script](https://github.com/golang/go/tree/master/src/cmd/go/internal/script)),
 which is [heavily used](https://github.com/golang/go/tree/master/src/cmd/go/testdata/script) to test the `go` command.
 * Supports patterns for checking stderr/stdout, command pass/fail assertions, and so on.
 * Integrates well with `go test`, including coverage support.
 * Inputs and sample output files can use the simple [txtar](https://pkg.go.dev/golang.org/x/tools/txtar)
 text archive format, also used by the Go playground.
 * Allows [automatically updating](https://pkg.go.dev/github.com/rogpeppe/go-internal/testscript#Params)
 golden files.
 * Built-in support for Go concepts like build tags.
 * Accompanied by a [testscript](https://github.com/rogpeppe/go-internal/tree/master/cmd/testscript) command
 for running standalone scripts with files embedded in txtar format.
 
 A nice introduction to using testscripts is this [blog post](https://bitfieldconsulting.com/golang/test-scripts) series.
 Both testscript and txtar were [originally created](https://github.com/golang/go/commit/5890e25b7ccb2d2249b2f8a02ef5dbc36047868b)
 by Russ Cox.
