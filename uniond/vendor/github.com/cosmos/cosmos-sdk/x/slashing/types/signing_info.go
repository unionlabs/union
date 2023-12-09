package types

import (
	"time"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewValidatorSigningInfo creates a new ValidatorSigningInfo instance
func NewValidatorSigningInfo(
	consAddr sdk.ConsAddress, startHeight, indexOffset int64,
	jailedUntil time.Time, tombstoned bool, missedBlocksCounter int64,
) ValidatorSigningInfo {
	return ValidatorSigningInfo{
		Address:             consAddr.String(),
		StartHeight:         startHeight,
		IndexOffset:         indexOffset,
		JailedUntil:         jailedUntil,
		Tombstoned:          tombstoned,
		MissedBlocksCounter: missedBlocksCounter,
	}
}

// UnmarshalValSigningInfo unmarshals a validator signing info from a store value
func UnmarshalValSigningInfo(cdc codec.Codec, value []byte) (signingInfo ValidatorSigningInfo, err error) {
	err = cdc.Unmarshal(value, &signingInfo)
	return signingInfo, err
}
