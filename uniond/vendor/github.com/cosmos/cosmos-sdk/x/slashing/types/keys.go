package types

import (
	"encoding/binary"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

const (
	// ModuleName is the name of the module
	ModuleName = "slashing"

	// StoreKey is the store key string for slashing
	StoreKey = ModuleName

	// RouterKey is the message route for slashing
	RouterKey = ModuleName
)

// Keys for slashing store
// Items are stored with the following key: values
//
// - 0x01<consAddrLen (1 Byte)><consAddress_Bytes>: ValidatorSigningInfo
//
// - 0x02<consAddrLen (1 Byte)><consAddress_Bytes><period_Bytes>: bool
//
// - 0x03<accAddrLen (1 Byte)><accAddr_Bytes>: cryptotypes.PubKey

var (
	ParamsKey                             = []byte{0x00} // Prefix for params key
	ValidatorSigningInfoKeyPrefix         = []byte{0x01} // Prefix for signing info
	ValidatorMissedBlockBitArrayKeyPrefix = []byte{0x02} // Prefix for missed block bit array
	AddrPubkeyRelationKeyPrefix           = []byte{0x03} // Prefix for address-pubkey relation
)

// ValidatorSigningInfoKey - stored by *Consensus* address (not operator address)
func ValidatorSigningInfoKey(v sdk.ConsAddress) []byte {
	return append(ValidatorSigningInfoKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// ValidatorSigningInfoAddress - extract the address from a validator signing info key
func ValidatorSigningInfoAddress(key []byte) (v sdk.ConsAddress) {
	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]

	return sdk.ConsAddress(addr)
}

// ValidatorMissedBlockBitArrayPrefixKey - stored by *Consensus* address (not operator address)
func ValidatorMissedBlockBitArrayPrefixKey(v sdk.ConsAddress) []byte {
	return append(ValidatorMissedBlockBitArrayKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// ValidatorMissedBlockBitArrayKey - stored by *Consensus* address (not operator address)
func ValidatorMissedBlockBitArrayKey(v sdk.ConsAddress, i int64) []byte {
	b := make([]byte, 8)
	binary.LittleEndian.PutUint64(b, uint64(i))

	return append(ValidatorMissedBlockBitArrayPrefixKey(v), b...)
}

// AddrPubkeyRelationKey gets pubkey relation key used to get the pubkey from the address
func AddrPubkeyRelationKey(addr []byte) []byte {
	return append(AddrPubkeyRelationKeyPrefix, address.MustLengthPrefix(addr)...)
}
