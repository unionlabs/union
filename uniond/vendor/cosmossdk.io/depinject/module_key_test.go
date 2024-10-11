package depinject

import (
	"testing"

	"gotest.tools/v3/assert"
)

func TestModuleKeyEquals(t *testing.T) {
	ctx := &ModuleKeyContext{}

	fooKey := ctx.For("foo")
	fooKey2 := ctx.For("foo")
	// two foo keys from the same context should be equal
	assert.Assert(t, fooKey.Equals(fooKey2))

	barKey := ctx.For("bar")
	// foo and bar keys should be not equal
	assert.Assert(t, !fooKey.Equals(barKey))

	ctx2 := &ModuleKeyContext{}
	fooKeyFromAnotherCtx := ctx2.For("foo")
	// foo keys from different context should be not equal
	assert.Assert(t, !fooKey.Equals(fooKeyFromAnotherCtx))
}
