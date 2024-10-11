// Package maligned implements a static analyzer to ensure that Go structs take up the least possible memory.
package maligned

import (
	"errors"
	"go/ast"
	"go/types"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to detect Go structs that would take less memory if their fields were sorted."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "maligned",
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
		(*ast.StructType)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		if s, ok := node.(*ast.StructType); ok {
			if err := malign(node.Pos(), pass.TypesInfo.Types[s].Type.(*types.Struct)); err != nil {
				pass.Reportf(node.Pos(), err.Error())
			}
		}
	})

	return nil, nil
}
