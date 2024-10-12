package localhost

import (
	"github.com/cosmos/ibc-go/v8/modules/core/exported"

	"cosmossdk.io/core/registry"
)

// RegisterInterfaces registers the tendermint concrete client-related
// implementations and interfaces.
func RegisterInterfaces(registry registry.InterfaceRegistrar) {
	registry.RegisterImplementations(
		(*exported.ClientState)(nil),
		&ClientState{},
	)
}
