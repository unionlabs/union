package interop_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGenerateGenesisState(t *testing.T) {
	numValidators := uint64(64)
	privKeys, pubKeys, err := interop.DeterministicallyGenerateKeys(0 /*startIndex*/, numValidators)
	require.NoError(t, err)
	depositDataItems, depositDataRoots, err := interop.DepositDataFromKeys(privKeys, pubKeys)
	require.NoError(t, err)
	tr, err := trie.GenerateTrieFromItems(depositDataRoots, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err)
	deposits, err := interop.GenerateDepositsFromData(depositDataItems, tr)
	require.NoError(t, err)
	root, err := tr.HashTreeRoot()
	require.NoError(t, err)
	genesisState, err := transition.GenesisBeaconState(context.Background(), deposits, 0, &eth.Eth1Data{
		DepositRoot:  root[:],
		DepositCount: uint64(len(deposits)),
	})
	require.NoError(t, err)
	want := int(numValidators)
	assert.Equal(t, want, genesisState.NumValidators())
	assert.Equal(t, uint64(0), genesisState.GenesisTime())
}
