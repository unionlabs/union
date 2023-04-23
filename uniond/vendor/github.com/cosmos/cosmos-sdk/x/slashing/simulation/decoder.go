package simulation

import (
	"bytes"
	"fmt"

	gogotypes "github.com/cosmos/gogoproto/types"

	"github.com/cosmos/cosmos-sdk/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// NewDecodeStore returns a decoder function closure that unmarshals the KVPair's
// Value to the corresponding slashing type.
func NewDecodeStore(cdc codec.BinaryCodec) func(kvA, kvB kv.Pair) string {
	return func(kvA, kvB kv.Pair) string {
		switch {
		case bytes.Equal(kvA.Key[:1], types.ValidatorSigningInfoKeyPrefix):
			var infoA, infoB types.ValidatorSigningInfo
			cdc.MustUnmarshal(kvA.Value, &infoA)
			cdc.MustUnmarshal(kvB.Value, &infoB)
			return fmt.Sprintf("%v\n%v", infoA, infoB)

		case bytes.Equal(kvA.Key[:1], types.ValidatorMissedBlockBitArrayKeyPrefix):
			var missedA, missedB gogotypes.BoolValue
			cdc.MustUnmarshal(kvA.Value, &missedA)
			cdc.MustUnmarshal(kvB.Value, &missedB)
			return fmt.Sprintf("missedA: %v\nmissedB: %v", missedA.Value, missedB.Value)

		case bytes.Equal(kvA.Key[:1], types.AddrPubkeyRelationKeyPrefix):
			var pubKeyA, pubKeyB cryptotypes.PubKey
			if err := cdc.UnmarshalInterface(kvA.Value, &pubKeyA); err != nil {
				panic(fmt.Sprint("Can't unmarshal kvA; ", err))
			}
			if err := cdc.UnmarshalInterface(kvB.Value, &pubKeyB); err != nil {
				panic(fmt.Sprint("Can't unmarshal kvB; ", err))
			}
			return fmt.Sprintf("PubKeyA: %s\nPubKeyB: %s", pubKeyA, pubKeyB)

		default:
			panic(fmt.Sprintf("invalid slashing key prefix %X", kvA.Key[:1]))
		}
	}
}
