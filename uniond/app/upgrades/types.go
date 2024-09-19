package upgrades

import (
	tfkeeper "union/x/tokenfactory/keeper"

	store "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	consensuskeeper "cosmossdk.io/x/consensus/keeper"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
)

type AppKeepers struct {
	ConsensusKeeper *consensuskeeper.Keeper
	StakingKeeper   *stakingkeeper.Keeper
	TfKeeper        *tfkeeper.Keeper
}

// source: https://github.com/osmosis-labs/osmosis/blob/c783ef52af8617d3ec613d9ce9035386ba8d4a49/app/upgrades/types.go#L24

// Upgrade defines a struct containing necessary fields that a SoftwareUpgradeProposal
// must have written, in order for the state migration to go smoothly.
// An upgrade must implement this struct, and then set it in the app.go.
// The app.go will then define the handler.
type Upgrade struct {
	// Upgrade version name, for the upgrade handler, e.g. `v7`
	UpgradeName string

	// CreateUpgradeHandler defines the function that creates an upgrade handler
	CreateUpgradeHandler func(*module.Manager, module.Configurator, *AppKeepers) upgradetypes.UpgradeHandler

	// Store upgrades, should be used for any new modules introduced, new modules deleted, or store names renamed.
	StoreUpgrades store.StoreUpgrades
}
