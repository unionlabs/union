package keeper_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	"cosmossdk.io/x/epochs/types"
)

func TestEpochsExportGenesis(t *testing.T) {
	ctx, epochsKeeper, _ := Setup(t)

	chainStartTime := ctx.HeaderInfo().Time
	chainStartHeight := ctx.HeaderInfo().Height

	genesis, err := epochsKeeper.ExportGenesis(ctx)
	require.NoError(t, err)
	require.Len(t, genesis.Epochs, 4)

	expectedEpochs := types.DefaultGenesis().Epochs
	for i := 0; i < len(expectedEpochs); i++ {
		expectedEpochs[i].CurrentEpochStartHeight = chainStartHeight
		expectedEpochs[i].StartTime = chainStartTime
	}
	require.Equal(t, expectedEpochs, genesis.Epochs)
}

func TestEpochsInitGenesis(t *testing.T) {
	ctx, epochsKeeper, _ := Setup(t)

	// On init genesis, default epochs information is set
	// To check init genesis again, should make it fresh status
	epochInfos, err := epochsKeeper.AllEpochInfos(ctx)
	require.NoError(t, err)
	for _, epochInfo := range epochInfos {
		err := epochsKeeper.EpochInfo.Remove(ctx, epochInfo.Identifier)
		require.NoError(t, err)
	}

	// now := time.Now()
	ctx.WithHeaderInfo(header.Info{Height: 1, Time: time.Now().UTC()})

	// test genesisState validation
	genesisState := types.GenesisState{
		Epochs: []types.EpochInfo{
			{
				Identifier:              "monthly",
				StartTime:               time.Time{},
				Duration:                time.Hour * 24,
				CurrentEpoch:            0,
				CurrentEpochStartHeight: ctx.BlockHeight(),
				CurrentEpochStartTime:   time.Time{},
				EpochCountingStarted:    true,
			},
			{
				Identifier:              "monthly",
				StartTime:               time.Time{},
				Duration:                time.Hour * 24,
				CurrentEpoch:            0,
				CurrentEpochStartHeight: ctx.BlockHeight(),
				CurrentEpochStartTime:   time.Time{},
				EpochCountingStarted:    true,
			},
		},
	}
	require.EqualError(t, genesisState.Validate(), "epoch identifier should be unique")

	genesisState = types.GenesisState{
		Epochs: []types.EpochInfo{
			{
				Identifier:              "monthly",
				StartTime:               time.Time{},
				Duration:                time.Hour * 24,
				CurrentEpoch:            0,
				CurrentEpochStartHeight: ctx.BlockHeight(),
				CurrentEpochStartTime:   time.Time{},
				EpochCountingStarted:    true,
			},
		},
	}

	err = epochsKeeper.InitGenesis(ctx, genesisState)
	require.NoError(t, err)
	epochInfo, err := epochsKeeper.EpochInfo.Get(ctx, "monthly")
	require.NoError(t, err)
	require.Equal(t, epochInfo.Identifier, "monthly")
	require.Equal(t, epochInfo.StartTime.UTC().String(), ctx.HeaderInfo().Time.UTC().String())
	require.Equal(t, epochInfo.Duration, time.Hour*24)
	require.Equal(t, epochInfo.CurrentEpoch, int64(0))
	require.Equal(t, epochInfo.CurrentEpochStartHeight, ctx.BlockHeight())
	require.Equal(t, epochInfo.CurrentEpochStartTime.UTC().String(), time.Time{}.String())
	require.Equal(t, epochInfo.EpochCountingStarted, true)
}
