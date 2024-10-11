// Package shadowpredecl implements a static analyzer which disallows declaring constructs
// that shadow predeclared Go identifiers by having the same name.
package shadowpredecl

import (
	"errors"
	"go/ast"
	"go/token"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to detect declarations that shadow predeclared identifiers by having the same name."

const messageTemplate = "%s '%s' shadows a predeclared identifier with the same name. Choose another name."

// Aligned with https://golang.org/ref/spec#Predeclared_identifiers
var predeclared = []string{"bool", "byte", "complex64", "complex128", "error", "float32", "float64", "int", "int8",
	"int16", "int32", "int64", "rune", "string", "uint", "uint8", "uint16", "uint32", "uint64", "uintptr", "true",
	"false", "iota", "nil", "append", "cap", "close", "complex", "copy", "delete", "imag", "len", "make", "new",
	"panic", "print", "println", "real", "recover"}

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "shadowpredecl",
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
		(*ast.FuncDecl)(nil),
		(*ast.FuncLit)(nil),
		(*ast.AssignStmt)(nil),
		(*ast.TypeSpec)(nil),
		(*ast.ValueSpec)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		switch declaration := node.(type) {
		case *ast.FuncDecl:
			if declaration.Recv != nil {
				return
			}
			name := declaration.Name.Name
			if shadows(name) {
				pass.Reportf(declaration.Name.NamePos, messageTemplate, "Function", name)
			}
			inspectFunctionParams(pass, declaration.Type.Params.List)
		case *ast.FuncLit:
			inspectFunctionParams(pass, declaration.Type.Params.List)
		case *ast.AssignStmt:
			if declaration.Tok == token.ASSIGN {
				return
			}
			for _, expr := range declaration.Lhs {
				if identifier, ok := expr.(*ast.Ident); ok {
					name := identifier.Name
					if shadows(name) {
						pass.Reportf(identifier.NamePos, messageTemplate, "Identifier", name)
					}
				}
			}
		case *ast.TypeSpec:
			name := declaration.Name.Name
			if shadows(name) {
				pass.Reportf(declaration.Name.NamePos, messageTemplate, "Type", name)
			}
		case *ast.ValueSpec:
			for _, identifier := range declaration.Names {
				name := identifier.Name
				if shadows(name) {
					pass.Reportf(identifier.NamePos, messageTemplate, "Identifier", name)
				}
			}
		}
	})

	return nil, nil
}

func inspectFunctionParams(pass *analysis.Pass, paramList []*ast.Field) {
	for _, field := range paramList {
		for _, identifier := range field.Names {
			name := identifier.Name
			if shadows(name) {
				pass.Reportf(identifier.NamePos, messageTemplate, "Identifier", name)
			}
		}
	}
}

func shadows(name string) bool {
	for _, identifier := range predeclared {
		if identifier == name {
			return true
		}
	}
	return false
}
