# cockroachdb/errors: Go errors with network portability

This library aims to be used as a drop-in replacement to
`github.com/pkg/errors` and Go's standard `errors` package.  It also
provides *network portability* of error objects, in ways suitable for
distributed systems with mixed-version software compatibility.

It also provides native and comprehensive support for [PII](https://en.wikipedia.org/wiki/Personal_data)-free details
and an opt-in [Sentry.io](https://sentry.io/) reporting mechanism that
automatically formats error details and strips them of PII.

See also [the design RFC](https://github.com/cockroachdb/cockroach/blob/master/docs/RFCS/20190318_error_handling.md).

![Build Status](https://github.com/cockroachdb/errors/actions/workflows/ci.yaml/badge.svg?branch=master)
[![Go Reference](https://pkg.go.dev/badge/github.com/cockroachdb/errors.svg)](https://pkg.go.dev/github.com/cockroachdb/errors)

Table of contents:

- [Features](#Features)
- [How to use](#How-to-use)
- [What comes out of an error?](#What-comes-out-of-an-error)
- [Available error leaves](#Available-error-leaves)
- [Available wrapper constructors](#Available-wrapper-constructors)
- [Providing PII-free details](#Providing-PII-free-details)
- [Building your own error types](#Building-your-own-error-types)
- [Error composition (summary)](#Error-composition-summary)
- [API (not constructing error objects)](#API-not-constructing-error-objects)

## Features

| Feature                                                                                               | Go's <1.13 `errors` | `github.com/pkg/errors` | Go 1.13 `errors`/`xerrors` | `cockroachdb/errors` |
|-------------------------------------------------------------------------------------------------------|---------------------|-------------------------|----------------------------|----------------------|
| error constructors (`New`, `Errorf` etc)                                                              | ✔                   | ✔                       | ✔                          | ✔                    |
| error causes (`Cause` / `Unwrap`)                                                                     |                     | ✔                       | ✔                          | ✔                    |
| cause barriers (`Opaque` / `Handled`)                                                                 |                     |                         | ✔                          | ✔                    |
| `errors.As()`, `errors.Is()`                                                                          |                     |                         | ✔                          | ✔                    |
| automatic error wrap when format ends with `: %w`                                                     |                     |                         | ✔                          | ✔                    |
| standard wrappers with efficient stack trace capture                                                  |                     | ✔                       |                            | ✔                    |
| **transparent protobuf encode/decode with forward compatibility**                                     |                     |                         |                            | ✔                    |
| **`errors.Is()` recognizes errors across the network**                                                |                     |                         |                            | ✔                    |
| **comprehensive support for PII-free reportable strings**                                             |                     |                         |                            | ✔                    |
| support for both `Cause()` and `Unwrap()` [go#31778](https://github.com/golang/go/issues/31778)       |                     |                         |                            | ✔                    |
| standard error reports to Sentry.io                                                                   |                     |                         |                            | ✔                    |
| wrappers to denote assertion failures                                                                 |                     |                         |                            | ✔                    |
| wrappers with issue tracker references                                                                |                     |                         |                            | ✔                    |
| wrappers for user-facing hints and details                                                            |                     |                         |                            | ✔                    |
| wrappers to attach secondary causes                                                                   |                     |                         |                            | ✔                    |
| wrappers to attach [`logtags`](https://github.com/cockroachdb/logtags) details from `context.Context` |                     |                         |                            | ✔                    |
| `errors.FormatError()`, `Formatter`, `Printer`                                                        |                     |                         | (under construction)       | ✔                    |
| `errors.SafeFormatError()`, `SafeFormatter`                                                           |                     |                         |                            | ✔                    |
| wrapper-aware `IsPermission()`, `IsTimeout()`, `IsExist()`, `IsNotExist()`                            |                     |                         |                            | ✔                    |

"Forward compatibility" above refers to the ability of this library to
recognize and properly handle network communication of error types it
does not know about, for example when a more recent version of a
software package sends a new error object to another system running an
older version of the package.

## How to use

- construct errors with `errors.New()`, etc as usual, but also see the other [error leaf constructors](#Available-error-leaves) below.
- wrap errors with `errors.Wrap()` as usual, but also see the [other wrappers](#Available-wrapper-constructors) below.
- test error identity with `errors.Is()` as usual.
  **Unique in this library**: this works even if the error has traversed the network!
  Also, `errors.IsAny()` to recognize two or more reference errors.
- replace uses of `os.IsPermission()`, `os.IsTimeout()`, `os.IsExist()` and `os.IsNotExist()` by their analog in sub-package `oserror` so
  that they can peek through layers of wrapping.
- access error causes with `errors.UnwrapOnce()` / `errors.UnwrapAll()` (note: `errors.Cause()` and `errors.Unwrap()` also provided for compatibility with other error packages).
- encode/decode errors to protobuf with `errors.EncodeError()` / `errors.DecodeError()`.
- extract **PII-free safe details** with `errors.GetSafeDetails()`.
- extract human-facing hints and details with `errors.GetAllHints()`/`errors.GetAllDetails()` or `errors.FlattenHints()`/`errors.FlattenDetails()`.
- produce detailed Sentry.io reports with `errors.BuildSentryReport()` / `errors.ReportError()`.
- implement your own error leaf types and wrapper types:
  - implement the `error` and `errors.Wrapper` interfaces as usual.
  - register encode/decode functions: call `errors.Register{Leaf,Wrapper}{Encoder,Decoder}()` in a `init()` function in your package.
  - implement `Format()` that redirects to `errors.FormatError()`.
  - see the section [Building your own error types](#Building-your-own-error-types) below.

## What comes out of an error?

| Error detail                                                    | `Error()` and format `%s`/`%q`/`%v` | format `%+v` | `GetSafeDetails()`            | Sentry report via `ReportError()` |
|-----------------------------------------------------------------|-------------------------------------|--------------|-------------------------------|-----------------------------------|
| main message, eg `New()`                                        | visible                             | visible      | yes (CHANGED IN v1.6)         | full (CHANGED IN v1.6)            |
| wrap prefix, eg `WithMessage()`                                 | visible (as prefix)                 | visible      | yes (CHANGED IN v1.6)         | full (CHANGED IN v1.6)            |
| stack trace, eg `WithStack()`                                   | not visible                         | simplified   | yes                           | full                              |
| hint , eg `WithHint()`                                          | not visible                         | visible      | no                            | type only                         |
| detail, eg `WithDetail()`                                       | not visible                         | visible      | no                            | type only                         |
| assertion failure annotation, eg `WithAssertionFailure()`       | not visible                         | visible      | no                            | type only                         |
| issue links, eg `WithIssueLink()`, `UnimplementedError()`       | not visible                         | visible      | yes                           | full                              |
| safe details, eg `WithSafeDetails()`                            | not visible                         | not visible  | yes                           | full                              |
| telemetry keys, eg. `WithTelemetryKey()`                        | not visible                         | visible      | yes                           | full                              |
| secondary errors, eg. `WithSecondaryError()`, `CombineErrors()` | not visible                         | visible      | redacted, recursively         | redacted, recursively             |
| barrier origins, eg. `Handled()`                                | not visible                         | visible      | redacted, recursively         | redacted, recursively             |
| error domain, eg. `WithDomain()`                                | not visible                         | visible      | yes                           | full                              |
| context tags, eg. `WithContextTags()`                           | not visible                         | visible      | keys visible, values redacted | keys visible, values redacted     |

## Available error leaves

An error *leaf* is an object that implements the `error` interface,
but does not refer to another error via a `Unwrap()` or `Cause()`
method.

- `New(string) error`, `Newf(string, ...interface{}) error`, `Errorf(string, ...interface{}) error`: leaf errors with message
  - **when to use: common error cases.**
  - what it does: also captures the stack trace at point of call and redacts the provided message for safe reporting.
  - how to access the detail: `Error()`, regular Go formatting. **Details in Sentry report.**
  - see also: Section [Error composition](#Error-composition-summary) below. `errors.NewWithDepth()` variants to customize at which call depth the stack trace is captured.

- `AssertionFailedf(string, ...interface{}) error`, `NewAssertionFailureWithWrappedErrf(error, string, ...interface{}) error`: signals an assertion failure / programming error.
  - **when to use: when an invariant is violated; when an unreachable code path is reached.**
  - what it does: also captures the stack trace at point of call, redacts the provided strings for safe reporting, prepares a hint to inform a human user.
  - how to access the detail: `IsAssertionFailure()`/`HasAssertionFailure()`, format with `%+v`, Safe details included in Sentry reports.
  - see also: Section [Error composition](#Error-composition-summary) below. `errors.AssertionFailedWithDepthf()` variant to customize at which call depth the stack trace is captured.

- `Handled(error) error`, `Opaque(error) error`, `HandledWithMessage(error, string) error`: captures an error cause but make it invisible to `Unwrap()` / `Is()`.
  - **when to use: when a new error occurs while handling an error, and the original error must be "hidden".**
  - what it does: captures the cause in a hidden field. The error message is preserved unless the `...WithMessage()` variant is used.
  - how to access the detail: format with `%+v`, redacted details reported in Sentry reports.

- `UnimplementedError(IssueLink, string) error`: captures a message string and a URL reference to an external resource to denote a feature that was not yet implemented.
  - **when to use: to inform (human) users that some feature is not implemented yet and refer them to some external resource.**
  - what it does: captures the message, URL and detail in a wrapper. The URL and detail are considered safe for reporting.
  - how to access the detail: `errors.GetAllHints()`, `errors.FlattenHints()`, format with `%+v`, URL and detail included in Sentry report (not the message).
  - see also: `errors.WithIssueLink()` below for errors that are not specifically about unimplemented features.

## Available wrapper constructors

An error *wrapper* is an object that implements the `error` interface,
and also refers to another error via an `Unwrap()` (preferred) and/or
`Cause()` method.

All wrapper constructors can be applied safely to a `nil` `error`:
they behave as no-ops in this case:

```go
// The following:
// if err := foo(); err != nil {
//    return errors.Wrap(err, "foo")
// }
// return nil
//
// is not needed. Instead, you can use this:
return errors.Wrap(foo(), "foo")
```

- `Wrap(error, string) error`, `Wrapf(error, string, ...interface{}) error`:
  - **when to use: on error return paths.**
  - what it does: combines `WithMessage()`, `WithStack()`, `WithSafeDetails()`.
  - how to access the details: `Error()`, regular Go formatting. **Details in Sentry report.**
  - see also: Section [Error composition](#Error-composition-summary) below. `WrapWithDepth()` variants to customize at which depth the stack trace is captured.

- `WithSecondaryError(error, error) error`: annotate an error with a secondary error.
  - **when to use: when an additional error occurs in the code that is handling a primary error.** Consider using `errors.CombineErrors()` instead (see below).
  - what it does: it captures the secondary error but hides it from `errors.Is()`.
  - how to access the detail: format with `%+v`, redacted recursively in Sentry reports.
  - see also: `errors.CombineErrors()`

- `CombineErrors(error, error) error`: combines two errors into one.
  - **when to use: when two operations occur concurrently and either can return an error, and only one final error must be returned.**
  - what it does: returns either of its arguments if the other is `nil`, otherwise calls `WithSecondaryError()`.
  - how to access the detail: see `WithSecondaryError()` above.

- `Mark(error, error) error`: gives the identity of one error to another error.
  - **when to use: when a caller expects to recognize a sentinel error with `errors.Is()` but the callee provides a diversity of error messages.**
  - what it does: it overrides the "error mark" used internally by `errors.Is()`.
  - how to access the detail: format with `%+v`, Sentry reports.

- `WithStack(error) error`: annotate with stack trace
  - **when to use:** usually not needed, use `errors.Wrap()`/`errors.Wrapf()` instead.

    **Special cases:**

    - when returning a sentinel, for example:

      ```go
      var myErr = errors.New("foo")

      func myFunc() error {
        if ... {
           return errors.WithStack(myErr)
        }
      }
      ```

    - on error return paths, when not trivial but also not warranting a wrap. For example:

      ```go
      err := foo()
      if err != nil {
        doSomething()
        if !somecond {
           return errors.WithStack(err)
        }
      }
        ```

  - what it does: captures (efficiently) a stack trace.
  - how to access the details: format with `%+v`, `errors.GetSafeDetails()`, Sentry reports. The stack trace is considered safe for reporting.
  - see also: `WithStackDepth()` to customize the call depth at which the stack trace is captured.

- `WithSafeDetails(error, string, ...interface{}) error`: safe details for reporting.
  - when to use: probably never. Use `errors.Wrap()`/`errors.Wrapf()` instead.
  - what it does: saves some strings for safe reporting.
  - how to access the detail: format with `%+v`, `errors.GetSafeDetails()`, Sentry report.

- `WithMessage(error, string) error`, `WithMessagef(error, string, ...interface{}) error`: message prefix.
  - when to use: probably never. Use `errors.Wrap()`/`errors.Wrapf()` instead.
  - what it does: adds a message prefix.
  - how to access the detail: `Error()`, regular Go formatting, Sentry Report.

- `WithDetail(error, string) error`, `WithDetailf(error, string, ...interface{}) error`, user-facing detail with contextual information.
  - **when to use: need to embark a message string to output when the error is presented to a human.**
  - what it does: captures detail strings.
  - how to access the detail: `errors.GetAllDetails()`, `errors.FlattenDetails()` (all details are preserved), format with `%+v`. Not included in Sentry reports.

- `WithHint(error, string) error`, `WithHintf(error, string, ...interface{}) error`: user-facing detail with suggestion for action to take.
  - **when to use: need to embark a message string to output when the error is presented to a human.**
  - what it does: captures hint strings.
  - how to access the detail: `errors.GetAllHints()`, `errors.FlattenHints()` (hints are de-duplicated), format with `%+v`. Not included in Sentry reports.

- `WithIssueLink(error, IssueLink) error`: annotate an error with an URL and arbitrary string.
  - **when to use: to refer (human) users to some external resources.**
  - what it does: captures the URL and detail in a wrapper. Both are considered safe for reporting.
  - how to access the detail: `errors.GetAllHints()`, `errors.FlattenHints()`,  `errors.GetSafeDetails()`, format with `%+v`, Sentry report.
  - see also: `errors.UnimplementedError()` to construct leaves (see previous section).

- `WithTelemetry(error, string) error`: annotate an error with a key suitable for telemetry.
  - **when to use: to gather strings during error handling, for capture in the telemetry sub-system of a server package.**
  - what it does: captures the string. The telemetry key is considered safe for reporting.
  - how to access the detail: `errors.GetTelemetryKeys()`,  `errors.GetSafeDetails()`, format with `%+v`, Sentry report.

- `WithDomain(error, Domain) error`, `HandledInDomain(error, Domain) error`, `HandledInDomainWithMessage(error, Domain, string) error` **(experimental)**: annotate an error with an origin package.
  - **when to use: at package boundaries.**
  - what it does: captures the identity of the error domain. Can be asserted with `errors.EnsureNotInDomain()`, `errors.NotInDomain()`.
  - how to access the detail: format with `%+v`, Sentry report.

- `WithAssertionFailure(error) error`: annotate an error as being an assertion failure.
  - when to use: probably never. Use `errors.AssertionFailedf()` and variants.
  - what it does: wraps the error with a special type. Triggers an auto-generated hint.
  - how to access the detail: `IsAssertionFailure()`/`HasAssertionFailure()`, `errors.GetAllHints()`, `errors.FlattenHints()`, format with `%+v`, Sentry report.

- `WithContextTags(error, context.Context) error`: annotate an error with the k/v pairs attached to a `context.Context` instance with the [`logtags`](https://github.com/cockroachdb/logtags) package.
  - **when to use: when capturing/producing an error and a `context.Context` is available.**
  - what it does: it captures the `logtags.Buffer` object in the wrapper.
  - how to access the detail: `errors.GetContextTags()`, format with `%+v`, Sentry reports.

## Providing PII-free details

The library support PII-free strings essentially as follows:

- by default, many strings included in an error object are considered
  to be PII-unsafe, and are stripped out when building a Sentry
  report.
- some fields in the library are assumed to be PII-safe by default.
- you can opt additional strings in to Sentry reports.

The following strings from this library are considered to be PII-free,
and thus included in Sentry reports automatically:

- the *type* of error objects,
- stack traces (containing only file paths, line numbers, function names - arguments are not included),
- issue tracker links (including URL and detail field),
- telemetry keys,
- error domains,
- context tag keys,
- the `format string` argument of `Newf`, `AssertionFailedf`, etc (the constructors ending with `...f()`),
- the *type* of the additional arguments passed to the `...f()` constructors,
- the *value of specific argument types* passed to the `...f()` constructors, when known to be PII-safe.
  For details of which arguments are considered PII-free, see the [`redact` package](https://github.com/cockroachdb/redact).

It is possible to opt additional in to Sentry reporting, using either of the following methods:

- implement the `errors.SafeDetailer` interface, providing the
  `SafeDetails() []string` method on your error type.

- enclose additional arguments passed to the `...f()` constructors with `errors.Safe()`. For example:
  `err := errors.Newf("my code: %d", errors.Safe(123))`
  — in this example, the value 123 will be included when a Sentry report is constructed.
  - it also makes it available via `errors.GetSafeDetails()`/`GetAllSafeDetails()`.
  - the value 123 is also part of the main error message returned by `Error()`.

- attach additional arbitrary strings with `errors.WithSafeDetails(error, string, ...interface{}) error` and
  also use `errors.Safe()`.
  For example: `err = errors.WithSafeDetails(err, "additional data: %s", errors.Safe("hello"))`.
  - in this example, the string "hello" will be included in Sentry reports.
  - however, it is not part of the main error message returned by `Error()`.

For more details on how Sentry reports are built, see the [`report`](report) sub-package.

## Building your own error types

You can create an error type as usual in Go: implement the `error`
interface, and, if your type is also a wrapper, the `errors.Wrapper`
interface (an `Unwrap()` method). You may also want to implement the
`Cause()` method for backward compatibility with
`github.com/pkg/errors`, if your project also uses that.

If your error type is a wrapper, you should implement a `Format()`
method that redirects to `errors.FormatError()`, otherwise `%+v` will
not work. Additionally, if your type has a payload not otherwise
visible via `Error()`, you may want to implement
`errors.SafeFormatter`. See [making `%+v` work with your
type](#Making-v-work-with-your-type) below for details.

Finally, you may want your new error type to be portable across
the network.

If your error type is a leaf, and already implements `proto.Message`
(from [gogoproto](https://github.com/gogo/protobuf)), you are all set
and the errors library will use that automatically. If you do not or
cannot implement `proto.Message`, or your error type is a wrapper,
read on.

At a minimum, you will need a *decoder function*: while
`cockroachdb/errors` already does a bunch of encoding/decoding work on
new types automatically, the one thing it really cannot do on its own
is instantiate a Go object using your new type.

Here is the simplest decode function for a new leaf error type and a
new wrapper type:

```go
// note: we use the gogoproto `proto` sub-package.
func yourDecode(_ string, _ []string, _ proto.Message) error {
   return &yourType{}
}

func init() {
   errors.RegisterLeafEncoder((*yourType)(nil), yourDecodeFunc)
}

func yourDecodeWrapper(cause error, _ string, _ []string, _ proto.Message) error {
   // Note: the library already takes care of encoding/decoding the cause.
   return &yourWrapperType{cause: cause}
}

func init() {
   errors.RegisterWrapperDecoder((*yourWrapperType)(nil), yourDecodeWrapper)
}
```

In the case where your type does not have any other field (empty
struct for leafs, just a cause for wrappers), this is all you have to
do.

(See the type `withAssertionFailure` in
[`assert/assert.go`](assert/assert.go) for an example of this simple
case.)

If your type does have additional fields, you *may* still not need a
custom encoder.  This is because the library automatically
encodes/decodes the main error message and any safe strings that your
error types makes available via the `errors.SafeDetailer` interface
(the `SafeDetails()` method).

Say, for example, you have the following leaf type:

```go
type myLeaf struct {
   code int
}

func (m *myLeaf) Error() string { return fmt.Sprintf("my error: %d" + m.code }
```

In that case, the library will automatically encode the result of
calling `Error()`. This string will then be passed back to your
decoder function as the first argument. This makes it possible
to decode the `code` field exactly:

```go
func myLeafDecoder(msg string, _ []string, _ proto.Message) error {
	codeS := strings.TrimPrefix(msg, "my error: ")
	code, _ := strconv.Atoi(codeS)
	// Note: error handling for strconv is omitted here to simplify
	// the explanation. If your decoder function should fail, simply
	// return a `nil` error object (not another unrelated error!).
	return &myLeaf{code: code}
}
```

Likewise, if your fields are PII-free, they are safe to expose via the
`errors.SafeDetailer` interface. Those strings also get encoded
automatically, and get passed to the decoder function as the second
argument.

For example, say you have the following leaf type:

```go
type myLeaf struct {
   // both fields are PII-free.
   code int
   tag string
}

func (m *myLeaf) Error() string { ... }
```

Then you can expose the fields as safe details as follows:

```go
func (m *myLeaf) SafeDetails() []string {
  return []string{fmt.Sprintf("%d", m.code), m.tag}
}
```

(If the data is PII-free, then it is good to do this in any case: it
enables any network system that receives an error of your type, but
does not know about it, to still produce useful Sentry reports.)

Once you have this, the decode function receives the strings and you
can use them to re-construct the error:

```go
func myLeafDecoder(_ string, details []string, _ proto.Message) error {
    // Note: you may want to test the length of the details slice
	// is correct.
    code, _ := strconv.Atoi(details[0])
    tag := details[1]
	return &myLeaf{code: code, tag: tag}
}
```

(For an example, see the `withTelemetry` type in [`telemetry/with_telemetry.go`](telemetry/with_telemetry.go).)

__The only case where you need a custom encoder is when your error
type contains some fields that are not reflected in the error message
(so you can't extract them back from there), and are not PII-free and
thus cannot be reported as "safe details".__

To take inspiration from examples, see the following types in the
library that need a custom encoder:

- Hints/details in [`hintdetail/with_hint.go`](hintdetail/with_hint.go) and [`hintdetail/with_detail.go`](hintdetail/with_detail.go).
- Secondary error wrappers in [`secondary/with_secondary.go`](secondary/with_secondary.go).
- Marker error wrappers at the end of [`markers/markers.go`](markers/markers.go).

### Making `%+v` work with your type

In short:

- When in doubt, you should always implement the `fmt.Formatter`
  interface (`Format(fmt.State, rune)`) on your custom error types,
  exactly as follows:

  ```go
  func (e *yourType) Format(s *fmt.State, verb rune) { errors.FormatError(e, s, verb) }
  ```

  (If you do not provide this redirection for your own custom wrapper
  type, this will disable the recursive application of the `%+v` flag
  to the causes chained from your wrapper.)

- You may optionally implement the `errors.SafeFormatter` interface:
  `SafeFormatError(p errors.Printer) (next error)`.  This is optional, but
  should be done when some details are not included by `Error()` and
  should be emitted upon `%+v`.

The example `withHTTPCode` wrapper [included in the source tree](exthttp/ext_http.go)
achieves this as follows:

```go
// Format() implements fmt.Formatter, is required until Go knows about FormatError.
func (w *withHTTPCode) Format(s fmt.State, verb rune) { errors.FormatError(w, s, verb) }

// FormatError() formats the error.
func (w *withHTTPCode) SafeFormatError(p errors.Printer) (next error) {
	// Note: no need to print out the cause here!
	// FormatError() knows how to do this automatically.
	if p.Detail() {
		p.Printf("http code: %d", errors.Safe(w.code))
	}
	return w.cause
}

```

Technical details follow:

- The errors library follows [the Go 2
proposal](https://go.googlesource.com/proposal/+/master/design/29934-error-values.md).

- At some point in the future, Go's standard `fmt` library will learn
  [how to recognize error wrappers, and how to use the `errors.Formatter`
  interface automatically](https://github.com/golang/go/issues/29934).  Until
  then, you must ensure that you also implement a `Format()` method
  (from `fmt.Formatter`) that redirects to `errors.FormatError`.

  Note: you may implement `fmt.Formatter` (`Format()` method) in this
  way without implementing `errors.Formatter` (a `FormatError()`
  method). In that case, `errors.FormatError` will use a separate code
  path that does "the right thing", even for wrappers.

- The library provides an implementation of `errors.FormatError()`,
  modeled after the same function in Go 2. This is responsible for
  printing out error details, and knows how to present a chain of
  causes in a semi-structured format upon formatting with `%+v`.

### Ensuring `errors.Is` works when errors/packages are renamed

If a Go package containing a custom error type is renamed, or the
error type itself is renamed, and errors of this type are transported
over the network, then another system with a different code layout
(e.g. running a different version of the software) may not be able to
recognize the error any more via `errors.Is`.

To ensure that network portability continues to work across multiple
software versions, in the case error types get renamed or Go packages
get moved / renamed / etc, the server code must call
`errors.RegisterTypeMigration()` from e.g. an `init()` function.

Example use:

```go
 previousPath := "github.com/old/path/to/error/package"
 previousTypeName := "oldpackage.oldErrorName"
 newErrorInstance := &newTypeName{...}
 errors.RegisterTypeMigration(previousPath, previousTypeName, newErrorInstance)
```

## Error composition (summary)

| Constructor                        | Composes                                                                          |
|------------------------------------|-----------------------------------------------------------------------------------|
| `New`                              | `NewWithDepth` (see below)                                                        |
| `Errorf`                           | = `Newf`                                                                          |
| `Newf`                             | `NewWithDepthf` (see below)                                                       |
| `WithMessage`                      | custom wrapper with message prefix and knowledge of safe strings                  |
| `Wrap`                             | `WrapWithDepth` (see below)                                                       |
| `Wrapf`                            | `WrapWithDepthf` (see below)                                                      |
| `AssertionFailed`                  | `AssertionFailedWithDepthf` (see below)                                           |
| `NewWithDepth`                     | custom leaf with knowledge of safe strings + `WithStackDepth` (see below)         |
| `NewWithDepthf`                    | custom leaf with knowledge of safe strings + `WithSafeDetails` + `WithStackDepth` |
| `WithMessagef`                     | custom wrapper with message prefix and knowledge of safe strings                  |
| `WrapWithDepth`                    | `WithMessage` + `WithStackDepth`                                                  |
| `WrapWithDepthf`                   | `WithMessagef` + `WithStackDepth`                                                 |
| `AssertionFailedWithDepthf`        | `NewWithDepthf` + `WithAssertionFailure`                                          |
| `NewAssertionErrorWithWrappedErrf` | `HandledWithMessagef` (barrier) + `WrapWithDepthf` +  `WithAssertionFailure`      |

## API (not constructing error objects)

The following is a summary of the non-constructor API functions, grouped by category.
Detailed documentation can be found at: https://pkg.go.dev/github.com/cockroachdb/errors

```go
// Access causes.
func UnwrapAll(err error) error
func UnwrapOnce(err error) error
func Cause(err error) error // compatibility
func Unwrap(err error) error // compatibility
type Wrapper interface { ... } // compatibility

// Error formatting.
type Formatter interface { ... } // compatibility, not recommended
type SafeFormatter interface { ... }
type Printer interface { ... }
func FormatError(err error, s fmt.State, verb rune)
func Formattable(err error) fmt.Formatter

// Identify errors.
func Is(err, reference error) bool
func IsAny(err error, references ...error) bool
func If(err error, pred func(err error) (interface{}, bool)) (interface{}, bool)
func As(err error, target interface{}) bool

// Encode/decode errors.
type EncodedError // this is protobuf-encodable
func EncodeError(ctx context.Context, err error) EncodedError
func DecodeError(ctx context.Context, enc EncodedError) error

// Register encode/decode functions for custom/new error types.
func RegisterLeafDecoder(typeName TypeKey, decoder LeafDecoder)
func RegisterLeafEncoder(typeName TypeKey, encoder LeafEncoder)
func RegisterWrapperDecoder(typeName TypeKey, decoder WrapperDecoder)
func RegisterWrapperEncoder(typeName TypeKey, encoder WrapperEncoder)
type LeafEncoder = func(ctx context.Context, err error) (msg string, safeDetails []string, payload proto.Message)
type LeafDecoder = func(ctx context.Context, msg string, safeDetails []string, payload proto.Message) error
type WrapperEncoder = func(ctx context.Context, err error) (msgPrefix string, safeDetails []string, payload proto.Message)
type WrapperDecoder = func(ctx context.Context, cause error, msgPrefix string, safeDetails []string, payload proto.Message) error

// Registering package renames for custom error types.
func RegisterTypeMigration(previousPkgPath, previousTypeName string, newType error)

// Sentry reports.
func BuildSentryReport(err error) (*sentry.Event, map[string]interface{})
func ReportError(err error) (string)

// Stack trace captures.
func GetOneLineSource(err error) (file string, line int, fn string, ok bool)
type ReportableStackTrace = sentry.StackTrace
func GetReportableStackTrace(err error) *ReportableStackTrace

// Safe (PII-free) details.
type SafeDetailPayload struct { ... }
func GetAllSafeDetails(err error) []SafeDetailPayload
func GetSafeDetails(err error) (payload SafeDetailPayload)

// Obsolete APIs.
type SafeMessager interface { ... }
func Redact(r interface{}) string

// Aliases redact.Safe.
func Safe(v interface{}) SafeMessager

// Assertion failures.
func HasAssertionFailure(err error) bool
func IsAssertionFailure(err error) bool

// User-facing details and hints.
func GetAllDetails(err error) []string
func FlattenDetails(err error) string
func GetAllHints(err error) []string
func FlattenHints(err error) string

// Issue links / URL wrappers.
func HasIssueLink(err error) bool
func IsIssueLink(err error) bool
func GetAllIssueLinks(err error) (issues []IssueLink)

// Unimplemented errors.
func HasUnimplementedError(err error) bool
func IsUnimplementedError(err error) bool

// Telemetry keys.
func GetTelemetryKeys(err error) []string

// Domain errors.
type Domain
const NoDomain Domain
func GetDomain(err error) Domain
func NamedDomain(domainName string) Domain
func PackageDomain() Domain
func PackageDomainAtDepth(depth int) Domain
func EnsureNotInDomain(err error, constructor DomainOverrideFn, forbiddenDomains ...Domain) error
func NotInDomain(err error, doms ...Domain) bool

// Context tags.
func GetContextTags(err error) []*logtags.Buffer
```
