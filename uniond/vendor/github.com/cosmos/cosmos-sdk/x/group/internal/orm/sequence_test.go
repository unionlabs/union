package orm

import (
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSequenceUniqueConstraint(t *testing.T) {
	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	seq := NewSequence(0x1)
	err := seq.InitVal(store, 2)
	require.NoError(t, err)
	err = seq.InitVal(store, 3)
	require.True(t, errors.ErrORMUniqueConstraint.Is(err))
}

func TestSequenceIncrements(t *testing.T) {
	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	seq := NewSequence(0x1)
	var i uint64
	for i = 1; i < 10; i++ {
		autoID := seq.NextVal(store)
		assert.Equal(t, i, autoID)
		assert.Equal(t, i, seq.CurVal(store))
	}

	seq = NewSequence(0x1)
	assert.Equal(t, uint64(10), seq.PeekNextVal(store))
	assert.Equal(t, uint64(9), seq.CurVal(store))
}
