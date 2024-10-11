package v4

import (
	"encoding/binary"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

const MissedBlockBitmapChunkSize = 1024 // 2^10 bits

var (
	ValidatorSigningInfoKeyPrefix         = []byte{0x01}
	validatorMissedBlockBitArrayKeyPrefix = []byte{0x02}
)

func ValidatorSigningInfoKey(v sdk.ConsAddress) []byte {
	return append(ValidatorSigningInfoKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

func ValidatorSigningInfoAddress(key []byte) (v sdk.ConsAddress) {
	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]

	return sdk.ConsAddress(addr)
}

func validatorMissedBlockBitArrayPrefixKey(v sdk.ConsAddress) []byte {
	return append(validatorMissedBlockBitArrayKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

func ValidatorMissedBlockBitArrayKey(v sdk.ConsAddress, i int64) []byte {
	b := make([]byte, 8)
	binary.LittleEndian.PutUint64(b, uint64(i))
	return append(validatorMissedBlockBitArrayPrefixKey(v), b...)
}

func validatorMissedBlockBitmapPrefixKey(v sdk.ConsAddress) []byte {
	return append(validatorMissedBlockBitArrayKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

func ValidatorMissedBlockBitmapKey(v sdk.ConsAddress, chunkIndex int64) []byte {
	bz := make([]byte, 8)
	binary.LittleEndian.PutUint64(bz, uint64(chunkIndex))

	return append(validatorMissedBlockBitmapPrefixKey(v), bz...)
}
