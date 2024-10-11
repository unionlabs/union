package transition_test

import (
	"context"
	"math"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	p2pType "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestExecuteAltairStateTransitionNoVerify_FullProcess(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, 100)

	syncCommittee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(syncCommittee))

	eth1Data := &ethpb.Eth1Data{
		DepositCount: 100,
		DepositRoot:  bytesutil.PadTo([]byte{2}, 32),
		BlockHash:    make([]byte, 32),
	}
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch-1))
	e := beaconState.Eth1Data()
	e.DepositCount = 100
	require.NoError(t, beaconState.SetEth1Data(e))
	bh := beaconState.LatestBlockHeader()
	bh.Slot = beaconState.Slot()
	require.NoError(t, beaconState.SetLatestBlockHeader(bh))
	require.NoError(t, beaconState.SetEth1DataVotes([]*ethpb.Eth1Data{eth1Data}))

	require.NoError(t, beaconState.SetSlot(beaconState.Slot()+1))
	epoch := time.CurrentEpoch(beaconState)
	randaoReveal, err := util.RandaoReveal(beaconState, epoch, privKeys)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlot(beaconState.Slot()-1))

	nextSlotState, err := transition.ProcessSlots(context.Background(), beaconState.Copy(), beaconState.Slot()+1)
	require.NoError(t, err)
	parentRoot, err := nextSlotState.LatestBlockHeader().HashTreeRoot()
	require.NoError(t, err)
	proposerIdx, err := helpers.BeaconProposerIndex(context.Background(), nextSlotState)
	require.NoError(t, err)
	block := util.NewBeaconBlockAltair()
	block.Block.ProposerIndex = proposerIdx
	block.Block.Slot = beaconState.Slot() + 1
	block.Block.ParentRoot = parentRoot[:]
	block.Block.Body.RandaoReveal = randaoReveal
	block.Block.Body.Eth1Data = eth1Data

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xff
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	h := ethpb.CopyBeaconBlockHeader(beaconState.LatestBlockHeader())
	prevStateRoot, err := beaconState.HashTreeRoot(context.Background())
	require.NoError(t, err)
	h.StateRoot = prevStateRoot[:]
	pbr, err := h.HashTreeRoot()
	require.NoError(t, err)
	syncSigs := make([]bls.Signature, len(indices))
	for i, indice := range indices {
		b := p2pType.SSZBytes(pbr[:])
		sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		syncSigs[i] = sig
	}
	aggregatedSig := bls.AggregateSignatures(syncSigs).Marshal()
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits:      syncBits,
		SyncCommitteeSignature: aggregatedSig,
	}
	block.Block.Body.SyncAggregate = syncAggregate
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	stateRoot, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	block.Block.StateRoot = stateRoot[:]

	c := beaconState.Copy()
	sig, err := util.BlockSignatureAltair(c, block.Block, privKeys)
	require.NoError(t, err)
	block.Signature = sig.Marshal()

	wsb, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	set, _, err := transition.ExecuteStateTransitionNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	verified, err := set.Verify()
	require.NoError(t, err)
	require.Equal(t, true, verified, "Could not verify signature set")
}

func TestExecuteAltairStateTransitionNoVerifySignature_CouldNotVerifyStateRoot(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, 100)

	syncCommittee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(syncCommittee))

	eth1Data := &ethpb.Eth1Data{
		DepositCount: 100,
		DepositRoot:  bytesutil.PadTo([]byte{2}, 32),
		BlockHash:    make([]byte, 32),
	}
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch-1))
	e := beaconState.Eth1Data()
	e.DepositCount = 100
	require.NoError(t, beaconState.SetEth1Data(e))
	bh := beaconState.LatestBlockHeader()
	bh.Slot = beaconState.Slot()
	require.NoError(t, beaconState.SetLatestBlockHeader(bh))
	require.NoError(t, beaconState.SetEth1DataVotes([]*ethpb.Eth1Data{eth1Data}))

	require.NoError(t, beaconState.SetSlot(beaconState.Slot()+1))
	epoch := time.CurrentEpoch(beaconState)
	randaoReveal, err := util.RandaoReveal(beaconState, epoch, privKeys)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlot(beaconState.Slot()-1))

	nextSlotState, err := transition.ProcessSlots(context.Background(), beaconState.Copy(), beaconState.Slot()+1)
	require.NoError(t, err)
	parentRoot, err := nextSlotState.LatestBlockHeader().HashTreeRoot()
	require.NoError(t, err)
	proposerIdx, err := helpers.BeaconProposerIndex(context.Background(), nextSlotState)
	require.NoError(t, err)
	block := util.NewBeaconBlockAltair()
	block.Block.ProposerIndex = proposerIdx
	block.Block.Slot = beaconState.Slot() + 1
	block.Block.ParentRoot = parentRoot[:]
	block.Block.Body.RandaoReveal = randaoReveal
	block.Block.Body.Eth1Data = eth1Data

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xff
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	h := ethpb.CopyBeaconBlockHeader(beaconState.LatestBlockHeader())
	prevStateRoot, err := beaconState.HashTreeRoot(context.Background())
	require.NoError(t, err)
	h.StateRoot = prevStateRoot[:]
	pbr, err := h.HashTreeRoot()
	require.NoError(t, err)
	syncSigs := make([]bls.Signature, len(indices))
	for i, indice := range indices {
		b := p2pType.SSZBytes(pbr[:])
		sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		syncSigs[i] = sig
	}
	aggregatedSig := bls.AggregateSignatures(syncSigs).Marshal()
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits:      syncBits,
		SyncCommitteeSignature: aggregatedSig,
	}
	block.Block.Body.SyncAggregate = syncAggregate

	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	stateRoot, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	block.Block.StateRoot = stateRoot[:]

	c := beaconState.Copy()
	sig, err := util.BlockSignatureAltair(c, block.Block, privKeys)
	require.NoError(t, err)
	block.Signature = sig.Marshal()

	block.Block.StateRoot = bytesutil.PadTo([]byte{'a'}, 32)
	wsb, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	_, _, err = transition.ExecuteStateTransitionNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.ErrorContains(t, "could not validate state root", err)
}

func TestExecuteStateTransitionNoVerifyAnySig_PassesProcessingConditions(t *testing.T) {
	beaconState, block := createFullAltairBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	set, _, err := transition.ExecuteStateTransitionNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	// Test Signature set verifies.
	verified, err := set.Verify()
	require.NoError(t, err)
	require.Equal(t, true, verified, "Could not verify signature set")
}

func TestProcessEpoch_BadBalanceAltair(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, 100)
	assert.NoError(t, s.SetSlot(63))
	assert.NoError(t, s.UpdateBalancesAtIndex(0, math.MaxUint64))
	participation := byte(0)
	participation, err := altair.AddValidatorFlag(participation, params.BeaconConfig().TimelyHeadFlagIndex)
	require.NoError(t, err)
	participation, err = altair.AddValidatorFlag(participation, params.BeaconConfig().TimelySourceFlagIndex)
	require.NoError(t, err)
	participation, err = altair.AddValidatorFlag(participation, params.BeaconConfig().TimelyTargetFlagIndex)
	require.NoError(t, err)

	epochParticipation, err := s.CurrentEpochParticipation()
	assert.NoError(t, err)
	epochParticipation[0] = participation
	assert.NoError(t, s.SetCurrentParticipationBits(epochParticipation))
	assert.NoError(t, s.SetPreviousParticipationBits(epochParticipation))
	_, err = altair.ProcessEpoch(context.Background(), s)
	assert.ErrorContains(t, "addition overflows", err)
}

func createFullAltairBlockWithOperations(t *testing.T) (state.BeaconState,
	*ethpb.SignedBeaconBlockAltair) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, 32)
	sCom, err := altair.NextSyncCommittee(context.Background(), beaconState)
	assert.NoError(t, err)
	assert.NoError(t, beaconState.SetCurrentSyncCommittee(sCom))
	tState := beaconState.Copy()
	blk, err := util.GenerateFullBlockAltair(tState, privKeys,
		&util.BlockGenConfig{NumAttestations: 1, NumVoluntaryExits: 0, NumDeposits: 0}, 1)
	require.NoError(t, err)

	return beaconState, blk
}
