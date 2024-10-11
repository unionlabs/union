// Package jsontoml is a program that converts JSON to TOML.
//
// # Usage
//
// Reading from stdin:
//
//	cat file.json | jsontoml > file.toml
//
// Reading from a file:
//
//	jsontoml file.json > file.toml
//
// # Installation
//
// Using Go:
//
//	go install github.com/pelletier/go-toml/v2/cmd/jsontoml@latest
package main

import (
	"encoding/json"
	"flag"
	"io"

	"github.com/pelletier/go-toml/v2"
	"github.com/pelletier/go-toml/v2/internal/cli"
)

const usage = `jsontoml can be used in two ways:
Reading from stdin:
  cat file.json | jsontoml > file.toml

Reading from a file:
  jsontoml file.json > file.toml
`

var useJsonNumber bool

func main() {
	flag.BoolVar(&useJsonNumber, "use-json-number", false, "unmarshal numbers into `json.Number` type instead of as `float64`")

	p := cli.Program{
		Usage: usage,
		Fn:    convert,
	}
	p.Execute()
}

func convert(r io.Reader, w io.Writer) error {
	var v interface{}

	d := json.NewDecoder(r)
	e := toml.NewEncoder(w)

	if useJsonNumber {
		d.UseNumber()
		e.SetMarshalJsonNumbers(true)
	}

	err := d.Decode(&v)
	if err != nil {
		return err
	}

	return e.Encode(v)
}
