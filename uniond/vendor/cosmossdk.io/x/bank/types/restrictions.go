package types

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// A MintingRestrictionFn can restrict minting of coins.
type MintingRestrictionFn func(ctx context.Context, coins sdk.Coins) error

var _ MintingRestrictionFn = NoOpMintingRestrictionFn

// NoOpMintingRestrictionFn is a no-op MintingRestrictionFn.
func NoOpMintingRestrictionFn(_ context.Context, _ sdk.Coins) error {
	return nil
}

// Then creates a composite restriction that runs this one then the provided second one.
func (r MintingRestrictionFn) Then(second MintingRestrictionFn) MintingRestrictionFn {
	return ComposeMintingRestrictions(r, second)
}

// ComposeMintingRestrictions combines multiple MintingRestrictionFn into one.
// nil entries are ignored.
// If all entries are nil, nil is returned.
// If exactly one entry is not nil, it is returned.
// Otherwise, a new MintingRestrictionFn is returned that runs the non-nil restrictions in the order they are given.
// The composition runs each minting restriction until an error is encountered and returns that error.
func ComposeMintingRestrictions(restrictions ...MintingRestrictionFn) MintingRestrictionFn {
	toRun := make([]MintingRestrictionFn, 0, len(restrictions))
	for _, r := range restrictions {
		if r != nil {
			toRun = append(toRun, r)
		}
	}
	switch len(toRun) {
	case 0:
		return nil
	case 1:
		return toRun[0]
	}
	return func(ctx context.Context, coins sdk.Coins) error {
		for _, r := range toRun {
			err := r(ctx, coins)
			if err != nil {
				return err
			}
		}
		return nil
	}
}

// A SendRestrictionFn can restrict sends and/or provide a new receiver address.
type SendRestrictionFn func(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) (newToAddr sdk.AccAddress, err error)

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (SendRestrictionFn) IsOnePerModuleType() {}

var _ SendRestrictionFn = NoOpSendRestrictionFn

// NoOpSendRestrictionFn is a no-op SendRestrictionFn.
func NoOpSendRestrictionFn(_ context.Context, _, toAddr sdk.AccAddress, _ sdk.Coins) (sdk.AccAddress, error) {
	return toAddr, nil
}

// Then creates a composite restriction that runs this one then the provided second one.
func (r SendRestrictionFn) Then(second SendRestrictionFn) SendRestrictionFn {
	return ComposeSendRestrictions(r, second)
}

// ComposeSendRestrictions combines multiple SendRestrictionFn into one.
// nil entries are ignored.
// If all entries are nil, nil is returned.
// If exactly one entry is not nil, it is returned.
// Otherwise, a new SendRestrictionFn is returned that runs the non-nil restrictions in the order they are given.
// The composition runs each send restriction until an error is encountered and returns that error,
// otherwise it returns the toAddr of the last send restriction.
func ComposeSendRestrictions(restrictions ...SendRestrictionFn) SendRestrictionFn {
	toRun := make([]SendRestrictionFn, 0, len(restrictions))
	for _, r := range restrictions {
		if r != nil {
			toRun = append(toRun, r)
		}
	}
	switch len(toRun) {
	case 0:
		return nil
	case 1:
		return toRun[0]
	}
	return func(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) (sdk.AccAddress, error) {
		var err error
		for _, r := range toRun {
			toAddr, err = r(ctx, fromAddr, toAddr, amt)
			if err != nil {
				return toAddr, err
			}
		}
		return toAddr, err
	}
}
