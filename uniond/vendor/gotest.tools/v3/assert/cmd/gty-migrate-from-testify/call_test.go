package main

import (
	"go/ast"
	"go/token"
	"testing"

	"gotest.tools/v3/assert"
)

func TestCall_String(t *testing.T) {
	c := &call{
		expr: &ast.CallExpr{Fun: ast.NewIdent("myFunc")},
	}
	assert.Equal(t, c.String(), "myFunc()")
}

func TestCall_StringWithFileInfo(t *testing.T) {
	c := &call{
		fileset: token.NewFileSet(),
		expr: &ast.CallExpr{
			Fun: &ast.Ident{
				Name:    "myFunc",
				NamePos: 17,
			}},
	}
	t.Run("unknown file", func(t *testing.T) {
		assert.Equal(t, c.StringWithFileInfo(), "myFunc() at unknown file")
	})

	t.Run("at position", func(t *testing.T) {
		c.fileset.AddFile("source.go", 10, 100)
		assert.Equal(t, c.StringWithFileInfo(), "myFunc() at source.go:1")
	})
}
