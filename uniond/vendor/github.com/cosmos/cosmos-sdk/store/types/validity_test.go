package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/types"
)

func TestAssertValidKey(t *testing.T) {
	t.Parallel()
	require.NotPanics(t, func() { types.AssertValidKey([]byte{0x01}) })
	require.Panics(t, func() { types.AssertValidKey([]byte{}) })
	require.Panics(t, func() { types.AssertValidKey(nil) })
}

func TestAssertValidValue(t *testing.T) {
	t.Parallel()
	require.NotPanics(t, func() { types.AssertValidValue([]byte{}) })
	require.NotPanics(t, func() { types.AssertValidValue([]byte{0x01}) })
	require.Panics(t, func() { types.AssertValidValue(nil) })
}
