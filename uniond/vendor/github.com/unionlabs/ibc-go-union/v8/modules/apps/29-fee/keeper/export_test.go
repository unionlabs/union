package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
)

// GetICS4Wrapper is a getter for the keeper's ICS4Wrapper.
func (k *Keeper) GetICS4Wrapper() porttypes.ICS4Wrapper {
	return k.ics4Wrapper
}

// LegacyTotal is a wrapper for the legacyTotal function for testing.
func LegacyTotal(f types.Fee) sdk.Coins {
	return legacyTotal(f)
}
