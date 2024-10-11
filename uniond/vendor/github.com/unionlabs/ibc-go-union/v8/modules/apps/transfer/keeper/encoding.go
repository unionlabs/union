package keeper

import (
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// UnmarshalDenomTrace attempts to decode and return an DenomTrace object from
// raw encoded bytes.
func (k Keeper) UnmarshalDenomTrace(bz []byte) (types.DenomTrace, error) {
	var denomTrace types.DenomTrace
	if err := k.cdc.Unmarshal(bz, &denomTrace); err != nil {
		return types.DenomTrace{}, err
	}

	return denomTrace, nil
}

// MustUnmarshalDenomTrace attempts to decode and return an DenomTrace object from
// raw encoded bytes. It panics on error.
func (k Keeper) MustUnmarshalDenomTrace(bz []byte) types.DenomTrace {
	var denomTrace types.DenomTrace
	k.cdc.MustUnmarshal(bz, &denomTrace)
	return denomTrace
}

// MarshalDenomTrace attempts to encode an DenomTrace object and returns the
// raw encoded bytes.
func (k Keeper) MarshalDenomTrace(denomTrace types.DenomTrace) ([]byte, error) {
	return k.cdc.Marshal(&denomTrace)
}

// MustMarshalDenomTrace attempts to encode an DenomTrace object and returns the
// raw encoded bytes. It panics on error.
func (k Keeper) MustMarshalDenomTrace(denomTrace types.DenomTrace) []byte {
	return k.cdc.MustMarshal(&denomTrace)
}
