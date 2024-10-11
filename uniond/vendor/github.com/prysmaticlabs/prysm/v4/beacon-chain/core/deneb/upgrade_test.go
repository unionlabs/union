package deneb_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/deneb"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestUpgradeToDeneb(t *testing.T) {
	st, _ := util.DeterministicGenesisStateCapella(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, st.SetHistoricalRoots([][]byte{{1}}))
	preForkState := st.Copy()
	mSt, err := deneb.UpgradeToDeneb(st)
	require.NoError(t, err)

	require.Equal(t, preForkState.GenesisTime(), mSt.GenesisTime())
	require.DeepSSZEqual(t, preForkState.GenesisValidatorsRoot(), mSt.GenesisValidatorsRoot())
	require.Equal(t, preForkState.Slot(), mSt.Slot())
	require.DeepSSZEqual(t, preForkState.LatestBlockHeader(), mSt.LatestBlockHeader())
	require.DeepSSZEqual(t, preForkState.BlockRoots(), mSt.BlockRoots())
	require.DeepSSZEqual(t, preForkState.StateRoots(), mSt.StateRoots())
	require.DeepSSZEqual(t, preForkState.Eth1Data(), mSt.Eth1Data())
	require.DeepSSZEqual(t, preForkState.Eth1DataVotes(), mSt.Eth1DataVotes())
	require.DeepSSZEqual(t, preForkState.Eth1DepositIndex(), mSt.Eth1DepositIndex())
	require.DeepSSZEqual(t, preForkState.Validators(), mSt.Validators())
	require.DeepSSZEqual(t, preForkState.Balances(), mSt.Balances())
	require.DeepSSZEqual(t, preForkState.RandaoMixes(), mSt.RandaoMixes())
	require.DeepSSZEqual(t, preForkState.Slashings(), mSt.Slashings())
	require.DeepSSZEqual(t, preForkState.JustificationBits(), mSt.JustificationBits())
	require.DeepSSZEqual(t, preForkState.PreviousJustifiedCheckpoint(), mSt.PreviousJustifiedCheckpoint())
	require.DeepSSZEqual(t, preForkState.CurrentJustifiedCheckpoint(), mSt.CurrentJustifiedCheckpoint())
	require.DeepSSZEqual(t, preForkState.FinalizedCheckpoint(), mSt.FinalizedCheckpoint())
	numValidators := mSt.NumValidators()
	p, err := mSt.PreviousEpochParticipation()
	require.NoError(t, err)
	require.DeepSSZEqual(t, make([]byte, numValidators), p)
	p, err = mSt.CurrentEpochParticipation()
	require.NoError(t, err)
	require.DeepSSZEqual(t, make([]byte, numValidators), p)
	s, err := mSt.InactivityScores()
	require.NoError(t, err)
	require.DeepSSZEqual(t, make([]uint64, numValidators), s)

	hr1, err := preForkState.HistoricalRoots()
	require.NoError(t, err)
	hr2, err := mSt.HistoricalRoots()
	require.NoError(t, err)
	require.DeepEqual(t, hr1, hr2)

	f := mSt.Fork()
	require.DeepSSZEqual(t, &ethpb.Fork{
		PreviousVersion: st.Fork().CurrentVersion,
		CurrentVersion:  params.BeaconConfig().DenebForkVersion,
		Epoch:           time.CurrentEpoch(st),
	}, f)
	csc, err := mSt.CurrentSyncCommittee()
	require.NoError(t, err)
	psc, err := preForkState.CurrentSyncCommittee()
	require.NoError(t, err)
	require.DeepSSZEqual(t, psc, csc)
	nsc, err := mSt.NextSyncCommittee()
	require.NoError(t, err)
	psc, err = preForkState.NextSyncCommittee()
	require.NoError(t, err)
	require.DeepSSZEqual(t, psc, nsc)

	header, err := mSt.LatestExecutionPayloadHeader()
	require.NoError(t, err)
	protoHeader, ok := header.Proto().(*enginev1.ExecutionPayloadHeaderDeneb)
	require.Equal(t, true, ok)
	prevHeader, err := preForkState.LatestExecutionPayloadHeader()
	require.NoError(t, err)
	txRoot, err := prevHeader.TransactionsRoot()
	require.NoError(t, err)

	wdRoot, err := prevHeader.WithdrawalsRoot()
	require.NoError(t, err)
	wanted := &enginev1.ExecutionPayloadHeaderDeneb{
		ParentHash:       prevHeader.ParentHash(),
		FeeRecipient:     prevHeader.FeeRecipient(),
		StateRoot:        prevHeader.StateRoot(),
		ReceiptsRoot:     prevHeader.ReceiptsRoot(),
		LogsBloom:        prevHeader.LogsBloom(),
		PrevRandao:       prevHeader.PrevRandao(),
		BlockNumber:      prevHeader.BlockNumber(),
		GasLimit:         prevHeader.GasLimit(),
		GasUsed:          prevHeader.GasUsed(),
		Timestamp:        prevHeader.Timestamp(),
		ExtraData:        prevHeader.ExtraData(),
		BaseFeePerGas:    prevHeader.BaseFeePerGas(),
		BlockHash:        prevHeader.BlockHash(),
		TransactionsRoot: txRoot,
		WithdrawalsRoot:  wdRoot,
	}
	require.DeepEqual(t, wanted, protoHeader)
}
