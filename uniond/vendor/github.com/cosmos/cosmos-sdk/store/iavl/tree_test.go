package iavl

import (
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cosmos/iavl"
	"github.com/stretchr/testify/require"
)

func TestImmutableTreePanics(t *testing.T) {
	t.Parallel()
	immTree := iavl.NewImmutableTree(dbm.NewMemDB(), 100, false)
	it := &immutableTree{immTree}
	require.Panics(t, func() { it.Set([]byte{}, []byte{}) })
	require.Panics(t, func() { it.Remove([]byte{}) })
	require.Panics(t, func() { it.SaveVersion() })           // nolint:errcheck
	require.Panics(t, func() { it.DeleteVersion(int64(1)) }) // nolint:errcheck

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
