package transition_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	field_params "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestExecuteStateTransitionNoVerify_FullProcess(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisState(t, 100)

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
	block := util.NewBeaconBlock()
	block.Block.ProposerIndex = proposerIdx
	block.Block.Slot = beaconState.Slot() + 1
	block.Block.ParentRoot = parentRoot[:]
	block.Block.Body.RandaoReveal = randaoReveal
	block.Block.Body.Eth1Data = eth1Data

	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	stateRoot, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	require.NoError(t, err)

	block.Block.StateRoot = stateRoot[:]

	sig, err := util.BlockSignature(beaconState, block.Block, privKeys)
	require.NoError(t, err)
	block.Signature = sig.Marshal()

	wsb, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	set, _, err := transition.ExecuteStateTransitionNoVerifyAnySig(context.Background(), beaconState, wsb)
	assert.NoError(t, err)
	verified, err := set.Verify()
	assert.NoError(t, err)
	assert.Equal(t, true, verified, "Could not verify signature set")
}

func TestExecuteStateTransitionNoVerifySignature_CouldNotVerifyStateRoot(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisState(t, 100)

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
	block := util.NewBeaconBlock()
	block.Block.ProposerIndex = proposerIdx
	block.Block.Slot = beaconState.Slot() + 1
	block.Block.ParentRoot = parentRoot[:]
	block.Block.Body.RandaoReveal = randaoReveal
	block.Block.Body.Eth1Data = eth1Data

	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	stateRoot, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	require.NoError(t, err)

	block.Block.StateRoot = stateRoot[:]

	sig, err := util.BlockSignature(beaconState, block.Block, privKeys)
	require.NoError(t, err)
	block.Signature = sig.Marshal()

	block.Block.StateRoot = bytesutil.PadTo([]byte{'a'}, 32)
	wsb, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	_, _, err = transition.ExecuteStateTransitionNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.ErrorContains(t, "could not validate state root", err)
}

func TestProcessBlockNoVerify_PassesProcessingConditions(t *testing.T) {
	beaconState, block, _, _, _ := createFullBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	set, _, err := transition.ProcessBlockNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	// Test Signature set verifies.
	verified, err := set.Verify()
	require.NoError(t, err)
	assert.Equal(t, true, verified, "Could not verify signature set.")
}

func TestProcessBlockNoVerifyAnySigAltair_OK(t *testing.T) {
	beaconState, block := createFullAltairBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	beaconState, err = transition.ProcessSlots(context.Background(), beaconState, wsb.Block().Slot())
	require.NoError(t, err)
	set, _, err := transition.ProcessBlockNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	verified, err := set.Verify()
	require.NoError(t, err)
	require.Equal(t, true, verified, "Could not verify signature set")
}

func TestProcessBlockNoVerify_SigSetContainsDescriptions(t *testing.T) {
	beaconState, block, _, _, _ := createFullBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	set, _, err := transition.ProcessBlockNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	assert.Equal(t, len(set.Signatures), len(set.Descriptions), "Signatures and descriptions do not match up")
	assert.Equal(t, "block signature", set.Descriptions[0])
	assert.Equal(t, "randao signature", set.Descriptions[1])
	assert.Equal(t, "attestation signature", set.Descriptions[2])
}

func TestProcessOperationsNoVerifyAttsSigs_OK(t *testing.T) {
	beaconState, block := createFullAltairBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	beaconState, err = transition.ProcessSlots(context.Background(), beaconState, wsb.Block().Slot())
	require.NoError(t, err)
	_, err = transition.ProcessOperationsNoVerifyAttsSigs(context.Background(), beaconState, wsb.Block())
	require.NoError(t, err)
}

func TestProcessOperationsNoVerifyAttsSigsBellatrix_OK(t *testing.T) {
	beaconState, block := createFullBellatrixBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	beaconState, err = transition.ProcessSlots(context.Background(), beaconState, wsb.Block().Slot())
	require.NoError(t, err)
	_, err = transition.ProcessOperationsNoVerifyAttsSigs(context.Background(), beaconState, wsb.Block())
	require.NoError(t, err)
}

func TestProcessOperationsNoVerifyAttsSigsCapella_OK(t *testing.T) {
	beaconState, block := createFullCapellaBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	beaconState, err = transition.ProcessSlots(context.Background(), beaconState, wsb.Block().Slot())
	require.NoError(t, err)
	_, err = transition.ProcessOperationsNoVerifyAttsSigs(context.Background(), beaconState, wsb.Block())
	require.NoError(t, err)
}

func TestCalculateStateRootAltair_OK(t *testing.T) {
	beaconState, block := createFullAltairBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)
	r, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	require.NoError(t, err)
	require.DeepNotEqual(t, params.BeaconConfig().ZeroHash, r)
}

func TestProcessBlockDifferentVersion(t *testing.T) {
	beaconState, _ := util.DeterministicGenesisState(t, 64) // Phase 0 state
	_, block := createFullAltairBlockWithOperations(t)
	wsb, err := blocks.NewSignedBeaconBlock(block) // Altair block
	require.NoError(t, err)
	_, _, err = transition.ProcessBlockNoVerifyAnySig(context.Background(), beaconState, wsb)
	require.ErrorContains(t, "state and block are different version. 0 != 1", err)
}

func TestVerifyBlobCommitmentCount(t *testing.T) {
	b := &ethpb.BeaconBlockDeneb{Body: &ethpb.BeaconBlockBodyDeneb{}}
	rb, err := blocks.NewBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, transition.VerifyBlobCommitmentCount(rb))

	b = &ethpb.BeaconBlockDeneb{Body: &ethpb.BeaconBlockBodyDeneb{BlobKzgCommitments: make([][]byte, field_params.MaxBlobsPerBlock+1)}}
	rb, err = blocks.NewBeaconBlock(b)
	require.NoError(t, err)
	require.ErrorContains(t, fmt.Sprintf("too many kzg commitments in block: %d", field_params.MaxBlobsPerBlock+1), transition.VerifyBlobCommitmentCount(rb))
}
