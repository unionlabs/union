package legacy

import (
	"fmt"

	"cosmossdk.io/core/registry"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// RegisterAminoMsg first checks that the msgName is <40 chars
// (else this would break ledger nano signing: https://github.com/cosmos/cosmos-sdk/issues/10870),
// then registers the concrete msg type with amino.
func RegisterAminoMsg(registrar registry.AminoRegistrar, msg sdk.Msg, msgName string) {
	if len(msgName) > 39 {
		panic(fmt.Errorf("msg name %s is too long to be registered with amino", msgName))
	}
	registrar.RegisterConcrete(msg, msgName)
}
