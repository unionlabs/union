package v0_9_0

import (
	"union/app/upgrades"

	store "github.com/cosmos/cosmos-sdk/store/types"
)

const UpgradeName = "v0.9.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{Added: []string{}},
}
