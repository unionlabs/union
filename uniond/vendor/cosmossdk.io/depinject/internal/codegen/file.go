package codegen

import (
	"fmt"
	"go/ast"
	"go/token"
	"strconv"
)

// FileGen is a utility for generating/patching golang file ASTs.
type FileGen struct {
	File           *ast.File
	idents         map[string]bool
	codegenPkgPath string
	pkgImportMap   map[string]*importInfo
}

// NewFileGen creates a new FileGen instance from a file AST with the provided package path.
func NewFileGen(file *ast.File, codegenPkgPath string) (*FileGen, error) {
	g := &FileGen{
		File:           file,
		idents:         map[string]bool{},
		codegenPkgPath: codegenPkgPath,
		pkgImportMap:   map[string]*importInfo{},
	}

	// add all go keywords to reserved idents
	for i := token.Token(0); i <= token.TILDE; i++ {
		name := i.String()
		if token.IsKeyword(name) {
			g.idents[name] = true
		}
	}

	// add all top-level decl idents
	for _, decl := range file.Decls {
		switch decl := decl.(type) {
		case *ast.FuncDecl:
			g.idents[decl.Name.Name] = true
		case *ast.GenDecl:
			for _, spec := range decl.Specs {
				switch spec := spec.(type) {
				case *ast.TypeSpec:
					g.idents[spec.Name.Name] = true
				case *ast.ValueSpec:
					for _, name := range spec.Names {
						g.idents[name.Name] = true
					}
				}
			}
		}
	}

	for _, spec := range file.Imports {
		pkgPath, err := strconv.Unquote(spec.Path.Value)
		if err != nil {
			return nil, err
		}

		var info *importInfo
		if spec.Name != nil {
			name := spec.Name.Name
			if name == "." {
				return nil, fmt.Errorf(". package imports are not allowed")
			}

			info = &importInfo{importPrefix: name, ImportSpec: spec}
		} else {
			prefix := defaultPkgPrefix(pkgPath)
			info = &importInfo{importPrefix: prefix, ImportSpec: spec}
		}
		g.pkgImportMap[pkgPath] = info
		g.idents[info.importPrefix] = true
	}

	return g, nil
}

// PatchFuncDecl returns a FuncGen instance for the function declaration with the given name or returns nil.
func (g *FileGen) PatchFuncDecl(name string) *FuncGen {
	for _, decl := range g.File.Decls {
		funcDecl, ok := decl.(*ast.FuncDecl)
		if ok {
			if funcDecl.Name.Name == name {
				return newFuncGen(g, funcDecl)
			}
		}
	}
	return nil
}
