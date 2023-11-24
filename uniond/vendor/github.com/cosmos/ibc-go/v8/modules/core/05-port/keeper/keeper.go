package keeper

import (
	"fmt"

	"cosmossdk.io/log"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	"github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// Keeper defines the IBC connection keeper
type Keeper struct {
	Router *types.Router

	scopedKeeper exported.ScopedKeeper
}

// NewKeeper creates a new IBC connection Keeper instance
func NewKeeper(sck exported.ScopedKeeper) Keeper {
	return Keeper{
		scopedKeeper: sck,
	}
}

// Logger returns a module-specific logger.
func (Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", "x/"+exported.ModuleName+"/"+types.SubModuleName)
}

// IsBound checks a given port ID is already bounded.
func (k Keeper) IsBound(ctx sdk.Context, portID string) bool {
	_, ok := k.scopedKeeper.GetCapability(ctx, host.PortPath(portID))
	return ok
}

// BindPort binds to a port and returns the associated capability.
// Ports must be bound statically when the chain starts in `app.go`.
// The capability must then be passed to a module which will need to pass
// it as an extra parameter when calling functions on the IBC module.
func (k *Keeper) BindPort(ctx sdk.Context, portID string) *capabilitytypes.Capability {
	if err := host.PortIdentifierValidator(portID); err != nil {
		panic(err.Error())
	}

	if k.IsBound(ctx, portID) {
		panic(fmt.Errorf("port %s is already bound", portID))
	}

	key, err := k.scopedKeeper.NewCapability(ctx, host.PortPath(portID))
	if err != nil {
		panic(err.Error())
	}

	k.Logger(ctx).Info("port binded", "port", portID)
	return key
}

// Authenticate authenticates a capability key against a port ID
// by checking if the memory address of the capability was previously
// generated and bound to the port (provided as a parameter) which the capability
// is being authenticated against.
func (k Keeper) Authenticate(ctx sdk.Context, key *capabilitytypes.Capability, portID string) bool {
	if err := host.PortIdentifierValidator(portID); err != nil {
		panic(err.Error())
	}

	return k.scopedKeeper.AuthenticateCapability(ctx, key, host.PortPath(portID))
}

// LookupModuleByPort will return the IBCModule along with the capability associated with a given portID
func (k Keeper) LookupModuleByPort(ctx sdk.Context, portID string) (string, *capabilitytypes.Capability, error) {
	modules, capability, err := k.scopedKeeper.LookupModules(ctx, host.PortPath(portID))
	if err != nil {
		return "", nil, err
	}

	return types.GetModuleOwner(modules), capability, nil
}
