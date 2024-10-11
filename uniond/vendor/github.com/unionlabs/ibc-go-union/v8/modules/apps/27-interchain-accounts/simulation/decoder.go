package simulation

import (
	"bytes"
	"fmt"

	"github.com/cosmos/cosmos-sdk/types/kv"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
)

// NewDecodeStore returns a decoder function closure that unmarshals the KVPair's
// Value to the corresponding DenomTrace type.
func NewDecodeStore() func(kvA, kvB kv.Pair) string {
	return func(kvA, kvB kv.Pair) string {
		switch {
		case bytes.Equal(kvA.Key[:len(types.PortKeyPrefix)], []byte(types.PortKeyPrefix)):
			return fmt.Sprintf("Port A: %s\nPort B: %s", string(kvA.Value), string(kvB.Value))
		case bytes.Equal(kvA.Key[:len(types.OwnerKeyPrefix)], []byte(types.OwnerKeyPrefix)):
			return fmt.Sprintf("Owner A: %s\nOwner B: %s", string(kvA.Value), string(kvB.Value))
		case bytes.Equal(kvA.Key[:len(types.ActiveChannelKeyPrefix)], []byte(types.ActiveChannelKeyPrefix)):
			return fmt.Sprintf("ActiveChannel A: %s\nActiveChannel B: %s", string(kvA.Value), string(kvB.Value))
		case bytes.Equal(kvA.Key[:len(types.IsMiddlewareEnabledPrefix)], []byte(types.IsMiddlewareEnabledPrefix)):
			return fmt.Sprintf("IsMiddlewareEnabled A: %s\nIsMiddlewareEnabled B: %s", string(kvA.Value), string(kvB.Value))

		default:
			panic(fmt.Errorf("invalid %s key prefix %s", types.ModuleName, kvA.Key))
		}
	}
}
