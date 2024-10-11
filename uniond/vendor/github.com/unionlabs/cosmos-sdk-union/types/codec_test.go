package types

import (
	"testing"

	"cosmossdk.io/collections/colltest"
	"cosmossdk.io/math"
)

func TestIntValue(t *testing.T) {
	colltest.TestValueCodec(t, IntValue, math.NewInt(10005994859))
}

func TestUintValue(t *testing.T) {
	colltest.TestValueCodec(t, UintValue, math.NewUint(1337))
	colltest.TestValueCodec(t, UintValue, math.ZeroUint())
	colltest.TestValueCodec(t, UintValue, math.NewUintFromString("1000000000000000000"))
}
