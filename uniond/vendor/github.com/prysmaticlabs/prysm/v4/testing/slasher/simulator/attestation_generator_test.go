package simulator

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/slashings"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGenerateAttestationsForSlot_Slashing(t *testing.T) {
	ctx := context.Background()
	simParams := &Parameters{
		SecondsPerSlot:         params.BeaconConfig().SecondsPerSlot,
		SlotsPerEpoch:          params.BeaconConfig().SlotsPerEpoch,
		AggregationPercent:     1,
		NumValidators:          64,
		AttesterSlashingProbab: 1,
	}
	srv := setupService(t, simParams)

	epoch3Atts, _, err := srv.generateAttestationsForSlot(ctx, params.BeaconConfig().SlotsPerEpoch*3)
	require.NoError(t, err)
	epoch4Atts, _, err := srv.generateAttestationsForSlot(ctx, params.BeaconConfig().SlotsPerEpoch*4)
	require.NoError(t, err)
	for i := 0; i < len(epoch3Atts); i += 2 {
		goodAtt := epoch3Atts[i]
		surroundAtt := epoch4Atts[i+1]
		require.Equal(t, true, slashings.IsSurround(surroundAtt, goodAtt))
	}
}

func TestGenerateAttestationsForSlot_CorrectIndices(t *testing.T) {
	ctx := context.Background()
	simParams := &Parameters{
		SecondsPerSlot:         params.BeaconConfig().SecondsPerSlot,
		SlotsPerEpoch:          params.BeaconConfig().SlotsPerEpoch,
		AggregationPercent:     1,
		NumValidators:          16384,
		AttesterSlashingProbab: 0,
	}
	srv := setupService(t, simParams)
	slot0Atts, _, err := srv.generateAttestationsForSlot(ctx, 0)
	require.NoError(t, err)
	slot1Atts, _, err := srv.generateAttestationsForSlot(ctx, 1)
	require.NoError(t, err)
	slot2Atts, _, err := srv.generateAttestationsForSlot(ctx, 2)
	require.NoError(t, err)
	var validatorIndices []uint64
	for _, att := range append(slot0Atts, slot1Atts...) {
		validatorIndices = append(validatorIndices, att.AttestingIndices...)
	}
	for _, att := range slot2Atts {
		validatorIndices = append(validatorIndices, att.AttestingIndices...)
	}

	// Making sure indices are one after the other for attestations.
	var validatorIndex uint64
	for _, ii := range validatorIndices {
		require.Equal(t, validatorIndex, ii)
		validatorIndex++
	}
}
