// Package ineffassign implements a static analyzer to ensure that there are no ineffectual
// assignments in source code.
package ineffassign

import (
	"errors"
	"go/ast"
	"sort"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to make sure there are no ineffectual assignments in source code"

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "ineffassign",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

func run(pass *analysis.Pass) (interface{}, error) {
	insp, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.File)(nil),
	}
	insp.Preorder(nodeFilter, func(node ast.Node) {
		f, ok := node.(*ast.File)
		if !ok {
			return
		}
		bld := &builder{vars: map[*ast.Object]*variable{}}
		bld.walk(f)
		chk := &checker{vars: bld.vars, seen: map[*block]bool{}}
		for _, b := range bld.roots {
			chk.check(b)
		}
		sort.Sort(chk.ineff)
		// Report ineffectual assignments if any.
		for _, id := range chk.ineff {
			if id.Name != "ctx" { // We allow ineffectual assignment to ctx (to override ctx).
				pass.Reportf(id.Pos(), "ineffectual assignment to %q", id.Name)
			}
		}
	})

	return nil, nil
}
