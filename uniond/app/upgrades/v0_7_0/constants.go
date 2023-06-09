package v0_7_0

import (
	"union/app/upgrades"

	ibcfeetypes "github.com/cosmos/ibc-go/v7/modules/apps/29-fee/types"
	"github.com/CosmWasm/wasmd/x/wasm"
	store "github.com/cosmos/cosmos-sdk/store/types"
)

const UpgradeName = "v0.7.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{Added: []string{ibcfeetypes.StoreKey, wasm.StoreKey}},
}
