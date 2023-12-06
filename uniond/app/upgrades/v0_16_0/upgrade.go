package v0_16_0

import (
	"context"
	"union/app/upgrades"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	"github.com/cosmos/cosmos-sdk/types/module"
)

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		consensusParams, err := keepers.ConsensusKeeper.Params(ctx, nil)
		if err != nil {
			panic(err)
		}

		PubKeyTypes := [...]string{"bn254"}

		consensusParams.Params.Block.MaxBytes = 22020096
		consensusParams.Params.Block.MaxGas = -1
		consensusParams.Params.Evidence.MaxAgeNumBlocks = 100000
		consensusParams.Params.Evidence.MaxAgeDuration = 172800000000000
		consensusParams.Params.Evidence.MaxBytes = 1048576
		consensusParams.Params.Validator.PubKeyTypes = PubKeyTypes[:]
		consensusParams.Params.Version.App = 0
		consensusParams.Params.Abci.VoteExtensionsEnableHeight = 0 // TODO decide on when to enable vote extensions
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
