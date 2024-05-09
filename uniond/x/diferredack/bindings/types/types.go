package types

import (
	cosmossdk_io_math "cosmossdk.io/math"
)

type Params struct {
	FeePercentage cosmossdk_io_math.LegacyDec `json:"denom_creation_fee"`
}
