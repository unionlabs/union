package codegen

import (
	"go/ast"
)

// FuncGen is a utility for generating/patching golang function declaration ASTs.
type FuncGen struct {
	*FileGen
	Func *ast.FuncDecl
}

func newFuncGen(fileGen *FileGen, f *ast.FuncDecl) *FuncGen {
	g := &FuncGen{FileGen: fileGen, Func: f}

	// reserve param idents
	for _, field := range f.Type.Params.List {
		for _, name := range field.Names {
			g.idents[name.Name] = true
		}
	}

	// reserve result
	for _, field := range f.Type.Results.List {
		for _, name := range field.Names {
			g.idents[name.Name] = true
		}
	}

	return g
}
