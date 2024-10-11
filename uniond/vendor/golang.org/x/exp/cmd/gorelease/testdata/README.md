This directory contains most tests for gorelease. Each test runs gorelease (the
`runRelease` function) with a given set of flags in a temporary directory
populated with files specified in the test itself or files from the module test
proxy. The output is compared against a golden `want` file specified in the
test.

## Test flags

A specific test may be run with a command like:

    go test -run=TestRelease/basic/v0_patch_suggest

where `basic/v0_patch_suggest` matches the file
`testdata/basic/v0_patch_suggest.test`.

The `-u` flag adds or updates the `want` file in each test to match the output.
This is useful for fixing tests after an intended change in behavior.

    go test -run=TestRelease/basic/v0_patch_suggest -u

The `-testwork` flag instructs the test framework to leave the test's temporary
directory and module proxy in place after running the test. This is useful
for debugging.

## Test format

Tests are written in `.test` files in `testdata` subdirectories. Each `.test`
file is a valid txtar file (see `golang.org/x/tools/txtar`). The comment section
contains the test parameters, which are a series of `key=value` pairs. Blank
lines and comments starting with `#` are allowed in this section. Valid keys
are:

* `mod`: sets the module path. Must be specified together with `version`. Copies
  the content of a module out of the test proxy into a temporary directory
  where `gorelease` is run.
* `version`: specified together with `mod`, it sets the version to retrieve from
  the test proxy. See more information below.
* `base`: the value of the `-base` flag passed to `gorelease`.
* `release`: the value of the `-version` flag passed to `gorelease`.
* `dir`: the directory where `gorelease` should be invoked. Useful when the test
  describes a whole repository, and `gorelease` should be invoked in a
  subdirectory.
* `error`: true if the test expects a hard error. False by default.
* `success`: true if the test expects a report to be printed with no errors
  or diagnostics. True by default.
* `skip`: non-empty if the test should be skipped. The value is a string passed
  to `t.Skip`.
* `proxyVersions`: empty if the test should include all `mod/` entries in the
  proxy, or else a comma-separated list of the modpath@version's it should
  include.

Test archives have a file named `want`, containing the expected output of the
test. A test will fail if the actual output differs from `want`.

If the `mod` and `version` parameters are not set, other files will be extracted
to the temporary directory where `gorelease` runs.

## Populating module contents

When building a test in `testdata/`, there are two ways to populate the module
directory being tested:

### Option 1: inline

You can inline files in the `.test` folder as described in
https://pkg.go.dev/golang.org/x/tools/txtar#hdr-Txtar_format. For example,

```
-- some.file --
the contents
of the file
```

### Option 2: specify an existing file

Often, multiple tests want to share the same setup - the same files. So, users
can write these common files in `testdata/mod/`, and use one of these files as
the module directory contents.

To specify a file in `testdata/mod/` to use as the module contents.

## Module format

Tests run with `GOPROXY` set to a local URL that points to a test proxy. The
test proxy serves modules described by `.txt` files in the `testdata/mod/`
subdirectory.

Each module is a txtar archive named `$modpath_$version.txt` where `$modpath`
is the module path (with slashes replaced with underscores) and `$version` is
the version. If the archive contains a file named `.mod`, that will be used to
respond to `.mod` requests; otherwise, `go.mod` will be used (`.mod` is only
necessary for modules that lack `go.mod` files). If the archive contains a
file named `.info`, that will be used to respond to `.info` requests; otherwise,
`.info` is synthesized from the version. All other files in the archive are
packed into a `.zip` file to satisfy `.zip` requests.
