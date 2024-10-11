package upgrades

import (
	"context"

	capabilitykeeper "github.com/cosmos/ibc-go/modules/capability/keeper"
	ibckeeper "github.com/cosmos/ibc-go/v8/modules/core/keeper"

	storetypes "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	consensusparamkeeper "cosmossdk.io/x/consensus/keeper"
	paramskeeper "cosmossdk.io/x/params/keeper"
)

type AppKeepers struct {
	AccountKeeper         *authkeeper.AccountKeeper
	ParamsKeeper          *paramskeeper.Keeper
	ConsensusParamsKeeper *consensusparamkeeper.Keeper
	Codec                 codec.Codec
	GetStoreKey           func(storeKey string) *storetypes.KVStoreKey
	CapabilityKeeper      *capabilitykeeper.Keeper
	IBCKeeper             *ibckeeper.Keeper
}
type ModuleManager interface {
	RunMigrations(ctx context.Context, cfg module.Configurator, fromVM module.VersionMap) (module.VersionMap, error)
	GetVersionMap() module.VersionMap
}

// Upgrade defines a struct containing necessary fields that a SoftwareUpgradeProposal
// must have written, in order for the state migration to go smoothly.
// An upgrade must implement this struct, and then set it in the app.go.
// The app.go will then define the handler.
type Upgrade struct {
	// Upgrade version name, for the upgrade handler, e.g. `v7`
	UpgradeName string

	// CreateUpgradeHandler defines the function that creates an upgrade handler
	CreateUpgradeHandler func(ModuleManager, module.Configurator, *AppKeepers) upgradetypes.UpgradeHandler
	StoreUpgrades        storetypes.StoreUpgrades
}
