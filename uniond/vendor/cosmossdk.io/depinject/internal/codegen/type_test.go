package codegen

import (
	"bytes"
	"go/ast"
	"go/printer"
	"go/token"
	"reflect"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/depinject/internal/graphviz"
)

type MyInt int

type AStruct struct {
	Foo int
}

type AGenericStruct[A, B any] struct {
	A A
	B B
}

type AStructWrapper AStruct

type AnInterface interface{}

func TestTypeExpr(t *testing.T) {
	expectTypeExpr(t, false, "bool")
	expectTypeExpr(t, uint(0), "uint")
	expectTypeExpr(t, uint8(0), "uint8")
	expectTypeExpr(t, uint16(0), "uint16")
	expectTypeExpr(t, uint32(0), "uint32")
	expectTypeExpr(t, uint64(0), "uint64")
	expectTypeExpr(t, int(0), "int")
	expectTypeExpr(t, int8(0), "int8")
	expectTypeExpr(t, int16(0), "int16")
	expectTypeExpr(t, int32(0), "int32")
	expectTypeExpr(t, int64(0), "int64")
	expectTypeExpr(t, float32(0), "float32")
	expectTypeExpr(t, float64(0), "float64")
	expectTypeExpr(t, complex64(0), "complex64")
	expectTypeExpr(t, complex128(0), "complex128")
	expectTypeExpr(t, MyInt(0), "codegen.MyInt")
	expectTypeExpr(t, [1]int{0}, "[1]int")
	expectTypeExpr(t, []int{}, "[]int")
	expectTypeExpr(t, make(chan int), "chan int")
	expectTypeExpr(t, make(<-chan int), "<-chan int")
	expectTypeExpr(t, make(chan<- int), "chan<- int")
	expectTypeExpr(t, func(int, string) (bool, error) { return false, nil },
		"func(int, string) (bool, error)",
	)
	expectTypeExpr(t, func(int, ...string) (bool, error) { return false, nil },
		"func(int, ...string) (bool, error)",
	)
	expectTypeExpr(t, AStruct{}, "codegen.AStruct")
	expectTypeExpr(t, map[string]graphviz.Attributes{}, "map[string]graphviz.Attributes")
	expectTypeExpr(t, &AStruct{}, "*codegen.AStruct")
	expectTypeExpr(t, AGenericStruct[graphviz.Node, FileGen]{}, "codegen.AGenericStruct[graphviz.Node, codegen.FileGen]")
	expectTypeExpr(t, AStructWrapper{}, "codegen.AStructWrapper")
	expectTypeExpr(t, "abc", "string")
	expectTypeExpr(t, uintptr(0), "uintptr")
	expectTypeExpr(t, (*AnInterface)(nil), "*codegen.AnInterface")
}

func expectTypeExpr(t *testing.T, value interface{}, expected string) {
	t.Helper()
	g, err := NewFileGen(&ast.File{}, "")
	assert.NilError(t, err)
	e, err := g.TypeExpr(reflect.TypeOf(value))
	assert.NilError(t, err)
	expectExpr(t, e, expected)
}

func expectExpr(t *testing.T, e ast.Expr, expected string) {
	t.Helper()
	fset := token.NewFileSet()
	buf := &bytes.Buffer{}
	assert.NilError(t, printer.Fprint(buf, fset, e))
	errBuf := &bytes.Buffer{}
	assert.NilError(t, ast.Fprint(errBuf, fset, e, nil))
	assert.Equal(t, expected, buf.String(), errBuf.String())
}
