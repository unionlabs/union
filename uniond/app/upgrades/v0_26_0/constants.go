package v0_26_0

import (
	"cosmossdk.io/core/store"
	"union/app/upgrades"

	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	icacontrollertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icahosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	ibcfeetypes "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
)

const UpgradeName = "v0.26.0"

const TokenFactoryStoreKey = "tokenfactory"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades: store.StoreUpgrades{
		Deleted: []string{
			ibcwasmtypes.StoreKey,
			icacontrollertypes.StoreKey,
			icahosttypes.StoreKey,
			ibcfeetypes.StoreKey,
			TokenFactoryStoreKey,
		},
	},
}
