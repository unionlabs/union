package keeper

/*
	This file is to allow for unexported functions and fields to be accessible to the testing package.
*/

import porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"

// GetICS4Wrapper is a getter for the keeper's ICS4Wrapper.
func (k *Keeper) GetICS4Wrapper() porttypes.ICS4Wrapper {
	return k.ics4Wrapper
}
