// Package interfacechecker implements a static analyzer to prevent incorrect conditional checks on select interfaces.
package interfacechecker

import (
	"errors"
	"go/ast"
	"go/token"
	"go/types"
	"strings"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Enforce usage of proper conditional check for interfaces"

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "interfacechecker",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

// These are the selected interfaces that we want to parse through and check nilness for.
var selectedInterfaces = []string{
	"interfaces.SignedBeaconBlock",
	"interfaces.MetadataV0",
	"interface.BeaconState",
	"interface.ReadOnlyBeaconState",
	"interface.WriteOnlyBeaconState",
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspection, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.IfStmt)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		stmt, ok := node.(*ast.IfStmt)
		if !ok {
			return
		}
		exp, ok := stmt.Cond.(*ast.BinaryExpr)
		if !ok {
			return
		}
		handleConditionalExpression(exp, pass)
	})

	return nil, nil
}

func handleConditionalExpression(exp *ast.BinaryExpr, pass *analysis.Pass) {
	identX, ok := exp.X.(*ast.Ident)
	if !ok {
		return
	}
	identY, ok := exp.Y.(*ast.Ident)
	if !ok {
		return
	}
	typeMap := pass.TypesInfo.Types
	if _, ok := typeMap[identX].Type.(*types.Slice); ok {
		return
	}
	if _, ok := typeMap[identY].Type.(*types.Slice); ok {
		return
	}
	for _, iface := range selectedInterfaces {
		xIsIface := strings.Contains(typeMap[identX].Type.String(), iface)
		xIsNil := typeMap[identX].IsNil()
		yisIface := strings.Contains(typeMap[identY].Type.String(), iface)
		yIsNil := typeMap[identY].IsNil()
		// Exit early if neither are of the desired interface
		if !xIsIface && !yisIface {
			continue
		}
		if xIsIface && yIsNil {
			reportFailure(identX.Pos(), pass)
		}
		if yisIface && xIsNil {
			reportFailure(identY.Pos(), pass)
		}
	}
}

func reportFailure(pos token.Pos, pass *analysis.Pass) {
	pass.Reportf(pos, "A single nilness check is being performed on an interface"+
		", this check needs another accompanying nilness check on the underlying object for the interface.")
}
