package field_params_test

import (
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func testFieldParametersMatchConfig(t *testing.T) {
	require.Equal(t, uint64(params.BeaconConfig().SlotsPerHistoricalRoot), uint64(fieldparams.BlockRootsLength))
	require.Equal(t, uint64(params.BeaconConfig().SlotsPerHistoricalRoot), uint64(fieldparams.StateRootsLength))
	require.Equal(t, params.BeaconConfig().HistoricalRootsLimit, uint64(fieldparams.HistoricalRootsLength))
	require.Equal(t, uint64(params.BeaconConfig().EpochsPerHistoricalVector), uint64(fieldparams.RandaoMixesLength))
	require.Equal(t, params.BeaconConfig().ValidatorRegistryLimit, uint64(fieldparams.ValidatorRegistryLimit))
	require.Equal(t, uint64(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().EpochsPerEth1VotingPeriod))), uint64(fieldparams.Eth1DataVotesLength))
	require.Equal(t, uint64(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().MaxAttestations)), uint64(fieldparams.PreviousEpochAttestationsLength))
	require.Equal(t, uint64(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().MaxAttestations)), uint64(fieldparams.CurrentEpochAttestationsLength))
	require.Equal(t, uint64(params.BeaconConfig().EpochsPerSlashingsVector), uint64(fieldparams.SlashingsLength))
	require.Equal(t, params.BeaconConfig().SyncCommitteeSize, uint64(fieldparams.SyncCommitteeLength))
}
