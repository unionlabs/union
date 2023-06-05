package types

import (
	"bytes"
	"encoding/json"
	"errors"
	"strings"

	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// RawContractMessage defines a json message that is sent or returned by a wasm contract.
// This type can hold any type of bytes. Until validateBasic is called there should not be
// any assumptions made that the data is valid syntax or semantic.
type RawContractMessage []byte

func (r RawContractMessage) MarshalJSON() ([]byte, error) {
	return json.RawMessage(r).MarshalJSON()
}

func (r *RawContractMessage) UnmarshalJSON(b []byte) error {
	if r == nil {
		return errors.New("unmarshalJSON on nil pointer")
	}
	*r = append((*r)[0:0], b...)
	return nil
}

func (r *RawContractMessage) ValidateBasic() error {
	if r == nil {
		return ErrEmpty
	}
	if !json.Valid(*r) {
		return ErrInvalid
	}
	return nil
}

// Bytes returns raw bytes type
func (r RawContractMessage) Bytes() []byte {
	return r
}

// Equal content is equal json. Byte equal but this can change in the future.
func (r RawContractMessage) Equal(o RawContractMessage) bool {
	return bytes.Equal(r.Bytes(), o.Bytes())
}

func (msg MsgStoreCode) Route() string {
	return RouterKey
}

func (msg MsgStoreCode) Type() string {
	return "store-code"
}

func (msg MsgStoreCode) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return err
	}

	if err := validateWasmCode(msg.WASMByteCode, MaxWasmSize); err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "code bytes %s", err.Error())
	}

	if msg.InstantiatePermission != nil {
		if err := msg.InstantiatePermission.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "instantiate permission")
		}
		// AccessTypeOnlyAddress is still considered valid as legacy instantiation permission
		// but not for new contracts
		if msg.InstantiatePermission.Permission == AccessTypeOnlyAddress {
			return ErrInvalid.Wrap("unsupported type, use AccessTypeAnyOfAddresses instead")
		}
	}
	return nil
}

func (msg MsgStoreCode) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgStoreCode) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgInstantiateContract) Route() string {
	return RouterKey
}

func (msg MsgInstantiateContract) Type() string {
	return "instantiate"
}

func (msg MsgInstantiateContract) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}

	if msg.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}

	if err := ValidateLabel(msg.Label); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "label is required")
	}

	if !msg.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}

	if len(msg.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(msg.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}
	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

func (msg MsgInstantiateContract) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgInstantiateContract) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgExecuteContract) Route() string {
	return RouterKey
}

func (msg MsgExecuteContract) Type() string {
	return "execute"
}

func (msg MsgExecuteContract) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}

	if !msg.Funds.IsValid() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, "sentFunds")
	}
	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

func (msg MsgExecuteContract) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgExecuteContract) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

// GetMsg returns the payload message send to the contract
func (msg MsgExecuteContract) GetMsg() RawContractMessage {
	return msg.Msg
}

// GetFunds returns tokens send to the contract
func (msg MsgExecuteContract) GetFunds() sdk.Coins {
	return msg.Funds
}

// GetContract returns the bech32 address of the contract
func (msg MsgExecuteContract) GetContract() string {
	return msg.Contract
}

func (msg MsgMigrateContract) Route() string {
	return RouterKey
}

func (msg MsgMigrateContract) Type() string {
	return "migrate"
}

func (msg MsgMigrateContract) ValidateBasic() error {
	if msg.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}

	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}

	return nil
}

func (msg MsgMigrateContract) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgMigrateContract) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

// GetMsg returns the payload message send to the contract
func (msg MsgMigrateContract) GetMsg() RawContractMessage {
	return msg.Msg
}

// GetFunds returns tokens send to the contract
func (msg MsgMigrateContract) GetFunds() sdk.Coins {
	return sdk.NewCoins()
}

// GetContract returns the bech32 address of the contract
func (msg MsgMigrateContract) GetContract() string {
	return msg.Contract
}

func (msg MsgUpdateAdmin) Route() string {
	return RouterKey
}

func (msg MsgUpdateAdmin) Type() string {
	return "update-contract-admin"
}

func (msg MsgUpdateAdmin) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if _, err := sdk.AccAddressFromBech32(msg.NewAdmin); err != nil {
		return errorsmod.Wrap(err, "new admin")
	}
	if strings.EqualFold(msg.Sender, msg.NewAdmin) {
		return errorsmod.Wrap(ErrInvalid, "new admin is the same as the old")
	}
	return nil
}

func (msg MsgUpdateAdmin) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgUpdateAdmin) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgClearAdmin) Route() string {
	return RouterKey
}

func (msg MsgClearAdmin) Type() string {
	return "clear-contract-admin"
}

func (msg MsgClearAdmin) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	return nil
}

func (msg MsgClearAdmin) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgClearAdmin) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgIBCSend) Route() string {
	return RouterKey
}

func (msg MsgIBCSend) Type() string {
	return "wasm-ibc-send"
}

func (msg MsgIBCSend) ValidateBasic() error {
	return nil
}

func (msg MsgIBCSend) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgIBCSend) GetSigners() []sdk.AccAddress {
	return nil
}

func (msg MsgIBCCloseChannel) Route() string {
	return RouterKey
}

func (msg MsgIBCCloseChannel) Type() string {
	return "wasm-ibc-close"
}

func (msg MsgIBCCloseChannel) ValidateBasic() error {
	return nil
}

func (msg MsgIBCCloseChannel) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgIBCCloseChannel) GetSigners() []sdk.AccAddress {
	return nil
}

var _ sdk.Msg = &MsgInstantiateContract2{}

func (msg MsgInstantiateContract2) Route() string {
	return RouterKey
}

func (msg MsgInstantiateContract2) Type() string {
	return "instantiate2"
}

func (msg MsgInstantiateContract2) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}

	if msg.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}

	if err := ValidateLabel(msg.Label); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "label is required")
	}

	if !msg.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}

	if len(msg.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(msg.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}
	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	if err := ValidateSalt(msg.Salt); err != nil {
		return errorsmod.Wrap(err, "salt")
	}
	return nil
}

func (msg MsgInstantiateContract2) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgInstantiateContract2) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgUpdateInstantiateConfig) Route() string {
	return RouterKey
}

func (msg MsgUpdateInstantiateConfig) Type() string {
	return "update-instantiate-config"
}

func (msg MsgUpdateInstantiateConfig) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Sender); err != nil {
		return errorsmod.Wrap(err, "sender")
	}

	if msg.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}

	if msg.NewInstantiatePermission == nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "instantiate permission is required")
	}

	if err := msg.NewInstantiatePermission.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "instantiate permission")
	}
	// AccessTypeOnlyAddress is still considered valid as legacy instantiation permission
	// but not for new contracts
	if msg.NewInstantiatePermission.Permission == AccessTypeOnlyAddress {
		return ErrInvalid.Wrap("unsupported type, use AccessTypeAnyOfAddresses instead")
	}
	// AccessTypeOnlyAddress is still considered valid as legacy instantiation permission
	// but not for new contracts
	if msg.NewInstantiatePermission.Permission == AccessTypeOnlyAddress {
		return ErrInvalid.Wrap("unsupported type, use AccessTypeAnyOfAddresses instead")
	}

	return nil
}

func (msg MsgUpdateInstantiateConfig) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgUpdateInstantiateConfig) GetSigners() []sdk.AccAddress {
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{senderAddr}
}

func (msg MsgUpdateParams) Route() string {
	return RouterKey
}

func (msg MsgUpdateParams) Type() string {
	return "update-params"
}

func (msg MsgUpdateParams) GetSigners() []sdk.AccAddress {
	authority, err := sdk.AccAddressFromBech32(msg.Authority)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{authority}
}

func (msg MsgUpdateParams) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgUpdateParams) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrap(err, "authority")
	}
	return msg.Params.ValidateBasic()
}

func (msg MsgPinCodes) Route() string {
	return RouterKey
}

func (msg MsgPinCodes) Type() string {
	return "pin-codes"
}

func (msg MsgPinCodes) GetSigners() []sdk.AccAddress {
	authority, err := sdk.AccAddressFromBech32(msg.Authority)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{authority}
}

func (msg MsgPinCodes) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgPinCodes) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrap(err, "authority")
	}
	if len(msg.CodeIDs) == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "empty code ids")
	}
	return nil
}

func (msg MsgUnpinCodes) Route() string {
	return RouterKey
}

func (msg MsgUnpinCodes) Type() string {
	return "unpin-codes"
}

func (msg MsgUnpinCodes) GetSigners() []sdk.AccAddress {
	authority, err := sdk.AccAddressFromBech32(msg.Authority)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{authority}
}

func (msg MsgUnpinCodes) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgUnpinCodes) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrap(err, "authority")
	}
	if len(msg.CodeIDs) == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "empty code ids")
	}
	return nil
}

func (msg MsgSudoContract) Route() string {
	return RouterKey
}

func (msg MsgSudoContract) Type() string {
	return "sudo-contract"
}

func (msg MsgSudoContract) GetSigners() []sdk.AccAddress {
	authority, err := sdk.AccAddressFromBech32(msg.Authority)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{authority}
}

func (msg MsgSudoContract) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgSudoContract) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrap(err, "authority")
	}
	if _, err := sdk.AccAddressFromBech32(msg.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

func (msg MsgStoreAndInstantiateContract) Route() string {
	return RouterKey
}

func (msg MsgStoreAndInstantiateContract) Type() string {
	return "store-and-instantiate-contract"
}

func (msg MsgStoreAndInstantiateContract) GetSigners() []sdk.AccAddress {
	authority, err := sdk.AccAddressFromBech32(msg.Authority)
	if err != nil { // should never happen as valid basic rejects invalid addresses
		panic(err.Error())
	}
	return []sdk.AccAddress{authority}
}

func (msg MsgStoreAndInstantiateContract) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

func (msg MsgStoreAndInstantiateContract) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Authority); err != nil {
		return errorsmod.Wrap(err, "authority")
	}

	if err := ValidateLabel(msg.Label); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "label is required")
	}

	if !msg.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}

	if len(msg.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(msg.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}

	if err := ValidateVerificationInfo(msg.Source, msg.Builder, msg.CodeHash); err != nil {
		return errorsmod.Wrapf(err, "code verification info")
	}

	if err := msg.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}

	if err := validateWasmCode(msg.WASMByteCode, MaxWasmSize); err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "code bytes %s", err.Error())
	}

	if msg.InstantiatePermission != nil {
		if err := msg.InstantiatePermission.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "instantiate permission")
		}
		// AccessTypeOnlyAddress is still considered valid as legacy instantiation permission
		// but not for new contracts
		if msg.InstantiatePermission.Permission == AccessTypeOnlyAddress {
			return ErrInvalid.Wrap("unsupported type, use AccessTypeAnyOfAddresses instead")
		}
	}
	return nil
}
