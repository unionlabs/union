package v1_1_0

import (
	store "cosmossdk.io/store/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const UpgradeName = "v1.1.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades:        store.StoreUpgrades{},
}
