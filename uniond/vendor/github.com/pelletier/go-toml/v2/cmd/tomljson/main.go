// Package tomljson is a program that converts TOML to JSON.
//
// # Usage
//
// Reading from stdin:
//
//	cat file.toml | tomljson > file.json
//
// Reading from a file:
//
//	tomljson file.toml > file.json
//
// # Installation
//
// Using Go:
//
//	go install github.com/pelletier/go-toml/v2/cmd/tomljson@latest
package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"

	"github.com/pelletier/go-toml/v2"
	"github.com/pelletier/go-toml/v2/internal/cli"
)

const usage = `tomljson can be used in two ways:
Reading from stdin:
  cat file.toml | tomljson > file.json

Reading from a file:
  tomljson file.toml > file.json
`

func main() {
	p := cli.Program{
		Usage: usage,
		Fn:    convert,
	}
	p.Execute()
}

func convert(r io.Reader, w io.Writer) error {
	var v interface{}

	d := toml.NewDecoder(r)
	err := d.Decode(&v)
	if err != nil {
		var derr *toml.DecodeError
		if errors.As(err, &derr) {
			row, col := derr.Position()
			return fmt.Errorf("%s\nerror occurred at row %d column %d", derr.String(), row, col)
		}
		return err
	}

	e := json.NewEncoder(w)
	e.SetIndent("", "  ")
	return e.Encode(v)
}
