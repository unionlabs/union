package validator

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestServer_getExits(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	config := params.BeaconConfig()
	config.ShardCommitteePeriod = 0
	params.OverrideBeaconConfig(config)

	beaconState, privKeys := util.DeterministicGenesisState(t, 256)

	proposerServer := &Server{
		ExitPool: voluntaryexits.NewPool(),
	}

	exits := make([]*eth.SignedVoluntaryExit, params.BeaconConfig().MaxVoluntaryExits)
	for i := primitives.ValidatorIndex(0); uint64(i) < params.BeaconConfig().MaxVoluntaryExits; i++ {
		exit, err := util.GenerateVoluntaryExits(beaconState, privKeys[i], i)
		require.NoError(t, err)
		proposerServer.ExitPool.InsertVoluntaryExit(exit)
		exits[i] = exit
	}

	e := proposerServer.getExits(beaconState, 1)
	require.Equal(t, len(e), int(params.BeaconConfig().MaxVoluntaryExits))
	require.DeepEqual(t, e, exits)
}
