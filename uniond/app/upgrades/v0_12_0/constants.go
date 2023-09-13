package v0_12_0

import (
	store "github.com/cosmos/cosmos-sdk/store/types"
	"union/app/upgrades"
)

const UpgradeName = "v0.12.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{Added: []string{}},
}
