package feegrant

import (
	time "time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

const (
	// ModuleName is the module name constant used in many places
	ModuleName = "feegrant"

	// StoreKey is the store key string for supply
	StoreKey = ModuleName

	// RouterKey is the message route for supply
	RouterKey = ModuleName

	// QuerierRoute is the querier route for supply
	QuerierRoute = ModuleName
)

var (
	// FeeAllowanceKeyPrefix is the set of the kvstore for fee allowance data
	// - 0x00<allowance_key_bytes>: allowance
	FeeAllowanceKeyPrefix = []byte{0x00}

	// FeeAllowanceQueueKeyPrefix is the set of the kvstore for fee allowance keys data
	// - 0x01<allowance_prefix_queue_key_bytes>: <empty value>
	FeeAllowanceQueueKeyPrefix = []byte{0x01}
)

// FeeAllowanceKey is the canonical key to store a grant from granter to grantee
// We store by grantee first to allow searching by everyone who granted to you
//
// Key format:
// - <0x00><len(grantee_address_bytes)><grantee_address_bytes><len(granter_address_bytes)><granter_address_bytes>
func FeeAllowanceKey(granter sdk.AccAddress, grantee sdk.AccAddress) []byte {
	return append(FeeAllowancePrefixByGrantee(grantee), address.MustLengthPrefix(granter.Bytes())...)
}

// FeeAllowancePrefixByGrantee returns a prefix to scan for all grants to this given address.
//
// Key format:
// - <0x00><len(grantee_address_bytes)><grantee_address_bytes>
func FeeAllowancePrefixByGrantee(grantee sdk.AccAddress) []byte {
	return append(FeeAllowanceKeyPrefix, address.MustLengthPrefix(grantee.Bytes())...)
}

// FeeAllowancePrefixQueue is the canonical key to store grant key.
//
// Key format:
// - <0x01><exp_bytes><len(grantee_address_bytes)><grantee_address_bytes><len(granter_address_bytes)><granter_address_bytes>
func FeeAllowancePrefixQueue(exp *time.Time, key []byte) []byte {
	allowanceByExpTimeKey := AllowanceByExpTimeKey(exp)
	return append(allowanceByExpTimeKey, key...)
}

// AllowanceByExpTimeKey returns a key with `FeeAllowanceQueueKeyPrefix`, expiry
//
// Key format:
// - <0x01><exp_bytes>
func AllowanceByExpTimeKey(exp *time.Time) []byte {
	// no need of appending len(exp_bytes) here, `FormatTimeBytes` gives const length everytime.
	return append(FeeAllowanceQueueKeyPrefix, sdk.FormatTimeBytes(*exp)...)
}

// ParseAddressesFromFeeAllowanceKey extracts and returns the granter, grantee from the given key.
func ParseAddressesFromFeeAllowanceKey(key []byte) (granter, grantee sdk.AccAddress) {
	// key is of format:
	// 0x00<granteeAddressLen (1 Byte)><granteeAddress_Bytes><granterAddressLen (1 Byte)><granterAddress_Bytes>
	granterAddrLen, granterAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, 1, 1) // ignore key[0] since it is a prefix key
	grantee, granterAddrEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrLenEndIndex+1, int(granterAddrLen[0]))

	granteeAddrLen, granteeAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrEndIndex+1, 1)
	granter, _ = sdk.ParseLengthPrefixedBytes(key, granteeAddrLenEndIndex+1, int(granteeAddrLen[0]))

	return granter, grantee
}

// ParseAddressesFromFeeAllowanceQueueKey extracts and returns the granter, grantee from the given key.
func ParseAddressesFromFeeAllowanceQueueKey(key []byte) (granter, grantee sdk.AccAddress) {
	lenTime := len(sdk.FormatTimeBytes(time.Now()))

	// key is of format:
	// <0x01><expiration_bytes(fixed length)><granteeAddressLen (1 Byte)><granteeAddress_Bytes><granterAddressLen (1 Byte)><granterAddress_Bytes>
	granterAddrLen, granterAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, 1+lenTime, 1) // ignore key[0] since it is a prefix key
	grantee, granterAddrEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrLenEndIndex+1, int(granterAddrLen[0]))

	granteeAddrLen, granteeAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrEndIndex+1, 1)
	granter, _ = sdk.ParseLengthPrefixedBytes(key, granteeAddrLenEndIndex+1, int(granteeAddrLen[0]))

	return granter, grantee
}
