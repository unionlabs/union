package types

import "cosmossdk.io/collections"

const (
	// module name
	ModuleName = "crisis"

	StoreKey = ModuleName
)

var ConstantFeeKey = collections.NewPrefix(1)
