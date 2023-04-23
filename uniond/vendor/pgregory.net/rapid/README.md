# rapid [![PkgGoDev][godev-img]][godev] [![CI][ci-img]][ci]

Rapid is a Go library for property-based testing.

Rapid checks that properties you define hold for a large number
of automatically generated test cases. If a failure is found, rapid
automatically minimizes the failing test case before presenting it.

Property-based testing emphasizes thinking about high level properties
the program should satisfy rather than coming up with a list
of individual examples of desired behavior (test cases).
This results in concise and powerful tests that are a pleasure to write.

Design and implementation of rapid are heavily inspired by
[Hypothesis](https://github.com/HypothesisWorks/hypothesis), which is itself
a descendant of [QuickCheck](https://hackage.haskell.org/package/QuickCheck).

## Features

- Idiomatic Go API
  - Type-safe data generation using generics
  - Designed to be used together with `go test` and the `testing` package
  - Works great with libraries like
    [testify/require](https://pkg.go.dev/github.com/stretchr/testify/require) and
    [testify/assert](https://pkg.go.dev/github.com/stretchr/testify/assert)
- Fully automatic minimization of failing test cases
- Persistence of minimized failing test cases
- Support for state machine ("stateful" or "model-based") testing
- No dependencies outside the Go standard library

## Examples

Here is what a trivial test using rapid looks like:

```go
package rapid_test

import (
	"net"
	"testing"

	"pgregory.net/rapid"
)

func TestParseValidIPv4(t *testing.T) {
	const ipv4re = `(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
		`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
		`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
		`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])`

	rapid.Check(t, func(t *rapid.T) {
		addr := rapid.StringMatching(ipv4re).Draw(t, "addr")
		ip := net.ParseIP(addr)
		if ip == nil || ip.String() != addr {
			t.Fatalf("parsed %q into %v", addr, ip)
		}
	})
}
```

You can [play around](https://go.dev/play/p/gtrfx-BK0t2) with the IPv4
regexp to see what happens when it is generating invalid addresses
(or try to pass the test with your own `ParseIP` implementation). More complete
function ([source code](./example_function_test.go),
[playground](https://go.dev/play/p/tZFU8zv8AUl)) and state machine
([source code](./example_statemachine_test.go),
[playground](https://go.dev/play/p/LRb_Nm1s9T5)) example tests are provided.
They both fail. Making them pass is a good way to get first real experience
of working with rapid.

## Usage

Just run `go test` as usual, it will pick up also all `rapid` tests.

There are a number of optional flags to influence rapid behavior, run
`go test -args -h` and look at the flags with the `-rapid.` prefix. You can
then pass such flags as usual. For example:

```
go test -rapid.checks=1000
```

## Comparison

Rapid aims to bring to Go the power and convenience Hypothesis brings to Python.

Compared to [gopter](https://pkg.go.dev/github.com/leanovate/gopter), rapid:

- provides type-safe data generation using generics
- has a much simpler API (queue test in [rapid](./example_statemachine_test.go) vs
  [gopter](https://github.com/leanovate/gopter/blob/master/commands/example_circularqueue_test.go))
- does not require any user code to minimize failing test cases
- persists minimized failing test cases to files for easy reproduction
- generates biased data to explore "small" values and edge cases more thoroughly (inspired by
  [SmallCheck](https://hackage.haskell.org/package/smallcheck))
- enables interactive tests by allowing data generation and test code to arbitrarily intermix

Compared to [testing/quick](https://golang.org/pkg/testing/quick/), rapid:

- provides much more control over test case generation
- supports state machine based testing
- automatically minimizes any failing test case
- has to settle for `rapid.Check` being the main exported function
  instead of much more stylish `quick.Check`
 
## Status

Rapid is preparing for stable 1.0 release. API breakage and bugs should be extremely rare.

If rapid fails to find a bug you believe it should, or the failing test case
that rapid reports does not look like a minimal one,
please [open an issue](https://github.com/flyingmutant/rapid/issues).

## License

Rapid is licensed under the [Mozilla Public License Version 2.0](./LICENSE). 

[godev-img]: https://pkg.go.dev/badge/pgregory.net/rapid
[godev]: https://pkg.go.dev/pgregory.net/rapid
[ci-img]: https://github.com/flyingmutant/rapid/workflows/CI/badge.svg
[ci]: https://github.com/flyingmutant/rapid/actions
