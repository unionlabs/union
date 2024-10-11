package types

import (
	"errors"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func NewGenesisState(cf []*ContinuousFund, budget []*Budget) *GenesisState {
	return &GenesisState{
		ContinuousFund: cf,
		Budget:         budget,
		LastBalance:    math.ZeroInt(),
		Distributions:  []*Distribution{},
	}
}

func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		ContinuousFund: []*ContinuousFund{},
		Budget:         []*Budget{},
	}
}

// ValidateGenesis validates the genesis state of protocolpool genesis input
func ValidateGenesis(gs *GenesisState) error {
	for _, cf := range gs.ContinuousFund {
		if err := validateContinuousFund(*cf); err != nil {
			return err
		}
	}
	for _, bp := range gs.Budget {
		if err := validateBudget(*bp); err != nil {
			return err
		}
	}
	return nil
}

func validateBudget(bp Budget) error {
	if bp.RecipientAddress == "" {
		return errors.New("recipient cannot be empty")
	}

	// Validate BudgetPerTranche
	if bp.BudgetPerTranche == nil || bp.BudgetPerTranche.IsZero() {
		return errors.New("budget per tranche cannot be zero")
	}
	if err := bp.BudgetPerTranche.Validate(); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, bp.BudgetPerTranche.String())
	}

	if bp.TranchesLeft == 0 {
		return errors.New("invalid budget proposal: tranches must be greater than zero")
	}

	if bp.Period == nil || *bp.Period == 0 {
		return errors.New("invalid budget proposal: period length should be greater than zero")
	}
	return nil
}

func validateContinuousFund(cf ContinuousFund) error {
	if cf.Recipient == "" {
		return errors.New("recipient cannot be empty")
	}

	// Validate percentage
	if cf.Percentage.IsNil() || cf.Percentage.IsZero() {
		return errors.New("percentage cannot be zero or empty")
	}
	if cf.Percentage.IsNegative() {
		return errors.New("percentage cannot be negative")
	}
	if cf.Percentage.GT(math.LegacyOneDec()) {
		return errors.New("percentage cannot be greater than one")
	}
	return nil
}
