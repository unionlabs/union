# MessagePack for Golang

[![Go Reference](https://pkg.go.dev/badge/github.com/shamaton/msgpack.svg)](https://pkg.go.dev/github.com/shamaton/msgpack)
![test](https://github.com/shamaton/msgpack/workflows/test/badge.svg)
[![Go Report Card](https://goreportcard.com/badge/github.com/shamaton/msgpack)](https://goreportcard.com/report/github.com/shamaton/msgpack)
[![codecov](https://codecov.io/gh/shamaton/msgpack/branch/master/graph/badge.svg?token=9PD2JUK5V3)](https://codecov.io/gh/shamaton/msgpack)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fshamaton%2Fmsgpack.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fshamaton%2Fmsgpack?ref=badge_shield)

## 📣 Notice
If your application serializes only primitive types, array, map and struct, code generation is also recommended.
You can get the fastest performance with [msgpackgen](https://github.com/shamaton/msgpackgen).

## Features
* Supported types : primitive / array / slice / struct / map / interface{} and time.Time
* Renaming fields via `msgpack:"field_name"`
* Omitting fields via `msgpack:"-"`
* Supports extend encoder / decoder
* Can also Encoding / Decoding struct as array

## Installation

Current version is **msgpack/v2**.
```sh
go get -u github.com/shamaton/msgpack/v2
```

## Quick Start
```go
package main

import (
  "github.com/shamaton/msgpack/v2"
  "net/http"
)

type Struct struct {
	String string
}

// simple
func main() {
	v := Struct{String: "msgpack"}

	d, err := msgpack.Marshal(v)
	if err != nil {
		panic(err)
	}
	r := Struct{}
	if err =  msgpack.Unmarshal(d, &r); err != nil {
		panic(err)
	}
}

// streaming
func handle(w http.ResponseWriter, r *http.Request) {
	var body Struct
	if err := msgpack.UnmarshalRead(r, &body); err != nil {
		panic(err)
    }
	if err := msgpack.MarshalWrite(w, body); err != nil {
		panic(err)
    }
}
```

## Benchmark
This result made from [shamaton/msgpack_bench](https://github.com/shamaton/msgpack_bench)

![msgpack_bench](https://user-images.githubusercontent.com/4637556/128299009-4823e79b-d70b-4d11-8f35-10a4758dfeca.png)

## License

This library is under the MIT License.
