package main

import (
	"go/ast"
	"go/types"
)

// walkForType walks the AST tree and returns the type of the expression
func walkForType(pkgInfo *types.Info, node ast.Node) types.Type {
	var result types.Type

	visit := func(node ast.Node) bool {
		if expr, ok := node.(ast.Expr); ok {
			if typeAndValue, ok := pkgInfo.Types[expr]; ok {
				result = typeAndValue.Type
				return false
			}
		}
		return true
	}
	ast.Inspect(node, visit)
	return result
}

func isUnknownType(typ types.Type) bool {
	if typ == nil {
		return true
	}
	basic, ok := typ.(*types.Basic)
	return ok && basic.Kind() == types.Invalid
}
