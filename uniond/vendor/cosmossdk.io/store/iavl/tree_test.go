package iavl

import (
	"testing"

	"github.com/cosmos/iavl"
	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
)

func TestImmutableTreePanics(t *testing.T) {
	t.Parallel()
	immTree := iavl.NewImmutableTree(coretesting.NewMemDB(), 100, false, log.NewNopLogger())
	it := &immutableTree{immTree}
	require.Panics(t, func() {
		_, err := it.Set([]byte{}, []byte{})
		require.NoError(t, err)
	})
	require.Panics(t, func() {
		_, _, err := it.Remove([]byte{})
		require.NoError(t, err)
	})
	require.Panics(t, func() { _, _, _ = it.SaveVersion() })
	require.Panics(t, func() { _ = it.DeleteVersionsTo(int64(1)) })

	val, err := it.GetVersioned(nil, 1)
	require.Error(t, err)
	require.Nil(t, val)

	imm, err := it.GetImmutable(1)
	require.Error(t, err)
	require.Nil(t, imm)

	imm, err = it.GetImmutable(0)
	require.NoError(t, err)
	require.NotNil(t, imm)
	require.Equal(t, immTree, imm)
}
