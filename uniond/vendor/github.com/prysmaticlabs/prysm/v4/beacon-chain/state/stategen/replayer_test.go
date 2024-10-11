package stategen

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func headerFromBlock(b interfaces.ReadOnlySignedBeaconBlock) (*ethpb.BeaconBlockHeader, error) {
	bodyRoot, err := b.Block().Body().HashTreeRoot()
	if err != nil {
		return nil, err
	}
	stateRoot := b.Block().StateRoot()
	parentRoot := b.Block().ParentRoot()
	return &ethpb.BeaconBlockHeader{
		Slot:          b.Block().Slot(),
		StateRoot:     stateRoot[:],
		ProposerIndex: b.Block().ProposerIndex(),
		BodyRoot:      bodyRoot[:],
		ParentRoot:    parentRoot[:],
	}, nil
}

func TestReplayBlocks_ZeroDiff(t *testing.T) {
	logHook := logTest.NewGlobal()
	ctx := context.Background()
	specs := []mockHistorySpec{{slot: 0}}
	hist := newMockHistory(t, specs, 0)
	ch := NewCanonicalHistory(hist, hist, hist)
	_, err := ch.ReplayerForSlot(0).ReplayBlocks(ctx)
	require.NoError(t, err)
	require.LogsDoNotContain(t, logHook, "Replaying canonical blocks from most recent state")
}

func TestReplayBlocks(t *testing.T) {
	ctx := context.Background()
	var zero, one, two, three, four, five primitives.Slot = 50, 51, 150, 151, 152, 200
	specs := []mockHistorySpec{
		{slot: zero},
		{slot: one, savedState: true},
		{slot: two},
		{slot: three},
		{slot: four},
		{slot: five, canonicalBlock: true},
	}

	hist := newMockHistory(t, specs, five+1)
	ch := NewCanonicalHistory(hist, hist, hist)
	st, err := ch.ReplayerForSlot(five).ReplayBlocks(ctx)
	require.NoError(t, err)
	expected := hist.hiddenStates[hist.slotMap[five]]
	expectedHTR, err := expected.HashTreeRoot(ctx)
	require.NoError(t, err)
	actualHTR, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)
	expectedLBH := expected.LatestBlockHeader()
	actualLBH := st.LatestBlockHeader()
	require.Equal(t, expectedLBH.Slot, actualLBH.Slot)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.ParentRoot), bytesutil.ToBytes32(actualLBH.ParentRoot))
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.StateRoot), bytesutil.ToBytes32(actualLBH.StateRoot))
	require.Equal(t, expectedLBH.ProposerIndex, actualLBH.ProposerIndex)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.BodyRoot), bytesutil.ToBytes32(actualLBH.BodyRoot))
	require.Equal(t, expectedHTR, actualHTR)

	st, err = ch.ReplayerForSlot(one).ReplayBlocks(ctx)
	require.NoError(t, err)
	expected = hist.states[hist.slotMap[one]]

	// no canonical blocks in between, so latest block process_block_header will be for genesis
	expectedLBH, err = headerFromBlock(hist.blocks[hist.slotMap[0]])
	require.NoError(t, err)
	actualLBH = st.LatestBlockHeader()
	require.Equal(t, expectedLBH.Slot, actualLBH.Slot)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.ParentRoot), bytesutil.ToBytes32(actualLBH.ParentRoot))
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.StateRoot), bytesutil.ToBytes32(actualLBH.StateRoot))
	require.Equal(t, expectedLBH.ProposerIndex, actualLBH.ProposerIndex)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.BodyRoot), bytesutil.ToBytes32(actualLBH.BodyRoot))

	require.Equal(t, expected.Slot(), st.Slot())
	// NOTE: HTR is not compared, because process_block is not called for non-canonical blocks,
	// so there are multiple differences compared to the "db" state that applies all blocks
}

func TestReplayToSlot(t *testing.T) {
	ctx := context.Background()
	var zero, one, two, three, four, five primitives.Slot = 50, 51, 150, 151, 152, 200
	specs := []mockHistorySpec{
		{slot: zero},
		{slot: one, savedState: true},
		{slot: two},
		{slot: three},
		{slot: four},
		{slot: five, canonicalBlock: true},
	}

	// first case tests that ReplayToSlot is equivalent to ReplayBlocks
	hist := newMockHistory(t, specs, five+1)
	ch := NewCanonicalHistory(hist, hist, hist)

	st, err := ch.ReplayerForSlot(five).ReplayToSlot(ctx, five)
	require.NoError(t, err)
	expected := hist.hiddenStates[hist.slotMap[five]]
	expectedHTR, err := expected.HashTreeRoot(ctx)
	require.NoError(t, err)
	actualHTR, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)
	expectedLBH := expected.LatestBlockHeader()
	actualLBH := st.LatestBlockHeader()
	require.Equal(t, expectedLBH.Slot, actualLBH.Slot)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.ParentRoot), bytesutil.ToBytes32(actualLBH.ParentRoot))
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.StateRoot), bytesutil.ToBytes32(actualLBH.StateRoot))
	require.Equal(t, expectedLBH.ProposerIndex, actualLBH.ProposerIndex)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.BodyRoot), bytesutil.ToBytes32(actualLBH.BodyRoot))
	require.Equal(t, expectedHTR, actualHTR)

	st, err = ch.ReplayerForSlot(five).ReplayToSlot(ctx, five+100)
	require.NoError(t, err)
	require.Equal(t, five+100, st.Slot())
	expectedLBH, err = headerFromBlock(hist.blocks[hist.slotMap[five]])
	require.NoError(t, err)
	actualLBH = st.LatestBlockHeader()
	require.Equal(t, expectedLBH.Slot, actualLBH.Slot)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.ParentRoot), bytesutil.ToBytes32(actualLBH.ParentRoot))
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.StateRoot), bytesutil.ToBytes32(actualLBH.StateRoot))
	require.Equal(t, expectedLBH.ProposerIndex, actualLBH.ProposerIndex)
	require.Equal(t, bytesutil.ToBytes32(expectedLBH.BodyRoot), bytesutil.ToBytes32(actualLBH.BodyRoot))
}
