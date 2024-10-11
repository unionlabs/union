package kv

import (
	"context"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_DepositContract(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()
	contractAddress := common.Address{1, 2, 3}
	retrieved, err := db.DepositContractAddress(ctx)
	require.NoError(t, err)
	assert.DeepEqual(t, []uint8(nil), retrieved, "Expected nil contract address")
	require.NoError(t, db.SaveDepositContractAddress(ctx, contractAddress))
	retrieved, err = db.DepositContractAddress(ctx)
	require.NoError(t, err)
	assert.Equal(t, contractAddress, common.BytesToAddress(retrieved), "Unexpected address")
	otherAddress := common.Address{4, 5, 6}
	err = db.SaveDepositContractAddress(ctx, otherAddress)
	want := "cannot override deposit contract address"
	assert.ErrorContains(t, want, err, "Should not have been able to override old deposit contract address")
}
