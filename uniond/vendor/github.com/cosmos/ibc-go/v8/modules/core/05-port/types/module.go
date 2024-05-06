package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// IBCModule defines an interface that implements all the callbacks
// that modules must define as specified in ICS-26
type IBCModule interface {
	// OnChanOpenInit will verify that the relayer-chosen parameters
	// are valid and perform any custom INIT logic.
	// It may return an error if the chosen parameters are invalid
	// in which case the handshake is aborted.
	// If the provided version string is non-empty, OnChanOpenInit should return
	// the version string if valid or an error if the provided version is invalid.
	// If the version string is empty, OnChanOpenInit is expected to
	// return a default version string representing the version(s) it supports.
	// If there is no default version string for the application,
	// it should return an error if provided version is empty string.
	OnChanOpenInit(
		ctx sdk.Context,
		order channeltypes.Order,
		connectionHops []string,
		portID string,
		channelID string,
		channelCap *capabilitytypes.Capability,
		counterparty channeltypes.Counterparty,
		version string,
	) (string, error)

	// OnChanOpenTry will verify the relayer-chosen parameters along with the
	// counterparty-chosen version string and perform custom TRY logic.
	// If the relayer-chosen parameters are invalid, the callback must return
	// an error to abort the handshake. If the counterparty-chosen version is not
	// compatible with this modules supported versions, the callback must return
	// an error to abort the handshake. If the versions are compatible, the try callback
	// must select the final version string and return it to core IBC.
	// OnChanOpenTry may also perform custom initialization logic
	OnChanOpenTry(
		ctx sdk.Context,
		order channeltypes.Order,
		connectionHops []string,
		portID,
		channelID string,
		channelCap *capabilitytypes.Capability,
		counterparty channeltypes.Counterparty,
		counterpartyVersion string,
	) (version string, err error)

	// OnChanOpenAck will error if the counterparty selected version string
	// is invalid to abort the handshake. It may also perform custom ACK logic.
	OnChanOpenAck(
		ctx sdk.Context,
		portID,
		channelID string,
		counterpartyChannelID string,
		counterpartyVersion string,
	) error

	// OnChanOpenConfirm will perform custom CONFIRM logic and may error to abort the handshake.
	OnChanOpenConfirm(
		ctx sdk.Context,
		portID,
		channelID string,
	) error

	OnChanCloseInit(
		ctx sdk.Context,
		portID,
		channelID string,
	) error

	OnChanCloseConfirm(
		ctx sdk.Context,
		portID,
		channelID string,
	) error

	// OnRecvPacket must return an acknowledgement that implements the Acknowledgement interface.
	// In the case of an asynchronous acknowledgement, nil should be returned.
	// If the acknowledgement returned is successful, the state changes on callback are written,
	// otherwise the application state changes are discarded. In either case the packet is received
	// and the acknowledgement is written (in synchronous cases).
	OnRecvPacket(
		ctx sdk.Context,
		packet channeltypes.Packet,
		relayer sdk.AccAddress,
	) exported.Acknowledgement

	OnAcknowledgementPacket(
		ctx sdk.Context,
		packet channeltypes.Packet,
		acknowledgement []byte,
		relayer sdk.AccAddress,
	) error

	OnTimeoutPacket(
		ctx sdk.Context,
		packet channeltypes.Packet,
		relayer sdk.AccAddress,
	) error
}

// UpgradableModule defines the callbacks required to perform a channel upgrade.
// Note: applications must ensure that state related to packet processing remains unmodified until the OnChanUpgradeOpen callback is executed.
// This guarantees that in-flight packets are correctly flushed using the existing channel parameters.
type UpgradableModule interface {
	// OnChanUpgradeInit enables additional custom logic to be executed when the channel upgrade is initialized.
	// It must validate the proposed version, order, and connection hops.
	// NOTE: in the case of crossing hellos, this callback may be executed on both chains.
	// NOTE: Any IBC application state changes made in this callback handler are not committed.
	OnChanUpgradeInit(
		ctx sdk.Context,
		portID, channelID string,
		proposedOrder channeltypes.Order,
		proposedConnectionHops []string,
		proposedVersion string,
	) (string, error)

	// OnChanUpgradeTry enables additional custom logic to be executed in the ChannelUpgradeTry step of the
	// channel upgrade handshake. It must validate the proposed version (provided by the counterparty), order,
	// and connection hops.
	// NOTE: Any IBC application state changes made in this callback handler are not committed.
	OnChanUpgradeTry(
		ctx sdk.Context,
		portID, channelID string,
		proposedOrder channeltypes.Order,
		proposedConnectionHops []string,
		counterpartyVersion string,
	) (string, error)

	// OnChanUpgradeAck enables additional custom logic to be executed in the ChannelUpgradeAck step of the
	// channel upgrade handshake. It must validate the version proposed by the counterparty.
	// NOTE: Any IBC application state changes made in this callback handler are not committed.
	OnChanUpgradeAck(
		ctx sdk.Context,
		portID,
		channelID,
		counterpartyVersion string,
	) error

	// OnChanUpgradeOpen enables additional custom logic to be executed when the channel upgrade has successfully completed, and the channel
	// has returned to the OPEN state. Any logic associated with changing of the channel fields should be performed
	// in this callback.
	OnChanUpgradeOpen(
		ctx sdk.Context,
		portID,
		channelID string,
		proposedOrder channeltypes.Order,
		proposedConnectionHops []string,
		proposedVersion string,
	)
}

// ICS4Wrapper implements the ICS4 interfaces that IBC applications use to send packets and acknowledgements.
type ICS4Wrapper interface {
	SendPacket(
		ctx sdk.Context,
		chanCap *capabilitytypes.Capability,
		sourcePort string,
		sourceChannel string,
		timeoutHeight clienttypes.Height,
		timeoutTimestamp uint64,
		data []byte,
	) (sequence uint64, err error)

	WriteAcknowledgement(
		ctx sdk.Context,
		chanCap *capabilitytypes.Capability,
		packet exported.PacketI,
		ack exported.Acknowledgement,
	) error

	GetAppVersion(
		ctx sdk.Context,
		portID,
		channelID string,
	) (string, bool)
}

// Middleware must implement IBCModule to wrap communication from core IBC to underlying application
// and ICS4Wrapper to wrap communication from underlying application to core IBC.
type Middleware interface {
	IBCModule
	ICS4Wrapper
}

// PacketDataUnmarshaler defines an optional interface which allows a middleware to
// request the packet data to be unmarshaled by the base application.
type PacketDataUnmarshaler interface {
	// UnmarshalPacketData unmarshals the packet data into a concrete type
	UnmarshalPacketData([]byte) (interface{}, error)
}
