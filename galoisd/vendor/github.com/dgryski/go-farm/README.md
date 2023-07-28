# go-farm

*Google's FarmHash hash functions implemented in Go*

[![Master Branch](https://img.shields.io/badge/-master:-gray.svg)](https://github.com/dgryski/go-farm/tree/master)
[![Master Build Status](https://secure.travis-ci.org/dgryski/go-farm.png?branch=master)](https://travis-ci.org/dgryski/go-farm?branch=master)
[![Master Coverage Status](https://coveralls.io/repos/dgryski/go-farm/badge.svg?branch=master&service=github)](https://coveralls.io/github/dgryski/go-farm?branch=master)
[![Go Report Card](https://goreportcard.com/badge/github.com/dgryski/go-farm)](https://goreportcard.com/report/github.com/dgryski/go-farm)
[![GoDoc](https://godoc.org/github.com/dgryski/go-farm?status.svg)](http://godoc.org/github.com/dgryski/go-farm)

## Description

FarmHash, a family of hash functions.

This is a (mechanical) translation of the non-SSE4/non-AESNI hash functions from Google's FarmHash (https://github.com/google/farmhash).


FarmHash provides hash functions for strings and other data.
The functions mix the input bits thoroughly but are not suitable for cryptography.

All members of the FarmHash family were designed with heavy reliance on previous work by Jyrki Alakuijala, Austin Appleby, Bob Jenkins, and others.

For more information please consult https://github.com/google/farmhash


## Getting started

This application is written in Go language, please refer to the guides in https://golang.org for getting started.

This project include a Makefile that allows you to test and build the project with simple commands.
To see all available options:
```bash
make help
```

## Running all tests

Before committing the code, please check if it passes all tests using
```bash
make qa
```

## License

As this is a highly derivative work, I have placed it under the same license as the original implementation.  See the
LICENSE file for details.
