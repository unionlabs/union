package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

const MaximumCounterpartyPayeeLength = 2048 // maximum length of the counterparty payee in bytes (value chosen arbitrarily)

var (
	_ sdk.Msg = (*MsgRegisterPayee)(nil)
	_ sdk.Msg = (*MsgRegisterCounterpartyPayee)(nil)
	_ sdk.Msg = (*MsgPayPacketFee)(nil)
	_ sdk.Msg = (*MsgPayPacketFeeAsync)(nil)

	_ sdk.HasValidateBasic = (*MsgRegisterPayee)(nil)
	_ sdk.HasValidateBasic = (*MsgRegisterCounterpartyPayee)(nil)
	_ sdk.HasValidateBasic = (*MsgPayPacketFee)(nil)
	_ sdk.HasValidateBasic = (*MsgPayPacketFeeAsync)(nil)
)

// NewMsgRegisterPayee creates a new instance of MsgRegisterPayee
func NewMsgRegisterPayee(portID, channelID, relayerAddr, payeeAddr string) *MsgRegisterPayee {
	return &MsgRegisterPayee{
		PortId:    portID,
		ChannelId: channelID,
		Relayer:   relayerAddr,
		Payee:     payeeAddr,
	}
}

// ValidateBasic implements sdk.Msg and performs basic stateless validation
func (msg MsgRegisterPayee) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return err
	}

	if err := host.ChannelIdentifierValidator(msg.ChannelId); err != nil {
		return err
	}

	_, err := sdk.AccAddressFromBech32(msg.Relayer)
	if err != nil {
		return errorsmod.Wrap(err, "failed to create sdk.AccAddress from relayer address")
	}

	_, err = sdk.AccAddressFromBech32(msg.Payee)
	if err != nil {
		return errorsmod.Wrap(err, "failed to create sdk.AccAddress from payee address")
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgRegisterPayee) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Relayer)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{signer}
}

// NewMsgRegisterCounterpartyPayee creates a new instance of MsgRegisterCounterpartyPayee
func NewMsgRegisterCounterpartyPayee(portID, channelID, relayerAddr, counterpartyPayeeAddr string) *MsgRegisterCounterpartyPayee {
	return &MsgRegisterCounterpartyPayee{
		PortId:            portID,
		ChannelId:         channelID,
		Relayer:           relayerAddr,
		CounterpartyPayee: counterpartyPayeeAddr,
	}
}

// ValidateBasic performs a basic check of the MsgRegisterCounterpartyAddress fields
func (msg MsgRegisterCounterpartyPayee) ValidateBasic() error {
	if err := host.PortIdentifierValidator(msg.PortId); err != nil {
		return err
	}

	if err := host.ChannelIdentifierValidator(msg.ChannelId); err != nil {
		return err
	}

	_, err := sdk.AccAddressFromBech32(msg.Relayer)
	if err != nil {
		return errorsmod.Wrap(err, "failed to create sdk.AccAddress from relayer address")
	}

	if strings.TrimSpace(msg.CounterpartyPayee) == "" {
		return ErrCounterpartyPayeeEmpty
	}

	if len(msg.CounterpartyPayee) > MaximumCounterpartyPayeeLength {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "counterparty payee address must not exceed %d bytes", MaximumCounterpartyPayeeLength)
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgRegisterCounterpartyPayee) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Relayer)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{signer}
}

// NewMsgPayPacketFee creates a new instance of MsgPayPacketFee
func NewMsgPayPacketFee(fee Fee, sourcePortID, sourceChannelID, signer string, relayers []string) *MsgPayPacketFee {
	return &MsgPayPacketFee{
		Fee:             fee,
		SourcePortId:    sourcePortID,
		SourceChannelId: sourceChannelID,
		Signer:          signer,
		Relayers:        relayers,
	}
}

// ValidateBasic performs a basic check of the MsgPayPacketFee fields
func (msg MsgPayPacketFee) ValidateBasic() error {
	// validate channelId
	if err := host.ChannelIdentifierValidator(msg.SourceChannelId); err != nil {
		return err
	}

	// validate portId
	if err := host.PortIdentifierValidator(msg.SourcePortId); err != nil {
		return err
	}

	// signer check
	if _, err := sdk.AccAddressFromBech32(msg.Signer); err != nil {
		return errorsmod.Wrap(err, "failed to convert msg.Signer into sdk.AccAddress")
	}

	// enforce relayer is not set
	if len(msg.Relayers) != 0 {
		return ErrRelayersNotEmpty
	}

	return msg.Fee.Validate()
}

// GetSigners implements sdk.Msg
func (msg MsgPayPacketFee) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}

// NewMsgPayPacketAsync creates a new instance of MsgPayPacketFee
func NewMsgPayPacketFeeAsync(packetID channeltypes.PacketId, packetFee PacketFee) *MsgPayPacketFeeAsync {
	return &MsgPayPacketFeeAsync{
		PacketId:  packetID,
		PacketFee: packetFee,
	}
}

// ValidateBasic performs a basic check of the MsgPayPacketFeeAsync fields
func (msg MsgPayPacketFeeAsync) ValidateBasic() error {
	if err := msg.PacketId.Validate(); err != nil {
		return err
	}

	return msg.PacketFee.Validate()
}

// GetSigners implements sdk.Msg
// The signer of the fee message must be the refund address
func (msg MsgPayPacketFeeAsync) GetSigners() []sdk.AccAddress {
	signer, err := sdk.AccAddressFromBech32(msg.PacketFee.RefundAddress)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{signer}
}
