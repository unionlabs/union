# Running analyzer unit tests

Analyzers' unit tests are ignored in bazel's build files, and therefore are not being triggered as part of the CI
pipeline. Because of this they should be invoked manually when writing a new analyzer or making changes to an existing
one. Otherwise, any issues will go unnoticed during the CI build.

The easiest way to run all unit tests for all analyzers is  `go test ./tools/analyzers/...`