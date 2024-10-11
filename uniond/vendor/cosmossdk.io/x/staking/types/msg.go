package types

import (
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/core/address"
	coretransaction "cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var (
	_ coretransaction.Msg                  = &MsgCreateValidator{}
	_ gogoprotoany.UnpackInterfacesMessage = (*MsgCreateValidator)(nil)
	_ coretransaction.Msg                  = &MsgEditValidator{}
	_ coretransaction.Msg                  = &MsgDelegate{}
	_ coretransaction.Msg                  = &MsgUndelegate{}
	_ coretransaction.Msg                  = &MsgBeginRedelegate{}
	_ coretransaction.Msg                  = &MsgCancelUnbondingDelegation{}
	_ coretransaction.Msg                  = &MsgUpdateParams{}
)

// NewMsgCreateValidator creates a new MsgCreateValidator instance.
// Delegator address and validator address are the same.
func NewMsgCreateValidator(
	valAddr string, pubKey cryptotypes.PubKey,
	selfDelegation sdk.Coin, description Description, commission CommissionRates, minSelfDelegation math.Int,
) (*MsgCreateValidator, error) {
	var pkAny *codectypes.Any
	if pubKey != nil {
		var err error
		if pkAny, err = codectypes.NewAnyWithValue(pubKey); err != nil {
			return nil, err
		}
	}
	return &MsgCreateValidator{
		Description:       description,
		ValidatorAddress:  valAddr,
		Pubkey:            pkAny,
		Value:             selfDelegation,
		Commission:        commission,
		MinSelfDelegation: minSelfDelegation,
	}, nil
}

// Validate validates the MsgCreateValidator sdk msg.
func (msg MsgCreateValidator) Validate(ac address.Codec) error {
	// note that unmarshaling from bech32 ensures both non-empty and valid
	_, err := ac.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	if msg.Pubkey == nil {
		return ErrEmptyValidatorPubKey
	}

	if !msg.Value.IsValid() || !msg.Value.Amount.IsPositive() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "invalid delegation amount")
	}

	if msg.Description == (Description{}) {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "empty description")
	}

	if msg.Commission == (CommissionRates{}) {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "empty commission")
	}

	if err := msg.Commission.Validate(); err != nil {
		return err
	}

	if !msg.MinSelfDelegation.IsPositive() {
		return errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"minimum self delegation must be a positive integer",
		)
	}

	if msg.Value.Amount.LT(msg.MinSelfDelegation) {
		return ErrSelfDelegationBelowMinimum
	}

	return nil
}

// GetMoniker returns the moniker of the validator
func (msg MsgCreateValidator) GetMoniker() string {
	return msg.Description.GetMoniker()
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgCreateValidator) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var pubKey cryptotypes.PubKey
	return unpacker.UnpackAny(msg.Pubkey, &pubKey)
}

// NewMsgEditValidator creates a new MsgEditValidator instance
func NewMsgEditValidator(valAddr string, description Description, newRate *math.LegacyDec, newMinSelfDelegation *math.Int) *MsgEditValidator {
	return &MsgEditValidator{
		Description:       description,
		CommissionRate:    newRate,
		ValidatorAddress:  valAddr,
		MinSelfDelegation: newMinSelfDelegation,
	}
}

// NewMsgDelegate creates a new MsgDelegate instance.
func NewMsgDelegate(delAddr, valAddr string, amount sdk.Coin) *MsgDelegate {
	return &MsgDelegate{
		DelegatorAddress: delAddr,
		ValidatorAddress: valAddr,
		Amount:           amount,
	}
}

// NewMsgBeginRedelegate creates a new MsgBeginRedelegate instance.
func NewMsgBeginRedelegate(
	delAddr, valSrcAddr, valDstAddr string, amount sdk.Coin,
) *MsgBeginRedelegate {
	return &MsgBeginRedelegate{
		DelegatorAddress:    delAddr,
		ValidatorSrcAddress: valSrcAddr,
		ValidatorDstAddress: valDstAddr,
		Amount:              amount,
	}
}

// NewMsgUndelegate creates a new MsgUndelegate instance.
func NewMsgUndelegate(delAddr, valAddr string, amount sdk.Coin) *MsgUndelegate {
	return &MsgUndelegate{
		DelegatorAddress: delAddr,
		ValidatorAddress: valAddr,
		Amount:           amount,
	}
}

// NewMsgCancelUnbondingDelegation creates a new MsgCancelUnbondingDelegation instance.
func NewMsgCancelUnbondingDelegation(delAddr, valAddr string, creationHeight int64, amount sdk.Coin) *MsgCancelUnbondingDelegation {
	return &MsgCancelUnbondingDelegation{
		DelegatorAddress: delAddr,
		ValidatorAddress: valAddr,
		Amount:           amount,
		CreationHeight:   creationHeight,
	}
}

// NewMsgRotateConsPubKey creates a new MsgRotateConsPubKey instance.
func NewMsgRotateConsPubKey(valAddr string, pubKey cryptotypes.PubKey) (*MsgRotateConsPubKey, error) {
	var pkAny *codectypes.Any
	if pubKey != nil {
		var err error
		if pkAny, err = codectypes.NewAnyWithValue(pubKey); err != nil {
			return nil, err
		}
	}
	return &MsgRotateConsPubKey{
		ValidatorAddress: valAddr,
		NewPubkey:        pkAny,
	}, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgRotateConsPubKey) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var pubKey cryptotypes.PubKey
	return unpacker.UnpackAny(msg.NewPubkey, &pubKey)
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (hi ConsPubKeyRotationHistory) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var oldPubKey cryptotypes.PubKey
	err := unpacker.UnpackAny(hi.OldConsPubkey, &oldPubKey)
	if err != nil {
		return err
	}
	var newPubKey cryptotypes.PubKey
	return unpacker.UnpackAny(hi.NewConsPubkey, &newPubKey)
}
