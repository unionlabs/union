// Package logruswitherror implements a static analyzer to ensure that log statements do not use
// errors in templated log statements. Authors should use logrus.WithError().
package logruswitherror

import (
	"errors"
	"go/ast"
	"go/types"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "This analyzer requires that log statements do not use errors in templated log statements."

const errImproperUsage = "use log.WithError rather than templated log statements with errors"

// Map of logrus templated log functions.
var logFns = map[string]interface{}{
	"Debugf":   nil,
	"Infof":    nil,
	"Printf":   nil,
	"Warnf":    nil,
	"Warningf": nil,
	"Errorf":   nil,
	"Fatalf":   nil,
	"Panicf":   nil,
}

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "logruswitherror",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspect, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.CallExpr)(nil),
	}

	inspect.Preorder(nodeFilter, func(node ast.Node) {
		switch stmt := node.(type) {
		case *ast.CallExpr:
			fse, ok := stmt.Fun.(*ast.SelectorExpr)
			if !ok {
				return
			}

			// Only complain on log functions.
			if x, ok := fse.X.(*ast.Ident); !ok || x.Name != "log" {
				return
			}

			// Lookup function name
			fnName := fse.Sel.Name

			// If function matches any of the logrus functions, check if it uses errors.
			if _, ok := logFns[fnName]; !ok {
				return
			}

			for _, arg := range stmt.Args {
				switch a := arg.(type) {
				case *ast.Ident:
					// Check if the error is a variable.
					if a.Obj == nil {
						return
					}

					var typ types.Type

					switch f := a.Obj.Decl.(type) {
					case *ast.AssignStmt:
						name := a.Name
						for _, lhs := range f.Lhs {
							if l, ok := lhs.(*ast.Ident); ok && l.Name == name {
								typ = pass.TypesInfo.TypeOf(l)
								break
							}
						}
					case *ast.Field:
						typ = pass.TypesInfo.TypeOf(f.Type)
					}

					if typ != nil && typ.String() == "error" {
						pass.Reportf(a.Pos(), errImproperUsage)
					}
				}
			}
		}
	})

	return nil, nil
}
