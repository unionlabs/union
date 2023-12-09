package types

import (
	"encoding/base64"

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
	proofInit []byte, proofHeight clienttypes.Height, signer string,
) *MsgChannelOpenTry {
	counterparty := NewCounterparty(counterpartyPortID, counterpartyChannelID)
	channel := NewChannel(TRYOPEN, channelOrder, counterparty, connectionHops, version)
	return &MsgChannelOpenTry{
		PortId:              portID,
		Channel:             channel,
		CounterpartyVersion: counterpartyVersion,
		ProofInit:           proofInit,
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
	portID, channelID, counterpartyChannelID string, cpv string, proofTry []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelOpenAck {
	return &MsgChannelOpenAck{
		PortId:                portID,
		ChannelId:             channelID,
		CounterpartyChannelId: counterpartyChannelID,
		CounterpartyVersion:   cpv,
		ProofTry:              proofTry,
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
	portID, channelID string, proofAck []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelOpenConfirm {
	return &MsgChannelOpenConfirm{
		PortId:      portID,
		ChannelId:   channelID,
		ProofAck:    proofAck,
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
func NewMsgChannelCloseConfirm(
	portID, channelID string, proofInit []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgChannelCloseConfirm {
	return &MsgChannelCloseConfirm{
		PortId:      portID,
		ChannelId:   channelID,
		ProofInit:   proofInit,
		ProofHeight: proofHeight,
		Signer:      signer,
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
	packet Packet, proofCommitment []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgRecvPacket {
	return &MsgRecvPacket{
		Packet:          packet,
		ProofCommitment: proofCommitment,
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
	packet Packet, nextSequenceRecv uint64, proofUnreceived []byte,
	proofHeight clienttypes.Height, signer string,
) *MsgTimeout {
	return &MsgTimeout{
		Packet:           packet,
		NextSequenceRecv: nextSequenceRecv,
		ProofUnreceived:  proofUnreceived,
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

// NewMsgTimeoutOnClose constructs new MsgTimeoutOnClose
func NewMsgTimeoutOnClose(
	packet Packet, nextSequenceRecv uint64,
	proofUnreceived, proofClose []byte,
	proofHeight clienttypes.Height, signer string,
) *MsgTimeoutOnClose {
	return &MsgTimeoutOnClose{
		Packet:           packet,
		NextSequenceRecv: nextSequenceRecv,
		ProofUnreceived:  proofUnreceived,
		ProofClose:       proofClose,
		ProofHeight:      proofHeight,
		Signer:           signer,
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
	ack, proofAcked []byte,
	proofHeight clienttypes.Height,
	signer string,
) *MsgAcknowledgement {
	return &MsgAcknowledgement{
		Packet:          packet,
		Acknowledgement: ack,
		ProofAcked:      proofAcked,
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
