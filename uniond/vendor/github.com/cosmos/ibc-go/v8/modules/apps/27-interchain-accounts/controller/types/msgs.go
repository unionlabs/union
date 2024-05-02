package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

const MaximumOwnerLength = 2048 // maximum length of the owner in bytes (value chosen arbitrarily)

var (
	_ sdk.Msg = (*MsgRegisterInterchainAccount)(nil)
	_ sdk.Msg = (*MsgSendTx)(nil)
	_ sdk.Msg = (*MsgUpdateParams)(nil)

	_ sdk.HasValidateBasic = (*MsgRegisterInterchainAccount)(nil)
	_ sdk.HasValidateBasic = (*MsgSendTx)(nil)
	_ sdk.HasValidateBasic = (*MsgUpdateParams)(nil)
)

// NewMsgRegisterInterchainAccountWithOrdering creates a new instance of MsgRegisterInterchainAccount.
func NewMsgRegisterInterchainAccountWithOrdering(connectionID, owner, version string, ordering channeltypes.Order) *MsgRegisterInterchainAccount {
	return &MsgRegisterInterchainAccount{
		ConnectionId: connectionID,
		Owner:        owner,
		Version:      version,
		Ordering:     ordering,
	}
}

// NewMsgRegisterInterchainAccount creates a new instance of MsgRegisterInterchainAccount.
// It uses channeltypes.ORDERED as the default ordering. Breakage in v9.0.0 will allow the ordering to be provided
// directly. Use NewMsgRegisterInterchainAccountWithOrder to provide the ordering in previous versions.
func NewMsgRegisterInterchainAccount(connectionID, owner, version string) *MsgRegisterInterchainAccount {
	return &MsgRegisterInterchainAccount{
		ConnectionId: connectionID,
		Owner:        owner,
		Version:      version,
		Ordering:     channeltypes.ORDERED,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgRegisterInterchainAccount) ValidateBasic() error {
	if err := host.ConnectionIdentifierValidator(msg.ConnectionId); err != nil {
		return errorsmod.Wrap(err, "invalid connection ID")
	}

	if strings.TrimSpace(msg.Owner) == "" {
		return errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "owner address cannot be empty")
	}

	if len(msg.Owner) > MaximumOwnerLength {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "owner address must not exceed %d bytes", MaximumOwnerLength)
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgRegisterInterchainAccount) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Owner)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{accAddr}
}

// NewMsgSendTx creates a new instance of MsgSendTx
func NewMsgSendTx(owner, connectionID string, relativeTimeoutTimestamp uint64, packetData icatypes.InterchainAccountPacketData) *MsgSendTx {
	return &MsgSendTx{
		ConnectionId:    connectionID,
		Owner:           owner,
		RelativeTimeout: relativeTimeoutTimestamp,
		PacketData:      packetData,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgSendTx) ValidateBasic() error {
	if err := host.ConnectionIdentifierValidator(msg.ConnectionId); err != nil {
		return errorsmod.Wrap(err, "invalid connection ID")
	}

	if strings.TrimSpace(msg.Owner) == "" {
		return errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "owner address cannot be empty")
	}

	if len(msg.Owner) > MaximumOwnerLength {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "owner address must not exceed %d bytes", MaximumOwnerLength)
	}

	if err := msg.PacketData.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "invalid interchain account packet data")
	}

	if msg.RelativeTimeout == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidRequest, "relative timeout cannot be zero")
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgSendTx) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Owner)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{accAddr}
}

// NewMsgUpdateParams creates a new MsgUpdateParams instance
func NewMsgUpdateParams(signer string, params Params) *MsgUpdateParams {
	return &MsgUpdateParams{
		Signer: signer,
		Params: params,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgUpdateParams) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgUpdateParams) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{accAddr}
}
