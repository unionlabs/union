// Package nop implements a static analyzer to ensure that code does not contain no-op instructions.
package nop

import (
	"errors"
	"go/ast"
	"go/token"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to detect no-op instructions."

const message = "Found a no-op instruction that can be safely removed. " +
	"It might be a result of writing code that does not do what was intended."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "nop",
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
		(*ast.StarExpr)(nil),
		(*ast.UnaryExpr)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		switch expr := node.(type) {
		case *ast.StarExpr:
			unaryExpr, ok := expr.X.(*ast.UnaryExpr)
			if !ok {
				return
			}

			if unaryExpr.Op == token.AND {
				pass.Reportf(expr.Star, message)
			}
		case *ast.UnaryExpr:
			if expr.Op == token.AND {
				if _, ok := expr.X.(*ast.StarExpr); ok {
					pass.Reportf(expr.OpPos, message)
				}
			}
		}
	})

	return nil, nil
}
