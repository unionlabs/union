<p align="center">
  <a href="https://sentry.io/?utm_source=github&utm_medium=logo" target="_blank">
    <picture>
      <source srcset="https://sentry-brand.storage.googleapis.com/sentry-logo-white.png" media="(prefers-color-scheme: dark)" />
      <source srcset="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" media="(prefers-color-scheme: light), (prefers-color-scheme: no-preference)" />
      <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" alt="Sentry" width="280">
    </picture>
  </a>
</p>

# Official Sentry SDK for Go

[![Build Status](https://github.com/getsentry/sentry-go/workflows/go-workflow/badge.svg)](https://github.com/getsentry/sentry-go/actions?query=workflow%3Ago-workflow)
[![Go Report Card](https://goreportcard.com/badge/github.com/getsentry/sentry-go)](https://goreportcard.com/report/github.com/getsentry/sentry-go)
[![Discord](https://img.shields.io/discord/621778831602221064)](https://discord.gg/Ww9hbqr)
[![GoDoc](https://godoc.org/github.com/getsentry/sentry-go?status.svg)](https://godoc.org/github.com/getsentry/sentry-go)
[![go.dev](https://img.shields.io/badge/go.dev-pkg-007d9c.svg?style=flat)](https://pkg.go.dev/github.com/getsentry/sentry-go)

`sentry-go` provides a Sentry client implementation for the Go programming
language. This is the next generation of the Go SDK for [Sentry](https://sentry.io/),
intended to replace the `raven-go` package.

> Looking for the old `raven-go` SDK documentation? See the Legacy client section [here](https://docs.sentry.io/clients/go/).
> If you want to start using `sentry-go` instead, check out the [migration guide](https://docs.sentry.io/platforms/go/migration/).

## Requirements

The only requirement is a Go compiler.

We verify this package against the 3 most recent releases of Go. Those are the
supported versions. The exact versions are defined in
[`GitHub workflow`](.github/workflows/test.yml).

In addition, we run tests against the current master branch of the Go toolchain,
though support for this configuration is best-effort.

## Installation

`sentry-go` can be installed like any other Go library through `go get`:

```console
$ go get github.com/getsentry/sentry-go@latest
```

Check out the [list of released versions](https://github.com/getsentry/sentry-go/releases).

## Configuration

To use `sentry-go`, you’ll need to import the `sentry-go` package and initialize
it with your DSN and other [options](https://pkg.go.dev/github.com/getsentry/sentry-go#ClientOptions).

If not specified in the SDK initialization, the
[DSN](https://docs.sentry.io/product/sentry-basics/dsn-explainer/),
[Release](https://docs.sentry.io/product/releases/) and
[Environment](https://docs.sentry.io/product/sentry-basics/environments/)
are read from the environment variables `SENTRY_DSN`, `SENTRY_RELEASE` and
`SENTRY_ENVIRONMENT`, respectively.

More on this in the [Configuration section of the official Sentry Go SDK documentation](https://docs.sentry.io/platforms/go/configuration/).

## Usage

The SDK supports reporting errors and tracking application performance.

To get started, have a look at one of our [examples](_examples/):
- [Basic error instrumentation](_examples/basic/main.go)
- [Error and tracing for HTTP servers](_examples/http/main.go)

We also provide a [complete API reference](https://pkg.go.dev/github.com/getsentry/sentry-go).

For more detailed information about how to get the most out of `sentry-go`,
checkout the official documentation:

- [Sentry Go SDK documentation](https://docs.sentry.io/platforms/go/)
- Guides:
  - [net/http](https://docs.sentry.io/platforms/go/guides/http/)
  - [echo](https://docs.sentry.io/platforms/go/guides/echo/)
  - [fasthttp](https://docs.sentry.io/platforms/go/guides/fasthttp/)
  - [gin](https://docs.sentry.io/platforms/go/guides/gin/)
  - [iris](https://docs.sentry.io/platforms/go/guides/iris/)
  - [martini](https://docs.sentry.io/platforms/go/guides/martini/)
  - [negroni](https://docs.sentry.io/platforms/go/guides/negroni/)

## Resources

- [Bug Tracker](https://github.com/getsentry/sentry-go/issues)
- [GitHub Project](https://github.com/getsentry/sentry-go)
- [![GoDoc](https://godoc.org/github.com/getsentry/sentry-go?status.svg)](https://godoc.org/github.com/getsentry/sentry-go)
- [![go.dev](https://img.shields.io/badge/go.dev-pkg-007d9c.svg?style=flat)](https://pkg.go.dev/github.com/getsentry/sentry-go)
- [![Documentation](https://img.shields.io/badge/documentation-sentry.io-green.svg)](https://docs.sentry.io/platforms/go/)
- [![Discussions](https://img.shields.io/github/discussions/getsentry/sentry-go.svg)](https://github.com/getsentry/sentry-go/discussions)
- [![Discord](https://img.shields.io/discord/621778831602221064)](https://discord.gg/Ww9hbqr)
- [![Stack Overflow](https://img.shields.io/badge/stack%20overflow-sentry-green.svg)](http://stackoverflow.com/questions/tagged/sentry)
- [![Twitter Follow](https://img.shields.io/twitter/follow/getsentry?label=getsentry&style=social)](https://twitter.com/intent/follow?screen_name=getsentry)

## License

Licensed under
[The MIT License](https://opensource.org/licenses/mit/), see
[`LICENSE`](LICENSE).

## Community

Join Sentry's [`#go` channel on Discord](https://discord.gg/Ww9hbqr) to get
involved and help us improve the SDK!
