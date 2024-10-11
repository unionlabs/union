package interop

import (
	"context"
	"testing"

	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGenerateGenesisStateBellatrix(t *testing.T) {
	ep := &enginev1.ExecutionPayload{
		ParentHash:    make([]byte, 32),
		FeeRecipient:  make([]byte, 20),
		StateRoot:     make([]byte, 32),
		ReceiptsRoot:  make([]byte, 32),
		LogsBloom:     make([]byte, 256),
		PrevRandao:    make([]byte, 32),
		BlockNumber:   0,
		GasLimit:      0,
		GasUsed:       0,
		Timestamp:     0,
		ExtraData:     make([]byte, 32),
		BaseFeePerGas: make([]byte, 32),
		BlockHash:     make([]byte, 32),
		Transactions:  make([][]byte, 0),
	}
	e1d := &ethpb.Eth1Data{
		DepositRoot:  make([]byte, 32),
		DepositCount: 0,
		BlockHash:    make([]byte, 32),
	}
	g, _, err := GenerateGenesisStateBellatrix(context.Background(), 0, params.BeaconConfig().MinGenesisActiveValidatorCount, ep, e1d)
	require.NoError(t, err)

	tr, err := trie.NewTrie(params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err)
	dr, err := tr.HashTreeRoot()
	require.NoError(t, err)
	g.Eth1Data.DepositRoot = dr[:]
	g.Eth1Data.BlockHash = make([]byte, 32)
	st, err := state_native.InitializeFromProtoUnsafeBellatrix(g)
	require.NoError(t, err)
	_, err = st.MarshalSSZ()
	require.NoError(t, err)
}
