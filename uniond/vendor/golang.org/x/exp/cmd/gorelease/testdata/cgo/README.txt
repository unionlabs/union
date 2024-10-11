Module example.com/cgo is used to test that packages with cgo code
can be loaded without errors when cgo is enabled.

TODO(jayconrod): test modules with cgo-only and cgo / pure Go implementations
with CGO_ENABLED=0 and 1. But first, decide how multiple platforms and
build constraints should be handled. Currently, gorelease only considers
the same configuration as 'go list'.