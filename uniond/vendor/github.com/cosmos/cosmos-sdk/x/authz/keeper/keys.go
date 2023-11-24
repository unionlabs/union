package keeper

import (
	"time"

	"github.com/cosmos/cosmos-sdk/internal/conv"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/authz"
)

// Keys for store prefixes
// Items are stored with the following key: values
//
// - 0x01<grant_Bytes>: Grant
// - 0x02<grant_expiration_Bytes>: GrantQueueItem
var (
	GrantKey         = []byte{0x01} // prefix for each key
	GrantQueuePrefix = []byte{0x02}
)

var lenTime = len(sdk.FormatTimeBytes(time.Now()))

// StoreKey is the store key string for authz
const StoreKey = authz.ModuleName

// grantStoreKey - return authorization store key
// Items are stored with the following key: values
//
// - 0x01<granterAddressLen (1 Byte)><granterAddress_Bytes><granteeAddressLen (1 Byte)><granteeAddress_Bytes><msgType_Bytes>: Grant
func grantStoreKey(grantee, granter sdk.AccAddress, msgType string) []byte {
	m := conv.UnsafeStrToBytes(msgType)
	granter = address.MustLengthPrefix(granter)
	grantee = address.MustLengthPrefix(grantee)
	key := sdk.AppendLengthPrefixedBytes(GrantKey, granter, grantee, m)

	return key
}

// parseGrantStoreKey - split granter, grantee address and msg type from the authorization key
func parseGrantStoreKey(key []byte) (granterAddr, granteeAddr sdk.AccAddress, msgType string) {
	// key is of format:
	// 0x01<granterAddressLen (1 Byte)><granterAddress_Bytes><granteeAddressLen (1 Byte)><granteeAddress_Bytes><msgType_Bytes>

	granterAddrLen, granterAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, 1, 1) // ignore key[0] since it is a prefix key
	granterAddr, granterAddrEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrLenEndIndex+1, int(granterAddrLen[0]))

	granteeAddrLen, granteeAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrEndIndex+1, 1)
	granteeAddr, granteeAddrEndIndex := sdk.ParseLengthPrefixedBytes(key, granteeAddrLenEndIndex+1, int(granteeAddrLen[0]))

	kv.AssertKeyAtLeastLength(key, granteeAddrEndIndex+1)
	return granterAddr, granteeAddr, conv.UnsafeBytesToStr(key[(granteeAddrEndIndex + 1):])
}

// parseGrantQueueKey split expiration time, granter and grantee from the grant queue key
func parseGrantQueueKey(key []byte) (time.Time, sdk.AccAddress, sdk.AccAddress, error) {
	// key is of format:
	// 0x02<grant_expiration_Bytes><granterAddress_Bytes><granteeAddressLen (1 Byte)><granteeAddress_Bytes>

	expBytes, expEndIndex := sdk.ParseLengthPrefixedBytes(key, 1, lenTime)

	exp, err := sdk.ParseTimeBytes(expBytes)
	if err != nil {
		return exp, nil, nil, err
	}

	granterAddrLen, granterAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, expEndIndex+1, 1)
	granter, granterEndIndex := sdk.ParseLengthPrefixedBytes(key, granterAddrLenEndIndex+1, int(granterAddrLen[0]))

	granteeAddrLen, granteeAddrLenEndIndex := sdk.ParseLengthPrefixedBytes(key, granterEndIndex+1, 1)
	grantee, _ := sdk.ParseLengthPrefixedBytes(key, granteeAddrLenEndIndex+1, int(granteeAddrLen[0]))

	return exp, granter, grantee, nil
}

// GrantQueueKey - return grant queue store key. If a given grant doesn't have a defined
// expiration, then it should not be used in the pruning queue.
// Key format is:
//
//	0x02<expiration><granterAddressLen (1 Byte)><granterAddressBytes><granteeAddressLen (1 Byte)><granteeAddressBytes>: GrantQueueItem
func GrantQueueKey(expiration time.Time, granter, grantee sdk.AccAddress) []byte {
	exp := sdk.FormatTimeBytes(expiration)
	granter = address.MustLengthPrefix(granter)
	grantee = address.MustLengthPrefix(grantee)

	return sdk.AppendLengthPrefixedBytes(GrantQueuePrefix, exp, granter, grantee)
}

// GrantQueueTimePrefix - return grant queue time prefix
func GrantQueueTimePrefix(expiration time.Time) []byte {
	return append(GrantQueuePrefix, sdk.FormatTimeBytes(expiration)...)
}

// firstAddressFromGrantStoreKey parses the first address only
func firstAddressFromGrantStoreKey(key []byte) sdk.AccAddress {
	addrLen := key[0]
	return sdk.AccAddress(key[1 : 1+addrLen])
}
