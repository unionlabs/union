package depinject

import (
	"os"
	"reflect"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/depinject/internal/graphviz"
)

func TestCheckIsExportedType(t *testing.T) {
	expectValidType(t, false)
	expectValidType(t, uint(0))
	expectValidType(t, uint8(0))
	expectValidType(t, uint16(0))
	expectValidType(t, uint32(0))
	expectValidType(t, uint64(0))
	expectValidType(t, int(0))
	expectValidType(t, int8(0))
	expectValidType(t, int16(0))
	expectValidType(t, int32(0))
	expectValidType(t, int64(0))
	expectValidType(t, float32(0))
	expectValidType(t, float64(0))
	expectValidType(t, complex64(0))
	expectValidType(t, complex128(0))
	expectValidType(t, os.FileMode(0))
	expectValidType(t, [1]int{0})
	expectValidType(t, []int{})
	expectValidType(t, "")
	expectValidType(t, make(chan int))
	expectValidType(t, make(<-chan int))
	expectValidType(t, make(chan<- int))
	expectValidType(t, func(int, string) (bool, error) { return false, nil })
	expectValidType(t, func(int, ...string) (bool, error) { return false, nil })
	expectValidType(t, In{})
	expectValidType(t, map[string]In{})
	expectValidType(t, &In{})
	expectValidType(t, uintptr(0))
	expectValidType(t, (*Location)(nil))

	expectInvalidType(t, container{}, "must be exported")
	expectInvalidType(t, &container{}, "must be exported")
	expectInvalidType(t, graphviz.Attributes{}, "internal")
	expectInvalidType(t, map[string]graphviz.Attributes{}, "internal")
	expectInvalidType(t, []graphviz.Attributes{}, "internal")
}

func expectValidType(t *testing.T, v interface{}) {
	t.Helper()
	assert.NilError(t, isExportedType(reflect.TypeOf(v)))
}

func expectInvalidType(t *testing.T, v interface{}, errContains string) {
	t.Helper()
	assert.ErrorContains(t, isExportedType(reflect.TypeOf(v)), errContains)
}
