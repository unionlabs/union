// Package featureconfig implements a static analyzer to prevent leaking globals in tests.
package featureconfig

import (
	"errors"
	"go/ast"
	"go/token"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Enforce usage of featureconfig.InitWithReset to prevent leaking globals in tests."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "featureconfig",
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
		(*ast.CallExpr)(nil),
		(*ast.ExprStmt)(nil),
		(*ast.GoStmt)(nil),
		(*ast.DeferStmt)(nil),
		(*ast.AssignStmt)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		if ce, ok := node.(*ast.CallExpr); ok && isPkgDot(ce.Fun, "featureconfig", "Init") {
			reportForbiddenUsage(pass, ce.Pos())
			return
		}
		switch stmt := node.(type) {
		case *ast.ExprStmt:
			if call, ok := stmt.X.(*ast.CallExpr); ok && isPkgDot(call.Fun, "featureconfig", "InitWithReset") {
				reportUnhandledReset(pass, call.Lparen)
			}
		case *ast.GoStmt:
			if isPkgDot(stmt.Call, "featureconfig", "InitWithReset") {
				reportUnhandledReset(pass, stmt.Call.Lparen)
			}
		case *ast.DeferStmt:
			if isPkgDot(stmt.Call, "featureconfig", "InitWithReset") {
				reportUnhandledReset(pass, stmt.Call.Lparen)
			}
		case *ast.AssignStmt:
			if ce, ok := stmt.Rhs[0].(*ast.CallExpr); ok && isPkgDot(ce.Fun, "featureconfig", "InitWithReset") {
				for i := 0; i < len(stmt.Lhs); i++ {
					if id, ok := stmt.Lhs[i].(*ast.Ident); ok {
						if id.Name == "_" {
							reportUnhandledReset(pass, id.NamePos)
						}
					}
				}
			}
		default:
		}
	})

	return nil, nil
}

func reportForbiddenUsage(pass *analysis.Pass, pos token.Pos) {
	pass.Reportf(pos, "Use of featureconfig.Init is forbidden in test code. Please use "+
		"featureconfig.InitWithReset and call reset in the same test function.")
}

func reportUnhandledReset(pass *analysis.Pass, pos token.Pos) {
	pass.Reportf(pos, "Unhandled reset featureconfig not found in test "+
		"method. Be sure to call the returned reset function from featureconfig.InitWithReset "+
		"within this test method.")
}

func isPkgDot(expr ast.Expr, pkg, name string) bool {
	sel, ok := expr.(*ast.SelectorExpr)
	return ok && isIdent(sel.X, pkg) && isIdent(sel.Sel, name)
}

func isIdent(expr ast.Expr, ident string) bool {
	id, ok := expr.(*ast.Ident)
	return ok && id.Name == ident
}
