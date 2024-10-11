package validator

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestServer_getSlashings(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisState(t, 64)

	proposerServer := &Server{
		SlashingsPool: slashings.NewPool(),
	}

	proposerSlashings := make([]*ethpb.ProposerSlashing, params.BeaconConfig().MaxProposerSlashings)
	for i := primitives.ValidatorIndex(0); uint64(i) < params.BeaconConfig().MaxProposerSlashings; i++ {
		proposerSlashing, err := util.GenerateProposerSlashingForValidator(beaconState, privKeys[i], i)
		require.NoError(t, err)
		proposerSlashings[i] = proposerSlashing
		err = proposerServer.SlashingsPool.InsertProposerSlashing(context.Background(), beaconState, proposerSlashing)
		require.NoError(t, err)
	}

	attSlashings := make([]*ethpb.AttesterSlashing, params.BeaconConfig().MaxAttesterSlashings)
	for i := uint64(0); i < params.BeaconConfig().MaxAttesterSlashings; i++ {
		attesterSlashing, err := util.GenerateAttesterSlashingForValidator(
			beaconState,
			privKeys[i+params.BeaconConfig().MaxProposerSlashings],
			primitives.ValidatorIndex(i+params.BeaconConfig().MaxProposerSlashings),
		)
		require.NoError(t, err)
		attSlashings[i] = attesterSlashing
		err = proposerServer.SlashingsPool.InsertAttesterSlashing(context.Background(), beaconState, attesterSlashing)
		require.NoError(t, err)
	}

	p, a := proposerServer.getSlashings(context.Background(), beaconState)
	require.Equal(t, len(p), int(params.BeaconConfig().MaxProposerSlashings))
	require.Equal(t, len(a), int(params.BeaconConfig().MaxAttesterSlashings))
	require.DeepEqual(t, p, proposerSlashings)
	require.DeepEqual(t, a, attSlashings)
}
