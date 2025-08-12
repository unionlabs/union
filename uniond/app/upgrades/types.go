package upgrades

import (
	store "cosmossdk.io/store/types"
	storetypes "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/types/module"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	consensuskeeper "github.com/cosmos/cosmos-sdk/x/consensus/keeper"
	crisiskeeper "github.com/cosmos/cosmos-sdk/x/crisis/keeper"
	distributionkeeper "github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	govkeeper "github.com/cosmos/cosmos-sdk/x/gov/keeper"
	mintkeeper "github.com/cosmos/cosmos-sdk/x/mint/keeper"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"

	feemarketkeeper "github.com/skip-mev/feemarket/x/feemarket/keeper"
)

type AppKeepers struct {
	AuthKeeper         authkeeper.AccountKeeper
	BankKeeper         bankkeeper.Keeper
	ConsensusKeeper    *consensuskeeper.Keeper
	CrisisKeeper       crisiskeeper.Keeper
	DistributionKeeper distributionkeeper.Keeper
	FeeMarketKeeper    feemarketkeeper.Keeper
	GovKeeper          govkeeper.Keeper
	MintKeeper         mintkeeper.Keeper
	StakingKeeper      *stakingkeeper.Keeper
}

type GetKeyFunc func(string) *storetypes.KVStoreKey

// source: https://github.com/osmosis-labs/osmosis/blob/c783ef52af8617d3ec613d9ce9035386ba8d4a49/app/upgrades/types.go#L24

// Upgrade defines a struct containing necessary fields that a SoftwareUpgradeProposal
// must have written, in order for the state migration to go smoothly.
// An upgrade must implement this struct, and then set it in the app.go.
// The app.go will then define the handler.
type Upgrade struct {
	// Upgrade version name, for the upgrade handler, e.g. `v7`
	UpgradeName string

	// CreateUpgradeHandler defines the function that creates an upgrade handler
	CreateUpgradeHandler func(*module.Manager, module.Configurator, *AppKeepers, GetKeyFunc) upgradetypes.UpgradeHandler

	// Store upgrades, should be used for any new modules introduced, new modules deleted, or store names renamed.
	StoreUpgrades store.StoreUpgrades
}
