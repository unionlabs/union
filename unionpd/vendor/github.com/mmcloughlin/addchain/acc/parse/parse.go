// Package parse implements a parser for acc programs.
package parse

import (
	"io"
	"strings"

	"github.com/mmcloughlin/addchain/acc/ast"
	"github.com/mmcloughlin/addchain/acc/parse/internal/parser"
)

//go:generate pigeon -o internal/parser/zparser.go acc.peg

// File parses filename.
func File(filename string) (*ast.Chain, error) {
	return cast(parser.ParseFile(filename))
}

// Reader parses the data from r using filename as information in
// error messages.
func Reader(filename string, r io.Reader) (*ast.Chain, error) {
	return cast(parser.ParseReader(filename, r))
}

// String parses s.
func String(s string) (*ast.Chain, error) {
	return Reader("string", strings.NewReader(s))
}

func cast(i interface{}, err error) (*ast.Chain, error) {
	if err != nil {
		return nil, err
	}
	return i.(*ast.Chain), nil
}
