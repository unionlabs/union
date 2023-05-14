// Package acc implements the "addition chain calculator" language: a
// domain-specific language (DSL) for addition chain computation.
package acc

import (
	"bytes"
	"io"
	"os"
	"strings"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/acc/parse"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/acc/printer"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// LoadFile is a convenience for loading an addition chain script from a file.
func LoadFile(filename string) (p *ir.Program, err error) {
	f, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer errutil.CheckClose(&err, f)
	return LoadReader(filename, f)
}

// LoadString is a convenience for loading and evaluating an addition chain
// script from a string.
func LoadString(src string) (*ir.Program, error) {
	return LoadReader("string", strings.NewReader(src))
}

// LoadReader is a convenience for loading and evaluating an addition chain
// script.
func LoadReader(filename string, r io.Reader) (*ir.Program, error) {
	// Parse to AST.
	s, err := parse.Reader(filename, r)
	if err != nil {
		return nil, err
	}

	// Translate to IR.
	p, err := Translate(s)
	if err != nil {
		return nil, err
	}

	// Evaluate the program.
	if err := pass.Eval(p); err != nil {
		return nil, err
	}

	return p, nil
}

// Write is a convenience for writing a program as an addition chain script.
func Write(w io.Writer, p *ir.Program) error {
	// Build AST.
	s, err := Build(p)
	if err != nil {
		return err
	}

	// Print.
	if err := printer.Fprint(w, s); err != nil {
		return err
	}

	return nil
}

// Save is a convenience for writing a program to a file.
func Save(filename string, p *ir.Program) (err error) {
	f, err := os.Create(filename)
	if err != nil {
		return err
	}
	defer errutil.CheckClose(&err, f)
	return Write(f, p)
}

// String is a convenience for obtaining a program as an addition chain script
// in string form.
func String(p *ir.Program) (string, error) {
	var buf bytes.Buffer
	if err := Write(&buf, p); err != nil {
		return "", err
	}
	return buf.String(), nil
}
