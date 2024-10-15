package types

import (
	"context"

	paramtypes "cosmossdk.io/x/params/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// AccountKeeper defines the expected account keeper
type AccountKeeper interface {
	NewAccount(ctx context.Context, acc sdk.AccountI) sdk.AccountI
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	SetAccount(ctx context.Context, acc sdk.AccountI)
	GetModuleAccount(ctx context.Context, name string) sdk.ModuleAccountI
	GetModuleAddress(name string) sdk.AccAddress
}

// ChannelKeeper defines the expected IBC channel keeper
type ChannelKeeper interface {
	GetChannel(ctx context.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool)
	GetNextSequenceSend(ctx context.Context, portID, channelID string) (uint64, bool)
	GetConnection(ctx context.Context, connectionID string) (connectiontypes.ConnectionEnd, error)
	GetAllChannelsWithPortPrefix(ctx context.Context, portPrefix string) []channeltypes.IdentifiedChannel
}

// PortKeeper defines the expected IBC port keeper
type PortKeeper interface {
	BindPort(ctx context.Context, portID string) *capabilitytypes.Capability
	IsBound(ctx context.Context, portID string) bool
}

// ParamSubspace defines the expected Subspace interface for module parameters.
type ParamSubspace interface {
	GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet)
	GetParamSetIfExists(ctx sdk.Context, ps paramtypes.ParamSet)
}
