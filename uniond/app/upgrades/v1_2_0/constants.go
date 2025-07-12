package v1_2_0

import (
	store "cosmossdk.io/store/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const UpgradeName = "v1.2.0"

var Upgrade = upgrades.Upgrade{
	UpgradeName:          UpgradeName,
	CreateUpgradeHandler: CreateUpgradeHandler,
	StoreUpgrades: store.StoreUpgrades{
		Added:   []string{},
		Renamed: []store.StoreRename{},
		Deleted: []string{
			"poa",
		},
	},
}
