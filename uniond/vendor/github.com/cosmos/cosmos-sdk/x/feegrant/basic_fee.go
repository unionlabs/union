package feegrant

import (
	time "time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ FeeAllowanceI = (*BasicAllowance)(nil)

// Accept can use fee payment requested as well as timestamp of the current block
// to determine whether or not to process this. This is checked in
// Keeper.UseGrantedFees and the return values should match how it is handled there.
//
// If it returns an error, the fee payment is rejected, otherwise it is accepted.
// The FeeAllowance implementation is expected to update it's internal state
// and will be saved again after an acceptance.
//
// If remove is true (regardless of the error), the FeeAllowance will be deleted from storage
// (eg. when it is used up). (See call to RevokeAllowance in Keeper.UseGrantedFees)
func (a *BasicAllowance) Accept(ctx sdk.Context, fee sdk.Coins, _ []sdk.Msg) (bool, error) {
	if a.Expiration != nil && a.Expiration.Before(ctx.BlockTime()) {
		return true, sdkerrors.Wrap(ErrFeeLimitExpired, "basic allowance")
	}

	if a.SpendLimit != nil {
		left, invalid := a.SpendLimit.SafeSub(fee...)
		if invalid {
			return false, sdkerrors.Wrap(ErrFeeLimitExceeded, "basic allowance")
		}

		a.SpendLimit = left
		return left.IsZero(), nil
	}

	return false, nil
}

// ValidateBasic implements FeeAllowance and enforces basic sanity checks
func (a BasicAllowance) ValidateBasic() error {
	if a.SpendLimit != nil {
		if !a.SpendLimit.IsValid() {
			return sdkerrors.Wrapf(sdkerrors.ErrInvalidCoins, "send amount is invalid: %s", a.SpendLimit)
		}
		if !a.SpendLimit.IsAllPositive() {
			return sdkerrors.Wrap(sdkerrors.ErrInvalidCoins, "spend limit must be positive")
		}
	}

	if a.Expiration != nil && a.Expiration.Unix() < 0 {
		return sdkerrors.Wrap(ErrInvalidDuration, "expiration time cannot be negative")
	}

	return nil
}

func (a BasicAllowance) ExpiresAt() (*time.Time, error) {
	return a.Expiration, nil
}
