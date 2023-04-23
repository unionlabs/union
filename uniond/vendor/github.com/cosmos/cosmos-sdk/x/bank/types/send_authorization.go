package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/authz"
)

// TODO: Revisit this once we have proper gas fee framework.
// Ref: https://github.com/cosmos/cosmos-sdk/issues/9054
// Ref: https://github.com/cosmos/cosmos-sdk/discussions/9072
const gasCostPerIteration = uint64(10)

var _ authz.Authorization = &SendAuthorization{}

// NewSendAuthorization creates a new SendAuthorization object.
func NewSendAuthorization(spendLimit sdk.Coins, allowed []sdk.AccAddress) *SendAuthorization {
	return &SendAuthorization{
		AllowList:  toBech32Addresses(allowed),
		SpendLimit: spendLimit,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a SendAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgSend{})
}

// Accept implements Authorization.Accept.
func (a SendAuthorization) Accept(ctx sdk.Context, msg sdk.Msg) (authz.AcceptResponse, error) {
	mSend, ok := msg.(*MsgSend)
	if !ok {
		return authz.AcceptResponse{}, sdkerrors.ErrInvalidType.Wrap("type mismatch")
	}

	limitLeft, isNegative := a.SpendLimit.SafeSub(mSend.Amount...)
	if isNegative {
		return authz.AcceptResponse{}, sdkerrors.ErrInsufficientFunds.Wrapf("requested amount is more than spend limit")
	}

	isAddrExists := false
	toAddr := mSend.ToAddress
	allowedList := a.GetAllowList()
	for _, addr := range allowedList {
		ctx.GasMeter().ConsumeGas(gasCostPerIteration, "send authorization")
		if addr == toAddr {
			isAddrExists = true
			break
		}
	}

	if len(allowedList) > 0 && !isAddrExists {
		return authz.AcceptResponse{}, sdkerrors.ErrUnauthorized.Wrapf("cannot send to %s address", toAddr)
	}

	if limitLeft.IsZero() {
		return authz.AcceptResponse{Accept: true, Delete: true}, nil
	}

	return authz.AcceptResponse{Accept: true, Delete: false, Updated: &SendAuthorization{SpendLimit: limitLeft, AllowList: allowedList}}, nil
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a SendAuthorization) ValidateBasic() error {
	if len(a.SpendLimit) == 0 {
		return sdkerrors.ErrInvalidCoins.Wrap("spend limit cannot be nil")
	}
	if !a.SpendLimit.IsAllPositive() {
		return sdkerrors.ErrInvalidCoins.Wrapf("spend limit must be positive")
	}

	found := make(map[string]bool, 0)
	for i := 0; i < len(a.AllowList); i++ {
		if found[a.AllowList[i]] {
			return ErrDuplicateEntry
		}

		found[a.AllowList[i]] = true
	}

	return nil
}

func toBech32Addresses(allowed []sdk.AccAddress) []string {
	if len(allowed) == 0 {
		return nil
	}

	allowedAddrs := make([]string, len(allowed))
	for i, addr := range allowed {
		allowedAddrs[i] = addr.String()
	}

	return allowedAddrs
}
