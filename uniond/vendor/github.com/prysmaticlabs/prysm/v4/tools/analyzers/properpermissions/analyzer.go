// Package properpermissions implements a static analyzer to ensure that Prysm does not
// use ioutil.MkdirAll or os.WriteFile as they are unsafe when it comes to guaranteeing
// file permissions and not overriding existing permissions. Instead, users are warned
// to utilize shared/file as the canonical solution.
package properpermissions

import (
	"errors"
	"fmt"
	"go/ast"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to enforce usage of Prysm's internal file-writing utils instead of os.MkdirAll or ioutil.WriteFile"

var (
	errUnsafePackage = errors.New(
		"os and ioutil dir and file writing functions are not permissions-safe, use shared/file",
	)
	disallowedFns = []string{"MkdirAll", "WriteFile"}
)

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "properpermissions",
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
		(*ast.File)(nil),
		(*ast.ImportSpec)(nil),
		(*ast.CallExpr)(nil),
	}

	aliases := make(map[string]string)

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		switch stmt := node.(type) {
		case *ast.File:
			// Reset aliases (per file).
			aliases = make(map[string]string)
		case *ast.ImportSpec:
			// Collect aliases.
			pkg := stmt.Path.Value
			if pkg == "\"os\"" {
				if stmt.Name != nil {
					aliases[stmt.Name.Name] = pkg
				} else {
					aliases["os"] = pkg
				}
			}
			if pkg == "\"io/ioutil\"" {
				if stmt.Name != nil {
					aliases[stmt.Name.Name] = pkg
				} else {
					aliases["ioutil"] = pkg
				}
			}
		case *ast.CallExpr:
			// Check if any of disallowed functions have been used.
			for alias, pkg := range aliases {
				for _, fn := range disallowedFns {
					if isPkgDot(stmt.Fun, alias, fn) {
						pass.Reportf(
							node.Pos(),
							fmt.Sprintf(
								"%v: %s.%s() (from %s)",
								errUnsafePackage,
								alias,
								fn,
								pkg,
							),
						)
					}
				}
			}
		}
	})

	return nil, nil
}

func isPkgDot(expr ast.Expr, pkg, name string) bool {
	sel, ok := expr.(*ast.SelectorExpr)
	res := ok && isIdent(sel.X, pkg) && isIdent(sel.Sel, name)
	return res
}

func isIdent(expr ast.Expr, ident string) bool {
	id, ok := expr.(*ast.Ident)
	return ok && id.Name == ident
}
