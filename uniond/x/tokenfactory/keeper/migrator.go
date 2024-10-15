package keeper

import (
	"fmt"

	banktypes "cosmossdk.io/x/bank/types"

	"union/x/tokenfactory/exported"
)

// Migrator is a struct for handling in-place state migrations.
type Migrator struct {
	keeper         Keeper
	legacySubspace exported.Subspace
}

func NewMigrator(k Keeper, ss exported.Subspace) Migrator {
	return Migrator{
		keeper:         k,
		legacySubspace: ss,
	}
}

func (m Migrator) SetMetadata(denomMetadata *banktypes.Metadata) {
	if len(denomMetadata.Base) == 0 {
		panic(fmt.Errorf("no base exists for denom %v", denomMetadata))
	}
	if len(denomMetadata.Display) == 0 {
		denomMetadata.Display = denomMetadata.Base
		denomMetadata.Name = denomMetadata.Base
		denomMetadata.Symbol = denomMetadata.Base
	} else {
		fmt.Printf("Denom %s already has denom set", denomMetadata.Base)
	}
}
