package v0_11_0

import (
	"union/app/upgrades"
	tftypes "union/x/tokenfactory/types"

	store "github.com/cosmos/cosmos-sdk/store/types"
)

const UpgradeName = "v0.11.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{Added: []string{tftypes.ModuleName}},
}
