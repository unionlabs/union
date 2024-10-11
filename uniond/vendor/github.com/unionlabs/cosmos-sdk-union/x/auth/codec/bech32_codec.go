package codec

import (
	"cosmossdk.io/core/address"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
)

func NewBech32Codec(prefix string) address.Codec {
	// Host custom bech32 address codec here, if auth ever do not depend on the Cosmos SDK.
	return addresscodec.NewBech32Codec(prefix)
}
