package codegen

import (
	"fmt"
	"go/ast"
)

// CreateIdent creates a new ident that doesn't conflict with reserved symbols,
// top-level declarations and other defined idents. Idents are unique across
// the whole file as it is assumed that codegen usually happens on one function
// per file.
func (g *FileGen) CreateIdent(namePrefix string) *ast.Ident {
	return ast.NewIdent(g.doCreateIdent(namePrefix))
}

func (g *FileGen) doCreateIdent(namePrefix string) string {
	v := namePrefix
	i := 2
	for {
		_, ok := g.idents[v]
		if !ok {
			g.idents[v] = true
			return v
		}

		v = fmt.Sprintf("%s%d", namePrefix, i)
		i++
	}
}
