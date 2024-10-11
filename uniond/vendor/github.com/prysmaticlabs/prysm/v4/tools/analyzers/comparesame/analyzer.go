// Package comparesame implements a static analyzer to ensure that code does not contain
// comparisons of identical expressions.
package comparesame

import (
	"bytes"
	"errors"
	"go/ast"
	"go/printer"
	"go/token"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to detect comparison (==, !=, >=, <=, >, <) of identical expressions."

const messageTemplate = "Boolean expression has identical expressions on both sides. The result is always %v."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "comparesame",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspection, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.BinaryExpr)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		expr, ok := node.(*ast.BinaryExpr)
		if !ok {
			return
		}

		switch expr.Op {
		case token.EQL, token.NEQ, token.GEQ, token.LEQ, token.GTR, token.LSS:
			var xBuf, yBuf bytes.Buffer
			if err := printer.Fprint(&xBuf, pass.Fset, expr.X); err != nil {
				pass.Reportf(expr.X.Pos(), err.Error())
			}
			if err := printer.Fprint(&yBuf, pass.Fset, expr.Y); err != nil {
				pass.Reportf(expr.Y.Pos(), err.Error())
			}
			if xBuf.String() == yBuf.String() {
				switch expr.Op {
				case token.EQL, token.NEQ, token.GEQ, token.LEQ:
					pass.Reportf(expr.OpPos, messageTemplate, true)
				case token.GTR, token.LSS:
					pass.Reportf(expr.OpPos, messageTemplate, false)
				}
			}
		}
	})

	return nil, nil
}
