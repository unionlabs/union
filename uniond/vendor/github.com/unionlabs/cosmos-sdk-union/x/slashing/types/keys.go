package types

import (
	"cosmossdk.io/collections"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

const (
	// ModuleName is the name of the module
	ModuleName = "slashing"

	// StoreKey is the store key string for slashing
	StoreKey = ModuleName

	// RouterKey is the message route for slashing
	RouterKey = ModuleName

	// MissedBlockBitmapChunkSize defines the chunk size, in number of bits, of a
	// validator missed block bitmap. Chunks are used to reduce the storage and
	// write overhead of IAVL nodes. The total size of the bitmap is roughly in
	// the range [0, SignedBlocksWindow) where each bit represents a block. A
	// validator's IndexOffset modulo the SignedBlocksWindow is used to retrieve
	// the chunk in that bitmap range. Once the chunk is retrieved, the same index
	// is used to check or flip a bit, where if a bit is set, it indicates the
	// validator missed that block.
	//
	// For a bitmap of N items, i.e. a validator's signed block window, the amount
	// of write complexity per write with a factor of f being the overhead of
	// IAVL being un-optimized, i.e. 2-4, is as follows:
	//
	// ChunkSize + (f * 256 <IAVL leaf hash>) + 256 * log_2(N / ChunkSize)
	//
	// As for the storage overhead, with the same factor f, it is as follows:
	// (N - 256) + (N / ChunkSize) * (512 * f)
	MissedBlockBitmapChunkSize = 1024 // 2^10 bits

	// GovModuleName duplicates the gov module's name to avoid a cyclic dependency with x/gov.
	// It should be synced with the gov module's name if it is ever changed.
	// See: https://github.com/cosmos/cosmos-sdk/blob/b62a28aac041829da5ded4aeacfcd7a42873d1c8/x/gov/types/keys.go#L9
	GovModuleName = "gov"
)

// Keys for slashing store
// Items are stored with the following key: values
//
// - 0x01<consAddrLen (1 Byte)><consAddress_Bytes>: ValidatorSigningInfo
//
// - 0x02<consAddrLen (1 Byte)><consAddress_Bytes><chunk_index>: bitmap_chunk
//
// - 0x03<accAddrLen (1 Byte)><accAddr_Bytes>: cryptotypes.PubKey

var (
	ParamsKey                           = collections.NewPrefix(0) // Prefix for params key
	ValidatorSigningInfoKeyPrefix       = collections.NewPrefix(1) // Prefix for signing info
	ValidatorMissedBlockBitmapKeyPrefix = collections.NewPrefix(2) // Prefix for missed block bitmap
	AddrPubkeyRelationKeyPrefix         = collections.NewPrefix(3) // Prefix for address-pubkey relation
)

// ValidatorSigningInfoKey - stored by *Consensus* address (not operator address)
func ValidatorSigningInfoKey(v sdk.ConsAddress) []byte {
	return append(ValidatorSigningInfoKeyPrefix, address.MustLengthPrefix(v.Bytes())...)
}
