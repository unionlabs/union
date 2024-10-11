package unsafetests_test

import (
	"fmt"
	"testing"
	"unsafe"

	"github.com/stretchr/testify/assert"
)

type ignoreTestingT struct{}

var _ assert.TestingT = ignoreTestingT{}

func (ignoreTestingT) Helper() {}

func (ignoreTestingT) Errorf(format string, args ...interface{}) {
	// Run the formatting, but ignore the result
	msg := fmt.Sprintf(format, args...)
	_ = msg
}

func TestUnsafePointers(t *testing.T) {
	var ignore ignoreTestingT

	assert.True(t, assert.Nil(t, unsafe.Pointer(nil), "unsafe.Pointer(nil) is nil"))
	assert.False(t, assert.NotNil(ignore, unsafe.Pointer(nil), "unsafe.Pointer(nil) is nil"))

	assert.True(t, assert.Nil(t, unsafe.Pointer((*int)(nil)), "unsafe.Pointer((*int)(nil)) is nil"))
	assert.False(t, assert.NotNil(ignore, unsafe.Pointer((*int)(nil)), "unsafe.Pointer((*int)(nil)) is nil"))

	assert.False(t, assert.Nil(ignore, unsafe.Pointer(new(int)), "unsafe.Pointer(new(int)) is NOT nil"))
	assert.True(t, assert.NotNil(t, unsafe.Pointer(new(int)), "unsafe.Pointer(new(int)) is NOT nil"))
}
