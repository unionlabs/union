package runtime

import (
	"testing"

	"github.com/stretchr/testify/require"

	corestore "cosmossdk.io/core/store"
)

func TestCheckStoreUpgrade(t *testing.T) {
	tests := []struct {
		name          string
		storeUpgrades *corestore.StoreUpgrades
		errMsg        string
	}{
		{
			name:          "Nil StoreUpgrades",
			storeUpgrades: nil,
			errMsg:        "store upgrades cannot be nil",
		},
		{
			name: "Valid StoreUpgrades",
			storeUpgrades: &corestore.StoreUpgrades{
				Added:   []string{"store1", "store2"},
				Deleted: []string{"store3", "store4"},
			},
		},
		{
			name: "Duplicate key in Added",
			storeUpgrades: &corestore.StoreUpgrades{
				Added:   []string{"store1", "store2", "store1"},
				Deleted: []string{"store3"},
			},
			errMsg: "store upgrade has duplicate key store1 in added",
		},
		{
			name: "Duplicate key in Deleted",
			storeUpgrades: &corestore.StoreUpgrades{
				Added:   []string{"store1"},
				Deleted: []string{"store2", "store3", "store2"},
			},
			errMsg: "store upgrade has duplicate key store2 in deleted",
		},
		{
			name: "Key in both Added and Deleted",
			storeUpgrades: &corestore.StoreUpgrades{
				Added:   []string{"store1", "store2"},
				Deleted: []string{"store2", "store3"},
			},
			errMsg: "store upgrade has key store2 in both added and deleted",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := checkStoreUpgrade(tt.storeUpgrades)
			if tt.errMsg == "" {
				require.NoError(t, err)
			} else {
				require.ErrorContains(t, err, tt.errMsg)
			}
		})
	}
}
