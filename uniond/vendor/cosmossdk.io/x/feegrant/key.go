package feegrant

import (
	"cosmossdk.io/collections"
)

const (
	// ModuleName is the module name constant used in many places
	ModuleName = "feegrant"

	// StoreKey is the store key string for supply
	StoreKey = ModuleName
)

var (
	// FeeAllowanceKeyPrefix is the set of the kvstore for fee allowance data
	// - 0x00<allowance_key_bytes>: allowance
	FeeAllowanceKeyPrefix = collections.NewPrefix(0)

	// FeeAllowanceQueueKeyPrefix is the set of the kvstore for fee allowance keys data
	// - 0x01<allowance_prefix_queue_key_bytes>: <empty value>
	FeeAllowanceQueueKeyPrefix = collections.NewPrefix(1)
)
