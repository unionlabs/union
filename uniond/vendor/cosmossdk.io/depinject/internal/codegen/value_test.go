package codegen

import (
	"go/ast"
	"reflect"
	"testing"

	"gotest.tools/v3/assert"
)

func TestValueExpr(t *testing.T) {
	// bool
	expectValueExpr(t, true, `true`)
	expectValueExpr(t, false, `false`)

	// uints
	expectValueExpr(t, uint(0), `0`)
	expectValueExpr(t, uint8(1), `1`)
	expectValueExpr(t, uint16(2), `2`)
	expectValueExpr(t, uint32(3), `3`)
	expectValueExpr(t, uint64(12345678), `12345678`)

	// ints
	expectValueExpr(t, 0, `0`)
	expectValueExpr(t, int8(-1), `-1`)
	expectValueExpr(t, int16(-2), `-2`)
	expectValueExpr(t, int32(-3), `-3`)
	expectValueExpr(t, int64(-12345678), `-12345678`)

	// floats
	expectValueExpr(t, float32(0.0), `0e+00`)
	expectValueExpr(t, float64(1.32e-9), `1.32e-09`)

	// complex
	expectValueExpr(t, complex64(1+2i), `(1e+00+2e+00i)`)
	expectValueExpr(t, complex128(1.32e-9+-3.03i), `(1.32e-09-3.03e+00i)`)

	// array
	expectValueExpr(t, [3]uint32{1, 4, 9}, `[3]uint32{1, 4, 9}`)

	// slice
	expectValueExpr(t, []uint32{1, 4, 9}, `[]uint32{1, 4, 9}`)

	// map
	expectValueExpr(t, map[string]int{"a": 1}, `map[string]int{"a": 1}`)

	// struct
	expectValueExpr(t, AStruct{Foo: 2}, `codegen.AStruct{Foo: 2}`)
	expectValueExpr(t, AStruct{}, `codegen.AStruct{}`) // empty default fields

	// struct pointer
	expectValueExpr(t, &AStruct{Foo: 2}, `&codegen.AStruct{Foo: 2}`)
	var nilStruct *AStruct
	expectValueExpr(t, nilStruct, `nil`)

	// struct wrapper
	expectValueExpr(t, &AStructWrapper{Foo: 2}, `&codegen.AStructWrapper{Foo: 2}`)

	// string
	expectValueExpr(t, "abc", `"abc"`)
}

func expectValueExpr(t *testing.T, value interface{}, expected string) {
	t.Helper()
	g, err := NewFileGen(&ast.File{}, "")
	assert.NilError(t, err)
	e, err := g.ValueExpr(reflect.ValueOf(value))
	assert.NilError(t, err)
	expectExpr(t, e, expected)
}
