package codegen

import (
	"go/parser"
	"go/token"
	"testing"

	"gotest.tools/v3/assert"
)

func TestImport(t *testing.T) {
	const badFileSrc = `
package mypkg

import . "example.com/foo"
`

	badFile, err := parser.ParseFile(token.NewFileSet(), "", badFileSrc, 0)
	assert.NilError(t, err)

	_, err = NewFileGen(badFile, "example.com/mypkg")
	assert.ErrorContains(t, err, ".")

	const goodFileSrc = `
package mypkg

import "example.com/foo"
import abc "example.com/bar"
`

	goodFile, err := parser.ParseFile(token.NewFileSet(), "", goodFileSrc, 0)
	assert.NilError(t, err)
	assert.Equal(t, 2, len(goodFile.Imports))

	fgen, err := NewFileGen(goodFile, "example.com/mypkg")
	assert.NilError(t, err)

	// self import is ""
	assert.Equal(t, "", fgen.AddOrGetImport("example.com/mypkg"))

	// bar import is abc, no new import was added
	assert.Equal(t, "abc", fgen.AddOrGetImport("example.com/bar"))
	assert.Equal(t, 2, len(goodFile.Imports))

	// foo import is foo, no new import is added
	assert.Equal(t, "foo", fgen.AddOrGetImport("example.com/foo"))
	assert.Equal(t, 2, len(goodFile.Imports))

	// baz import is baz, a new import is added
	assert.Equal(t, "baz", fgen.AddOrGetImport("example.com/baz"))
	assert.Equal(t, 3, len(goodFile.Imports))

	// another foo import is foo2, a new import is added
	assert.Equal(t, "foo2", fgen.AddOrGetImport("example2.com/foo"))
	assert.Equal(t, 4, len(goodFile.Imports))

	// another baz import is baz2, a new import is added
	assert.Equal(t, "baz2", fgen.AddOrGetImport("example.com/foo/baz"))
	assert.Equal(t, 5, len(goodFile.Imports))
}
