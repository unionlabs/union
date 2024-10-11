package v2

import (
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

var (
	ModuleName = "feegrant"

	// FeeAllowanceKeyPrefix is the set of the kvstore for fee allowance data
	// - 0x00<allowance_key_bytes>: allowance
	FeeAllowanceKeyPrefix = []byte{0x00}

	// FeeAllowanceQueueKeyPrefix is the set of the kvstore for fee allowance keys data
	// - 0x01<allowance_prefix_queue_key_bytes>: <empty value>
	FeeAllowanceQueueKeyPrefix = []byte{0x01}
)

// FeeAllowancePrefixQueue is the canonical key to store grant key.
//
// Key format:
// - <0x01><exp_bytes><len(grantee_address_bytes)><grantee_address_bytes><len(granter_address_bytes)><granter_address_bytes>
func FeeAllowancePrefixQueue(exp *time.Time, granterAddrBz []byte) []byte {
	// no need of appending len(exp_bytes) here, `FormatTimeBytes` gives const length every time.
	var key []byte
	key = append(key, FeeAllowanceQueueKeyPrefix...)
	key = append(key, sdk.FormatTimeBytes(*exp)...)
	return append(key, granterAddrBz...)
}

// FeeAllowanceKey is the canonical key to store a grant from granter to grantee
// We store by grantee first to allow searching by everyone who granted to you
//
// Key format:
// - <0x00><len(grantee_address_bytes)><grantee_address_bytes><len(granter_address_bytes)><granter_address_bytes>
func FeeAllowanceKey(granter, grantee sdk.AccAddress) []byte {
	return append(FeeAllowancePrefixByGrantee(grantee), address.MustLengthPrefix(granter.Bytes())...)
}

// FeeAllowancePrefixByGrantee returns a prefix to scan for all grants to this given address.
//
// Key format:
// - <0x00><len(grantee_address_bytes)><grantee_address_bytes>
func FeeAllowancePrefixByGrantee(grantee sdk.AccAddress) []byte {
	return append(FeeAllowanceKeyPrefix, address.MustLengthPrefix(grantee.Bytes())...)
}
