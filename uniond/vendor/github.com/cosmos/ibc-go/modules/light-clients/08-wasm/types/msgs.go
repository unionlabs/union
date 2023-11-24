package types

import (
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var (
	_ sdk.Msg              = (*MsgStoreCode)(nil)
	_ sdk.Msg              = (*MsgMigrateContract)(nil)
	_ sdk.Msg              = (*MsgRemoveChecksum)(nil)
	_ sdk.HasValidateBasic = (*MsgStoreCode)(nil)
	_ sdk.HasValidateBasic = (*MsgMigrateContract)(nil)
	_ sdk.HasValidateBasic = (*MsgRemoveChecksum)(nil)
)

// MsgStoreCode creates a new MsgStoreCode instance
func NewMsgStoreCode(signer string, code []byte) *MsgStoreCode {
	return &MsgStoreCode{
		Signer:       signer,
		WasmByteCode: code,
	}
}

// ValidateBasic implements sdk.HasValidateBasic
func (m MsgStoreCode) ValidateBasic() error {
	if err := ValidateWasmCode(m.WasmByteCode); err != nil {
		return err
	}

	_, err := sdk.AccAddressFromBech32(m.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return nil
}

// NewMsgRemoveChecksum creates a new MsgRemoveChecksum instance
func NewMsgRemoveChecksum(signer string, checksum []byte) *MsgRemoveChecksum {
	return &MsgRemoveChecksum{
		Signer:   signer,
		Checksum: checksum,
	}
}

// ValidateBasic implements sdk.HasValidateBasic
func (m MsgRemoveChecksum) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(m.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	if err := ValidateWasmChecksum(m.Checksum); err != nil {
		return err
	}

	return nil
}

// MsgMigrateContract creates a new MsgMigrateContract instance
func NewMsgMigrateContract(signer, clientID string, checksum, migrateMsg []byte) *MsgMigrateContract {
	return &MsgMigrateContract{
		Signer:   signer,
		ClientId: clientID,
		Checksum: checksum,
		Msg:      migrateMsg,
	}
}

// ValidateBasic implements sdk.HasValidateBasic
func (m MsgMigrateContract) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(m.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	if err := ValidateWasmChecksum(m.Checksum); err != nil {
		return err
	}

	if err := ValidateClientID(m.ClientId); err != nil {
		return err
	}

	if len(m.Msg) == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidRequest, "migrate message cannot be empty")
	}

	return nil
}
