package types

import (
	"encoding/base64"
	"slices"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var (
	_ sdk.Msg = (*MsgChannelOpenInit)(nil)
	_ sdk.Msg = (*MsgChannelOpenTry)(nil)
	_ sdk.Msg = (*MsgChannelOpenAck)(nil)
	_ sdk.Msg = (*MsgChannelOpenConfirm)(nil)
	_ sdk.Msg = (*MsgChannelCloseInit)(nil)
	_ sdk.Msg = (*MsgChannelCloseConfirm)(nil)
	_ sdk.Msg = (*MsgRecvPacket)(nil)
	_ sdk.Msg = (*MsgAcknowledgement)(nil)
	_ sdk.Msg = (*MsgTimeout)(nil)
	_ sdk.Msg = (*MsgTimeoutOnClose)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeInit)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeTry)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeAck)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeConfirm)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeTimeout)(nil)
	_ sdk.Msg = (*MsgChannelUpgradeCancel)(nil)
	_ sdk.Msg = (*MsgPruneAcknowledgements)(nil)

	_ sdk.HasValidateBasic = (*MsgChannelOpenInit)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelOpenTry)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelOpenAck)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelOpenConfirm)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelCloseInit)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelCloseConfirm)(nil)
	_ sdk.HasValidateBasic = (*MsgRecvPacket)(nil)
	_ sdk.HasValidateBasic = (*MsgAcknowledgement)(nil)
	_ sdk.HasValidateBasic = (*MsgTimeout)(nil)
	_ sdk.HasValidateBasic = (*MsgTimeoutOnClose)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeInit)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeTry)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeAck)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeConfirm)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeTimeout)(nil)
	_ sdk.HasValidateBasic = (*MsgChannelUpgradeCancel)(nil)
	_ sdk.HasValidateBasic = (*MsgPruneAcknowledgements)(nil)
)

// NewMsgChannelOpenInit creates a new MsgChannelOpenInit. It sets the counterparty channel
// identifier to be empty.
func NewMsgChannelOpenInit(
	portID, version string, channelOrder Order, connectionHops []string,
	counterpartyPortID string, signer string,
) *MsgChannelOpenInit {
	counterparty := NewCounterparty(counterpartyPortID, "")
	channel := NewChannel(INIT, channelOrder, counterparty, connectionHops, version)
	return &MsgChannelOpenInit{
		PortId:  portID,
		Channel: channel,
		Signer:  signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelOpenInit) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if msg.Channel.State != INIT {
		return errorsmod.Wrapf(ErrInvalidChannelState,
			"channel state must be INIT in MsgChannelOpenInit. expected: %s, got: %s",
			INIT, msg.Channel.State,
		)
	}
	if msg.Channel.Counterparty.ChannelId != "" {
		return errorsmod.Wrap(ErrInvalidCounterparty, "counterparty channel identifier must be empty")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Channel.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgChannelOpenInit) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgChannelOpenTry creates a new MsgChannelOpenTry instance
// The version string is deprecated and will be ignored by core IBC.
// It is left as an argument for go API backwards compatibility.
func NewMsgChannelOpenTry(
	portID, version string, channelOrder Order, connectionHops []string,
	counterpartyPortID, counterpartyChannelID, counterpartyVersion string,
	initProof []byte, proofHeight clienttypes.Height, signer string,
) *MsgChannelOpenTry {
	counterparty := NewCounterparty(counterpartyPortID, counterpartyChannelID)
	channel := NewChannel(TRYOPEN, channelOrder, counterparty, connectionHops, version)
	return &MsgChannelOpenTry{
		PortId:              portID,
		Channel:             channel,
		CounterpartyVersion: counterpartyVersion,
		ProofInit:           initProof,
		ProofHeight:         proofHeight,
		Signer:              signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelOpenTry) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if msg.PreviousChannelId != "" {
		return errorsmod.Wrap(ErrInvalidChannelIdentifier, "previous channel identifier must be empty, this field has been deprecated as crossing hellos are no longer supported")
	}
	if len(msg.ProofInit) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty init proof")
	}
	if msg.Channel.State != TRYOPEN {
		return errorsmod.Wrapf(ErrInvalidChannelState,
			"channel state must be TRYOPEN in MsgChannelOpenTry. expected: %s, got: %s",
			TRYOPEN, msg.Channel.State,
		)
	}
	// counterparty validate basic allows empty counterparty channel identifiers
	if err := host.ChannelIdentifierValidator(msg.Channel.Counterparty.ChannelId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty channel ID")
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Channel.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgChannelOpenTry) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgChannelOpenAck creates a new MsgChannelOpenAck instance
func NewMsgChannelOpenAck(
	portID, channelID, counterpartyChannelID string, cpv string, tryProof []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelOpenAck {
	return &MsgChannelOpenAck{
		PortId:                portID,
		ChannelId:             channelID,
		CounterpartyChannelId: counterpartyChannelID,
		CounterpartyVersion:   cpv,
		ProofTry:              tryProof,
		ProofHeight:           proofHeight,
		Signer:                signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelOpenAck) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}
	if err := host.ChannelIdentifierValidator(msg.CounterpartyChannelId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty channel ID")
	}
	if len(msg.ProofTry) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty try proof")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgChannelOpenAck) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgChannelOpenConfirm creates a new MsgChannelOpenConfirm instance
func NewMsgChannelOpenConfirm(
	portID, channelID string, ackProof []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelOpenConfirm {
	return &MsgChannelOpenConfirm{
		PortId:      portID,
		ChannelId:   channelID,
		ProofAck:    ackProof,
		ProofHeight: proofHeight,
		Signer:      signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelOpenConfirm) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}
	if len(msg.ProofAck) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty acknowledgement proof")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgChannelOpenConfirm) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgChannelCloseInit creates a new MsgChannelCloseInit instance
func NewMsgChannelCloseInit(
	portID string, channelID string, signer string,
) *MsgChannelCloseInit {
	return &MsgChannelCloseInit{
		PortId:    portID,
		ChannelId: channelID,
		Signer:    signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelCloseInit) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgChannelCloseInit) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgChannelCloseConfirm creates a new MsgChannelCloseConfirm instance
// Breakage in v9.0.0 will allow for the counterparty upgrade sequence to be provided.
// Please use NewMsgChannelCloseConfirmWithCounterpartyUpgradeSequence to provide the
// counterparty upgrade sequence in this version.
func NewMsgChannelCloseConfirm(
	portID, channelID string, initProof []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelCloseConfirm {
	return &MsgChannelCloseConfirm{
		PortId:                      portID,
		ChannelId:                   channelID,
		ProofInit:                   initProof,
		ProofHeight:                 proofHeight,
		Signer:                      signer,
		CounterpartyUpgradeSequence: 0,
	}
}

// NewMsgChannelCloseConfirmWithCounterpartyUpgradeSequence creates a new MsgChannelCloseConfirm instance
// with a non-zero counterparty upgrade sequence.
func NewMsgChannelCloseConfirmWithCounterpartyUpgradeSequence(
	portID, channelID string, initProof []byte, proofHeight clienttypes.Height,
	signer string, counterpartyUpgradeSequence uint64,
) *MsgChannelCloseConfirm {
	return &MsgChannelCloseConfirm{
		PortId:                      portID,
		ChannelId:                   channelID,
		ProofInit:                   initProof,
		ProofHeight:                 proofHeight,
		Signer:                      signer,
		CounterpartyUpgradeSequence: counterpartyUpgradeSequence,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelCloseConfirm) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}
	if len(msg.ProofInit) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty init proof")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgChannelCloseConfirm) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgRecvPacket constructs new MsgRecvPacket
func NewMsgRecvPacket(
	packet Packet, commitmentProof []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgRecvPacket {
	return &MsgRecvPacket{
		Packet:          packet,
		ProofCommitment: commitmentProof,
		ProofHeight:     proofHeight,
		Signer:          signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgRecvPacket) ValidateBasic() error {
	if len(msg.ProofCommitment) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty commitment proof")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Packet.ValidateBasic()
}

// GetDataSignBytes returns the base64-encoded bytes used for the
// data field when signing the packet.
func (msg MsgRecvPacket) GetDataSignBytes() []byte {
	s := "\"" + base64.StdEncoding.EncodeToString(msg.Packet.Data) + "\""
	return []byte(s)
}

// GetSigners implements sdk.Msg
func (msg MsgRecvPacket) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgTimeout constructs new MsgTimeout
func NewMsgTimeout(
	packet Packet, nextSequenceRecv uint64, unreceivedProof []byte,
	proofHeight clienttypes.Height, signer string,
) *MsgTimeout {
	return &MsgTimeout{
		Packet:           packet,
		NextSequenceRecv: nextSequenceRecv,
		ProofUnreceived:  unreceivedProof,
		ProofHeight:      proofHeight,
		Signer:           signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgTimeout) ValidateBasic() error {
	if len(msg.ProofUnreceived) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty unreceived proof")
	}
	if msg.NextSequenceRecv == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidSequence, "next sequence receive cannot be 0")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Packet.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgTimeout) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgTimeoutOnClose constructs a new MsgTimeoutOnClose.
// The counterparty upgrade sequence is set to 0. Breakage in
// v9.0.0 will allow the counterparty upgrade sequence to be provided.
// Please use NewMsgTimeoutOnCloseWithCounterpartyUpgradeSequence in this version
// to provide the counterparty upgrade sequence.
func NewMsgTimeoutOnClose(
	packet Packet, nextSequenceRecv uint64,
	unreceivedProof, closeProof []byte,
	proofHeight clienttypes.Height, signer string,
) *MsgTimeoutOnClose {
	return &MsgTimeoutOnClose{
		Packet:                      packet,
		NextSequenceRecv:            nextSequenceRecv,
		ProofUnreceived:             unreceivedProof,
		ProofClose:                  closeProof,
		ProofHeight:                 proofHeight,
		Signer:                      signer,
		CounterpartyUpgradeSequence: 0,
	}
}

// NewMsgTimeoutOnCloseWithCounterpartyUpgradeSequence constructs a new MsgTimeoutOnClose
// with the provided counterparty upgrade sequence.
func NewMsgTimeoutOnCloseWithCounterpartyUpgradeSequence(
	packet Packet, nextSequenceRecv uint64,
	unreceivedProof, closeProof []byte,
	proofHeight clienttypes.Height, signer string,
	counterpartyUpgradeSequence uint64,
) *MsgTimeoutOnClose {
	return &MsgTimeoutOnClose{
		Packet:                      packet,
		NextSequenceRecv:            nextSequenceRecv,
		ProofUnreceived:             unreceivedProof,
		ProofClose:                  closeProof,
		ProofHeight:                 proofHeight,
		Signer:                      signer,
		CounterpartyUpgradeSequence: counterpartyUpgradeSequence,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgTimeoutOnClose) ValidateBasic() error {
	if msg.NextSequenceRecv == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidSequence, "next sequence receive cannot be 0")
	}
	if len(msg.ProofUnreceived) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty unreceived proof")
	}
	if len(msg.ProofClose) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof of closed counterparty channel end")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Packet.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgTimeoutOnClose) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgAcknowledgement constructs a new MsgAcknowledgement
func NewMsgAcknowledgement(
	packet Packet,
	ack, ackedProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgAcknowledgement {
	return &MsgAcknowledgement{
		Packet:          packet,
		Acknowledgement: ack,
		ProofAcked:      ackedProof,
		ProofHeight:     proofHeight,
		Signer:          signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgAcknowledgement) ValidateBasic() error {
	if len(msg.ProofAcked) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty acknowledgement proof")
	}
	if len(msg.Acknowledgement) == 0 {
		return errorsmod.Wrap(ErrInvalidAcknowledgement, "ack bytes cannot be empty")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Packet.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgAcknowledgement) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

var _ sdk.Msg = &MsgChannelUpgradeInit{}

// NewMsgChannelUpgradeInit constructs a new MsgChannelUpgradeInit
// nolint:interfacer
func NewMsgChannelUpgradeInit(
	portID, channelID string,
	upgradeFields UpgradeFields,
	signer string,
) *MsgChannelUpgradeInit {
	return &MsgChannelUpgradeInit{
		PortId:    portID,
		ChannelId: channelID,
		Fields:    upgradeFields,
		Signer:    signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeInit) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return msg.Fields.ValidateBasic()
}

var _ sdk.Msg = &MsgChannelUpgradeTry{}

// NewMsgChannelUpgradeTry constructs a new MsgChannelUpgradeTry
// nolint:interfacer
func NewMsgChannelUpgradeTry(
	portID,
	channelID string,
	proposedConnectionHops []string,
	counterpartyUpgradeFields UpgradeFields,
	counterpartyUpgradeSequence uint64,
	channelProof []byte,
	upgradeProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgChannelUpgradeTry {
	return &MsgChannelUpgradeTry{
		PortId:                        portID,
		ChannelId:                     channelID,
		ProposedUpgradeConnectionHops: proposedConnectionHops,
		CounterpartyUpgradeFields:     counterpartyUpgradeFields,
		CounterpartyUpgradeSequence:   counterpartyUpgradeSequence,
		ProofChannel:                  channelProof,
		ProofUpgrade:                  upgradeProof,
		ProofHeight:                   proofHeight,
		Signer:                        signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeTry) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	if len(msg.ProposedUpgradeConnectionHops) == 0 {
		return errorsmod.Wrap(ErrInvalidUpgrade, "proposed connection hops cannot be empty")
	}

	if err := msg.CounterpartyUpgradeFields.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "error validating counterparty upgrade fields")
	}

	if msg.CounterpartyUpgradeSequence == 0 {
		return errorsmod.Wrap(ErrInvalidUpgradeSequence, "counterparty sequence cannot be 0")
	}

	if len(msg.ProofChannel) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty channel proof")
	}

	if len(msg.ProofUpgrade) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty upgrade proof")
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

var _ sdk.Msg = &MsgChannelUpgradeAck{}

// NewMsgChannelUpgradeAck constructs a new MsgChannelUpgradeAck
// nolint:interfacer
func NewMsgChannelUpgradeAck(portID, channelID string, counterpartyUpgrade Upgrade, channelProof, upgradeProof []byte, proofHeight clienttypes.Height, signer string) *MsgChannelUpgradeAck {
	return &MsgChannelUpgradeAck{
		PortId:              portID,
		ChannelId:           channelID,
		CounterpartyUpgrade: counterpartyUpgrade,
		ProofChannel:        channelProof,
		ProofUpgrade:        upgradeProof,
		ProofHeight:         proofHeight,
		Signer:              signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeAck) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}
	if len(msg.ProofChannel) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty channel proof")
	}
	if len(msg.ProofUpgrade) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty upgrade sequence proof")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Signer); err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return msg.CounterpartyUpgrade.ValidateBasic()
}

var _ sdk.Msg = &MsgChannelUpgradeConfirm{}

// NewMsgChannelUpgradeConfirm constructs a new MsgChannelUpgradeConfirm
func NewMsgChannelUpgradeConfirm(
	portID,
	channelID string,
	counterpartyChannelState State,
	counterpartyUpgrade Upgrade,
	channelProof,
	upgradeProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgChannelUpgradeConfirm {
	return &MsgChannelUpgradeConfirm{
		PortId:                   portID,
		ChannelId:                channelID,
		CounterpartyChannelState: counterpartyChannelState,
		CounterpartyUpgrade:      counterpartyUpgrade,
		ProofChannel:             channelProof,
		ProofUpgrade:             upgradeProof,
		ProofHeight:              proofHeight,
		Signer:                   signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeConfirm) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	if !slices.Contains([]State{FLUSHING, FLUSHCOMPLETE}, msg.CounterpartyChannelState) {
		return errorsmod.Wrapf(ErrInvalidChannelState, "expected channel state to be one of: %s or %s, got: %s", FLUSHING, FLUSHCOMPLETE, msg.CounterpartyChannelState)
	}

	if len(msg.ProofChannel) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty channel proof")
	}

	if len(msg.ProofUpgrade) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty upgrade proof")
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return msg.CounterpartyUpgrade.ValidateBasic()
}

var _ sdk.Msg = &MsgChannelUpgradeOpen{}

// NewMsgChannelUpgradeOpen constructs a new MsgChannelUpgradeOpen
// nolint:interfacer
func NewMsgChannelUpgradeOpen(
	portID,
	channelID string,
	counterpartyChannelState State,
	counterpartyUpgradeSequence uint64,
	channelProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgChannelUpgradeOpen {
	return &MsgChannelUpgradeOpen{
		PortId:                      portID,
		ChannelId:                   channelID,
		CounterpartyChannelState:    counterpartyChannelState,
		CounterpartyUpgradeSequence: counterpartyUpgradeSequence,
		ProofChannel:                channelProof,
		ProofHeight:                 proofHeight,
		Signer:                      signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeOpen) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	if len(msg.ProofChannel) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty channel proof")
	}

	if !slices.Contains([]State{FLUSHCOMPLETE, OPEN}, msg.CounterpartyChannelState) {
		return errorsmod.Wrapf(ErrInvalidChannelState, "expected channel state to be one of: [%s, %s], got: %s", FLUSHCOMPLETE, OPEN, msg.CounterpartyChannelState)
	}

	if msg.CounterpartyUpgradeSequence == 0 {
		return errorsmod.Wrap(ErrInvalidUpgradeSequence, "counterparty upgrade sequence must be non-zero")
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

var _ sdk.Msg = &MsgChannelUpgradeTimeout{}

// NewMsgChannelUpgradeTimeout constructs a new MsgChannelUpgradeTimeout
// nolint:interfacer
func NewMsgChannelUpgradeTimeout(
	portID, channelID string,
	counterpartyChannel Channel,
	channelProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgChannelUpgradeTimeout {
	return &MsgChannelUpgradeTimeout{
		PortId:              portID,
		ChannelId:           channelID,
		CounterpartyChannel: counterpartyChannel,
		ProofChannel:        channelProof,
		ProofHeight:         proofHeight,
		Signer:              signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgChannelUpgradeTimeout) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	if len(msg.ProofChannel) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof")
	}

	if !slices.Contains([]State{FLUSHING, OPEN}, msg.CounterpartyChannel.State) {
		return errorsmod.Wrapf(ErrInvalidChannelState, "expected counterparty channel state to be one of: [%s, %s], got: %s", FLUSHING, OPEN, msg.CounterpartyChannel.State)
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

var _ sdk.Msg = &MsgChannelUpgradeCancel{}

// NewMsgChannelUpgradeCancel constructs a new MsgChannelUpgradeCancel
// nolint:interfacer
func NewMsgChannelUpgradeCancel(
	portID, channelID string,
	errorReceipt ErrorReceipt,
	errorReceiptProof []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgChannelUpgradeCancel {
	return &MsgChannelUpgradeCancel{
		PortId:            portID,
		ChannelId:         channelID,
		ErrorReceipt:      errorReceipt,
		ProofErrorReceipt: errorReceiptProof,
		ProofHeight:       proofHeight,
		Signer:            signer,
	}
}

// ValidateBasic implements sdk.Msg. No checks are done for ErrorReceipt and ProofErrorReceipt
// since they are not required if the current channel state is not in FLUSHCOMPLETE and the signer
// is the designated authority (e.g. the governance module).
func (msg MsgChannelUpgradeCancel) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

// NewMsgUpdateChannelParams creates a new instance of MsgUpdateParams.
func NewMsgUpdateChannelParams(authority string, params Params) *MsgUpdateParams {
	return &MsgUpdateParams{
		Authority: authority,
		Params:    params,
	}
}

// ValidateBasic performs basic checks on a MsgUpdateParams.
func (msg *MsgUpdateParams) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Params.Validate()
}

// NewMsgPruneAcknowledgements creates a new instance of MsgPruneAcknowledgements.
func NewMsgPruneAcknowledgements(portID, channelID string, limit uint64, signer string) *MsgPruneAcknowledgements {
	return &MsgPruneAcknowledgements{
		PortId:    portID,
		ChannelId: channelID,
		Limit:     limit,
		Signer:    signer,
	}
}

// ValidateBasic performs basic checks on a MsgPruneAcknowledgements.
func (msg *MsgPruneAcknowledgements) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}

	if !IsValidChannelID(msg.ChannelId) {
		return ErrInvalidChannelIdentifier
	}

	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	if msg.Limit == 0 {
		return errorsmod.Wrap(ErrInvalidPruningLimit, "number of acknowledgements to prune must be greater than 0")
	}

	return nil
}
