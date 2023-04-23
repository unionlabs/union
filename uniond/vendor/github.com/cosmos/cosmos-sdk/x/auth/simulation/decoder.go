package simulation

import (
	"bytes"
	"fmt"

	gogotypes "github.com/cosmos/gogoproto/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

type AuthUnmarshaler interface {
	UnmarshalAccount([]byte) (types.AccountI, error)
	GetCodec() codec.BinaryCodec
}

// NewDecodeStore returns a decoder function closure that unmarshals the KVPair's
// Value to the corresponding auth type.
func NewDecodeStore(ak AuthUnmarshaler) func(kvA, kvB kv.Pair) string {
	return func(kvA, kvB kv.Pair) string {
		switch {
		case bytes.Equal(kvA.Key[:1], types.AddressStoreKeyPrefix):
			accA, err := ak.UnmarshalAccount(kvA.Value)
			if err != nil {
				panic(err)
			}

			accB, err := ak.UnmarshalAccount(kvB.Value)
			if err != nil {
				panic(err)
			}

			return fmt.Sprintf("%v\n%v", accA, accB)

		case bytes.Equal(kvA.Key, types.GlobalAccountNumberKey):
			var globalAccNumberA, globalAccNumberB gogotypes.UInt64Value
			ak.GetCodec().MustUnmarshal(kvA.Value, &globalAccNumberA)
			ak.GetCodec().MustUnmarshal(kvB.Value, &globalAccNumberB)

			return fmt.Sprintf("GlobalAccNumberA: %d\nGlobalAccNumberB: %d", globalAccNumberA, globalAccNumberB)

		case bytes.HasPrefix(kvA.Key, types.AccountNumberStoreKeyPrefix):
			var accNumA, accNumB sdk.AccAddress
			err := accNumA.Unmarshal(kvA.Value)
			if err != nil {
				panic(err)
			}

			err = accNumB.Unmarshal(kvB.Value)
			if err != nil {
				panic(err)
			}

			return fmt.Sprintf("AccNumA: %s\nAccNumB: %s", accNumA, accNumB)

		default:
			panic(fmt.Sprintf("unexpected %s key %X (%s)", types.ModuleName, kvA.Key, kvA.Key))
		}
	}
}
