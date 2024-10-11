package v2

import (
	"time"

	"cosmossdk.io/x/authz/internal/conv"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

// Keys for store prefixes
// Items are stored with the following key: values
//
// - 0x01<grant_Bytes>: Grant
// - 0x02<grant_expiration_Bytes>: GrantQueueItem
var (
	GrantPrefix      = []byte{0x01}
	GrantQueuePrefix = []byte{0x02}
)

// GrantQueueKey - return grant queue store key
// Key format is
//
// - 0x02<grant_expiration_Bytes>: GrantQueueItem
func GrantQueueKey(expiration time.Time, granter, grantee sdk.AccAddress) []byte {
	exp := sdk.FormatTimeBytes(expiration)
	granter = address.MustLengthPrefix(granter)
	grantee = address.MustLengthPrefix(grantee)

	l := 1 + len(exp) + len(granter) + len(grantee)
	key := make([]byte, l)
	copy(key, GrantQueuePrefix)
	copy(key[1:], exp)
	copy(key[1+len(exp):], granter)
	copy(key[1+len(exp)+len(granter):], grantee)
	return key
}

// GrantStoreKey - return authorization store key
// Items are stored with the following key: values
//
// - 0x01<granterAddressLen (1 Byte)><granterAddress_Bytes><granteeAddressLen (1 Byte)><granteeAddress_Bytes><msgType_Bytes>: Grant
func GrantStoreKey(grantee, granter sdk.AccAddress, msgType string) []byte {
	m := conv.UnsafeStrToBytes(msgType)
	granter = address.MustLengthPrefix(granter)
	grantee = address.MustLengthPrefix(grantee)

	l := 1 + len(grantee) + len(granter) + len(m)
	key := make([]byte, l)
	copy(key, GrantPrefix)
	copy(key[1:], granter)
	copy(key[1+len(granter):], grantee)
	copy(key[l-len(m):], m)

	return key
}

// ParseGrantKey - split granter, grantee address and msg type from the authorization key
func ParseGrantKey(key []byte) (granterAddr, granteeAddr sdk.AccAddress, msgType string) {
	// key is of format:
	// <granterAddressLen (1 Byte)><granterAddress_Bytes><granteeAddressLen (1 Byte)><granteeAddress_Bytes><msgType_Bytes>
	kv.AssertKeyAtLeastLength(key, 2)
	granterAddrLen := key[0]
	kv.AssertKeyAtLeastLength(key, int(2+granterAddrLen))
	granterAddr = sdk.AccAddress(key[1 : 1+granterAddrLen])
	granteeAddrLen := int(key[1+granterAddrLen])
	kv.AssertKeyAtLeastLength(key, 3+int(granterAddrLen+byte(granteeAddrLen)))
	granteeAddr = sdk.AccAddress(key[2+granterAddrLen : 2+granterAddrLen+byte(granteeAddrLen)])

	return granterAddr, granteeAddr, conv.UnsafeBytesToStr(key[2+granterAddrLen+byte(granteeAddrLen):])
}
