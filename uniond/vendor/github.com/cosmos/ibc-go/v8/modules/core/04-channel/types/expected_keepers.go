package types

import (
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected account IBC client keeper
type ClientKeeper interface {
	GetClientStatus(ctx sdk.Context, clientState exported.ClientState, clientID string) exported.Status
	GetClientState(ctx sdk.Context, clientID string) (exported.ClientState, bool)
	GetClientConsensusState(ctx sdk.Context, clientID string, height exported.Height) (exported.ConsensusState, bool)
	ClientStore(ctx sdk.Context, clientID string) storetypes.KVStore
}

// ConnectionKeeper expected account IBC connection keeper
type ConnectionKeeper interface {
	GetConnection(ctx sdk.Context, connectionID string) (connectiontypes.ConnectionEnd, bool)
	GetTimestampAtHeight(
		ctx sdk.Context,
		connection connectiontypes.ConnectionEnd,
		height exported.Height,
	) (uint64, error)
	VerifyChannelState(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		channel exported.ChannelI,
	) error
	VerifyPacketCommitment(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		sequence uint64,
		commitmentBytes []byte,
	) error
	VerifyPacketAcknowledgement(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		sequence uint64,
		acknowledgement []byte,
	) error
	VerifyPacketReceiptAbsence(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		sequence uint64,
	) error
	VerifyNextSequenceRecv(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		nextSequenceRecv uint64,
	) error
	VerifyChannelUpgrade(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		upgrade Upgrade,
	) error
	VerifyChannelUpgradeError(
		ctx sdk.Context,
		connection exported.ConnectionI,
		height exported.Height,
		proof []byte,
		portID,
		channelID string,
		errorReceipt ErrorReceipt,
	) error
}

// PortKeeper expected account IBC port keeper
type PortKeeper interface {
	Authenticate(ctx sdk.Context, key *capabilitytypes.Capability, portID string) bool
}
