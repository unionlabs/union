# Finder library for [Afero](https://github.com/spf13/afero)

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sagikazarmark/locafero/ci.yaml?style=flat-square)](https://github.com/sagikazarmark/locafero/actions/workflows/ci.yaml)
[![go.dev reference](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=flat-square)](https://pkg.go.dev/mod/github.com/sagikazarmark/locafero)
![Go Version](https://img.shields.io/badge/go%20version-%3E=1.20-61CFDD.svg?style=flat-square)
[![built with nix](https://img.shields.io/badge/builtwith-nix-7d81f7?style=flat-square)](https://builtwithnix.org)

**Finder library for [Afero](https://github.com/spf13/afero) ported from [go-finder](https://github.com/sagikazarmark/go-finder).**

> [!WARNING]
> This is an experimental library under development.
>
> **Backwards compatibility is not guaranteed, expect breaking changes.**

## Installation

```shell
go get github.com/sagikazarmark/locafero
```

## Usage

Check out the [package example](https://pkg.go.dev/github.com/sagikazarmark/locafero#example-package) on go.dev.

## Development

**For an optimal developer experience, it is recommended to install [Nix](https://nixos.org/download.html) and [direnv](https://direnv.net/docs/installation.html).**

Run the test suite:

```shell
just test
```

## License

The project is licensed under the [MIT License](LICENSE).
