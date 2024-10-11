package exported

import (
	"context"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
)

// ScopedKeeper defines the expected x/capability scoped keeper interface
type ScopedKeeper interface {
	NewCapability(ctx context.Context, name string) (*capabilitytypes.Capability, error)
	GetCapability(ctx context.Context, name string) (*capabilitytypes.Capability, bool)
	AuthenticateCapability(ctx context.Context, cap *capabilitytypes.Capability, name string) bool
	LookupModules(ctx context.Context, name string) ([]string, *capabilitytypes.Capability, error)
	ClaimCapability(ctx context.Context, cap *capabilitytypes.Capability, name string) error
}
