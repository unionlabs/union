package ast

import (
	"io"
	"os"

	"github.com/mmcloughlin/addchain/internal/errutil"
	"github.com/mmcloughlin/addchain/internal/print"
)

// Print an AST node to standard out.
func Print(n interface{}) error {
	return Fprint(os.Stdout, n)
}

// Fprint writes the AST node n to w.
func Fprint(w io.Writer, n interface{}) error {
	p := newprinter(w)
	p.node(n)
	return p.Error()
}

type printer struct {
	print.Printer
}

func newprinter(w io.Writer) *printer {
	p := &printer{
		Printer: print.New(w),
	}
	p.SetIndentString(".  ")
	return p
}

func (p *printer) node(n interface{}) {
	switch n := n.(type) {
	case *Chain:
		p.enter("chain")
		for _, stmt := range n.Statements {
			p.statement(stmt)
		}
		p.leave()
	case Statement:
		p.statement(n)
	case Operand:
		p.Linef("operand(%d)", n)
	case Identifier:
		p.Linef("identifier(%q)", n)
	case Add:
		p.add(n)
	case Double:
		p.double(n)
	case Shift:
		p.shift(n)
	default:
		p.SetError(errutil.UnexpectedType(n))
	}
}

func (p *printer) statement(stmt Statement) {
	p.enter("statement")
	p.Printf("name = ")
	p.node(stmt.Name)
	p.Printf("expr = ")
	p.node(stmt.Expr)
	p.leave()
}

func (p *printer) add(a Add) {
	p.enter("add")
	p.Printf("x = ")
	p.node(a.X)
	p.Printf("y = ")
	p.node(a.Y)
	p.leave()
}

func (p *printer) double(d Double) {
	p.enter("double")
	p.Printf("x = ")
	p.node(d.X)
	p.leave()
}

func (p *printer) shift(s Shift) {
	p.enter("shift")
	p.Linef("s = %d", s.S)
	p.Printf("x = ")
	p.node(s.X)
	p.leave()
}

func (p *printer) enter(name string) {
	p.Linef("%s {", name)
	p.Indent()
}

func (p *printer) leave() {
	p.Dedent()
	p.Linef("}")
}
