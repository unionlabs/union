package v2

import (
	"encoding/binary"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

var ValidatorMissedBlockBitArrayKeyPrefix = []byte{0x02}

func ValidatorMissedBlockBitArrayPrefixKey(v sdk.ConsAddress) []byte {
	return append(ValidatorMissedBlockBitArrayKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}

func ValidatorMissedBlockBitArrayKey(v sdk.ConsAddress, i int64) []byte {
	b := make([]byte, 8)
	binary.LittleEndian.PutUint64(b, uint64(i))

	return append(ValidatorMissedBlockBitArrayPrefixKey(v), b...)
}
