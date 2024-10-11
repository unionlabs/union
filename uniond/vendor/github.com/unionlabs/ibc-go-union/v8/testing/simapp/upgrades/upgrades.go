package upgrades

import (
	"context"

	storetypes "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	consensusparamskeeper "cosmossdk.io/x/consensus/keeper"
	paramskeeper "cosmossdk.io/x/params/keeper"
	paramstypes "cosmossdk.io/x/params/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"

	capabilitykeeper "github.com/cosmos/ibc-go/modules/capability/keeper"
	v6 "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/migrations/v6"
	clientkeeper "github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctmmigrations "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint/migrations"
)

const (
	// V5 defines the upgrade name for the ibc-go/v5 upgrade handler.
	V5 = "normal upgrade" // NOTE: keeping as "normal upgrade" as existing tags depend on this name
	// V6 defines the upgrade name for the ibc-go/v6 upgrade handler.
	V6 = "v6"
	// V7 defines the upgrade name for the ibc-go/v7 upgrade handler.
	V7 = "v7"
	// V7_1 defines the upgrade name for the ibc-go/v7.1 upgrade handler.
	V7_1 = "v7.1"
	// V8 defines the upgrade name for the ibc-go/v8 upgrade handler.
	V8 = "v8"
	// V8_1 defines the upgrade name for the ibc-go/v8.1 upgrade handler.
	V8_1 = "v8.1"
)

// CreateDefaultUpgradeHandler creates an upgrade handler which can be used for regular upgrade tests
// that do not require special logic
func CreateDefaultUpgradeHandler(
	mm *module.Manager,
	configurator module.Configurator,
) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, _ upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		return mm.RunMigrations(ctx, configurator, vm)
	}
}

// CreateV6UpgradeHandler creates an upgrade handler for the ibc-go/v6 SimApp upgrade.
// NOTE: The v6.MigrateICS27ChannelCapabiliity function can be omitted if chains do not yet implement an ICS27 controller module
func CreateV6UpgradeHandler(
	mm *module.Manager,
	configurator module.Configurator,
	cdc codec.BinaryCodec,
	capabilityStoreKey *storetypes.KVStoreKey,
	capabilityKeeper *capabilitykeeper.Keeper,
	moduleName string,
) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, _ upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		sdkCtx := sdk.UnwrapSDKContext(ctx)
		if err := v6.MigrateICS27ChannelCapability(sdkCtx, cdc, capabilityStoreKey, capabilityKeeper, moduleName); err != nil {
			return nil, err
		}

		return mm.RunMigrations(ctx, configurator, vm)
	}
}

// CreateV7UpgradeHandler creates an upgrade handler for the ibc-go/v7 SimApp upgrade.
func CreateV7UpgradeHandler(
	mm *module.Manager,
	configurator module.Configurator,
	cdc codec.BinaryCodec,
	clientKeeper clientkeeper.Keeper,
	consensusParamsKeeper consensusparamskeeper.Keeper,
	paramsKeeper paramskeeper.Keeper,
) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, _ upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		sdkCtx := sdk.UnwrapSDKContext(ctx)
		// OPTIONAL: prune expired tendermint consensus states to save storage space
		if _, err := ibctmmigrations.PruneExpiredConsensusStates(sdkCtx, cdc, clientKeeper); err != nil {
			return nil, err
		}

		legacyBaseAppSubspace := paramsKeeper.Subspace(baseapp.Paramspace).WithKeyTable(paramstypes.ConsensusParamsKeyTable())
		err := baseapp.MigrateParams(sdkCtx, legacyBaseAppSubspace, consensusParamsKeeper.ParamsStore)
		if err != nil {
			panic(err)
		}

		return mm.RunMigrations(ctx, configurator, vm)
	}
}

// CreateV7LocalhostUpgradeHandler creates an upgrade handler for the ibc-go/v7.1 SimApp upgrade.
func CreateV7LocalhostUpgradeHandler(
	mm *module.Manager,
	configurator module.Configurator,
	clientKeeper clientkeeper.Keeper,
) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, _ upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		sdkCtx := sdk.UnwrapSDKContext(ctx)
		// explicitly update the IBC 02-client params, adding the localhost client type
		params := clientKeeper.GetParams(sdkCtx)
		params.AllowedClients = append(params.AllowedClients, exported.Localhost)
		clientKeeper.SetParams(sdkCtx, params)

		return mm.RunMigrations(ctx, configurator, vm)
	}
}
