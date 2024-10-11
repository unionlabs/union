package simulator

import (
	"bytes"
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGenerateBlockHeadersForSlot_Slashing(t *testing.T) {
	ctx := context.Background()
	simParams := &Parameters{
		AggregationPercent:     1,
		NumValidators:          64,
		ProposerSlashingProbab: 1,
	}
	srv := setupService(t, simParams)

	slot1Blocks, _, err := srv.generateBlockHeadersForSlot(ctx, 1)
	require.NoError(t, err)
	require.Equal(t, 2, len(slot1Blocks))

	block1Root, err := slot1Blocks[0].HashTreeRoot()
	require.NoError(t, err)
	block2Root, err := slot1Blocks[1].HashTreeRoot()
	require.NoError(t, err)
	if slot1Blocks[0].Header.ProposerIndex == slot1Blocks[1].Header.ProposerIndex && bytes.Equal(block1Root[:], block2Root[:]) {
		t.Error("Blocks received were not slashable")
	}
}
