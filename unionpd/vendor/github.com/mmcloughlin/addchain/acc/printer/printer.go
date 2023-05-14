// Package printer implements printing of acc AST nodes.
package printer

import (
	"bytes"
	"io"
	"os"

	"github.com/mmcloughlin/addchain/acc/ast"
	"github.com/mmcloughlin/addchain/internal/errutil"
	"github.com/mmcloughlin/addchain/internal/print"
)

// String prints the AST and returns resulting string.
func String(n interface{}) (string, error) {
	b, err := Bytes(n)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// Bytes prints the AST and returns resulting bytes.
func Bytes(n interface{}) ([]byte, error) {
	var buf bytes.Buffer
	if err := Fprint(&buf, n); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// Print an AST node to standard out.
func Print(n interface{}) error {
	return Fprint(os.Stdout, n)
}

// Fprint writes the AST node n to w.
func Fprint(w io.Writer, n interface{}) error {
	p := newprinter(w)
	p.node(n)
	p.Flush()
	return p.Error()
}

type printer struct {
	*print.TabWriter
}

func newprinter(w io.Writer) *printer {
	return &printer{
		TabWriter: print.NewTabWriter(w, 1, 4, 1, ' ', 0),
	}
}

func (p *printer) node(n interface{}) {
	switch n := n.(type) {
	case *ast.Chain:
		for _, stmt := range n.Statements {
			p.statement(stmt)
		}
	case ast.Statement:
		p.statement(n)
	case ast.Expr:
		p.expr(n, nil)
	default:
		p.SetError(errutil.UnexpectedType(n))
	}
}

func (p *printer) statement(stmt ast.Statement) {
	if len(stmt.Name) > 0 {
		p.Printf("%s\t=\t", stmt.Name)
	} else {
		p.Printf("return\t\t")
	}
	p.expr(stmt.Expr, nil)
	p.NL()
}

func (p *printer) expr(e, parent ast.Expr) {
	// Parens required if the precence of this operator is less than its parent.
	if parent != nil && e.Precedence() < parent.Precedence() {
		p.Printf("(")
		p.expr(e, nil)
		p.Printf(")")
		return
	}

	switch e := e.(type) {
	case ast.Operand:
		p.operand(e)
	case ast.Identifier:
		p.identifier(e)
	case ast.Add:
		p.add(e)
	case ast.Double:
		p.double(e)
	case ast.Shift:
		p.shift(e)
	default:
		p.SetError(errutil.UnexpectedType(e))
	}
}

func (p *printer) add(a ast.Add) {
	p.expr(a.X, a)
	p.Printf(" + ")
	p.expr(a.Y, a)
}

func (p *printer) double(d ast.Double) {
	p.Printf("2*")
	p.expr(d.X, d)
}

func (p *printer) shift(s ast.Shift) {
	p.expr(s.X, s)
	p.Printf(" << %d", s.S)
}

func (p *printer) identifier(name ast.Identifier) {
	p.Printf("%s", name)
}

func (p *printer) operand(op ast.Operand) {
	if op == 0 {
		p.Printf("1")
	} else {
		p.Printf("[%d]", op)
	}
}
