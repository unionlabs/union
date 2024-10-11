package codegen

import (
	"go/parser"
	"go/token"
	"testing"

	"gotest.tools/v3/assert"
)

func TestReservedIdents(t *testing.T) {
	const src = `
package mypkg

import "example.com/bar"

type MyInt int

var myVar int

func Build(a, a2 int, b string) (c int, err error) {}
`
	file, err := parser.ParseFile(token.NewFileSet(), "", src, 0)
	assert.NilError(t, err)

	fileGen, err := NewFileGen(file, "example.com/mypkg")
	assert.NilError(t, err)
	funcGen := fileGen.PatchFuncDecl("Build")
	assert.Assert(t, funcGen != nil)

	// go keywords get a suffix
	assert.Equal(t, "type2", funcGen.CreateIdent("type").Name)
	assert.Equal(t, "package2", funcGen.CreateIdent("package").Name)
	assert.Equal(t, "goto2", funcGen.CreateIdent("goto").Name)

	// import name prefixes get suffixes
	assert.Equal(t, "bar2", funcGen.CreateIdent("bar").Name)

	// top-level decl names get prefixes
	assert.Equal(t, "MyInt2", funcGen.CreateIdent("MyInt").Name)
	assert.Equal(t, "myVar2", funcGen.CreateIdent("myVar").Name)

	// param and result names get suffixes at func level
	assert.Equal(t, "a3", funcGen.CreateIdent("a").Name)
	assert.Equal(t, "b2", funcGen.CreateIdent("b").Name)
	assert.Equal(t, "c2", funcGen.CreateIdent("c").Name)
	assert.Equal(t, "err2", funcGen.CreateIdent("err").Name)
}
