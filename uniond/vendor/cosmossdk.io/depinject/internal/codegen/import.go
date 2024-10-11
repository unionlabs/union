package codegen

import (
	"fmt"
	"go/ast"
	"go/token"
	"strings"
)

type importInfo struct {
	*ast.ImportSpec
	importPrefix string
}

// AddOrGetImport adds a new import for the provided pkgPath (if needed) and
// returns the unique import prefix for that path.
func (g *FileGen) AddOrGetImport(pkgPath string) (importPrefix string) {
	if pkgPath == "" || pkgPath == g.codegenPkgPath {
		return ""
	}

	if i, ok := g.pkgImportMap[pkgPath]; ok {
		return i.importPrefix
	}

	imp := &ast.ImportSpec{
		Path: &ast.BasicLit{Kind: token.STRING, Value: fmt.Sprintf("%q", pkgPath)},
	}

	defaultPrefix := defaultPkgPrefix(pkgPath)
	importPrefix = g.doCreateIdent(defaultPrefix)

	if importPrefix != defaultPrefix {
		imp.Name = ast.NewIdent(importPrefix)
	}
	g.File.Imports = append(g.File.Imports, imp)
	g.pkgImportMap[pkgPath] = &importInfo{
		ImportSpec:   imp,
		importPrefix: importPrefix,
	}
	g.idents[importPrefix] = true
	return importPrefix
}

func defaultPkgPrefix(pkgPath string) string {
	pkgParts := strings.Split(pkgPath, "/")
	return pkgParts[len(pkgParts)-1]
}
