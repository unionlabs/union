package blockchain

import (
	"context"
	"testing"

	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestReportEpochMetrics_BadHeadState(t *testing.T) {
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	h, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, h.SetValidators(nil))
	err = reportEpochMetrics(context.Background(), s, h)
	require.ErrorContains(t, "failed to initialize precompute: state has nil validator slice", err)
}

func TestReportEpochMetrics_BadAttestation(t *testing.T) {
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	h, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, h.AppendCurrentEpochAttestations(&eth.PendingAttestation{InclusionDelay: 0}))
	err = reportEpochMetrics(context.Background(), s, h)
	require.ErrorContains(t, "attestation with inclusion delay of 0", err)
}

func TestReportEpochMetrics_SlashedValidatorOutOfBound(t *testing.T) {
	h, _ := util.DeterministicGenesisState(t, 1)
	v, err := h.ValidatorAtIndex(0)
	require.NoError(t, err)
	v.Slashed = true
	require.NoError(t, h.UpdateValidatorAtIndex(0, v))
	require.NoError(t, h.AppendCurrentEpochAttestations(&eth.PendingAttestation{InclusionDelay: 1, Data: util.HydrateAttestationData(&eth.AttestationData{})}))
	err = reportEpochMetrics(context.Background(), h, h)
	require.ErrorContains(t, "slot 0 out of bounds", err)
}
