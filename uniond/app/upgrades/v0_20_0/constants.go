package v0_20_0

import (
	store "cosmossdk.io/store/types"
	"union/app/upgrades"
)

const UpgradeName = "v0.20.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{},
}
