package v0_22_0

import (
	store "cosmossdk.io/store/types"
	"union/app/upgrades"
)

const UpgradeName = "v0.22.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{},
}
