package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// x/distribution module sentinel errors
var (
	ErrEmptyDelegatorAddr      = sdkerrors.Register(ModuleName, 2, "delegator address is empty")
	ErrEmptyWithdrawAddr       = sdkerrors.Register(ModuleName, 3, "withdraw address is empty")
	ErrEmptyValidatorAddr      = sdkerrors.Register(ModuleName, 4, "validator address is empty")
	ErrEmptyDelegationDistInfo = sdkerrors.Register(ModuleName, 5, "no delegation distribution info")
	ErrNoValidatorDistInfo     = sdkerrors.Register(ModuleName, 6, "no validator distribution info")
	ErrNoValidatorCommission   = sdkerrors.Register(ModuleName, 7, "no validator commission to withdraw")
	ErrSetWithdrawAddrDisabled = sdkerrors.Register(ModuleName, 8, "set withdraw address disabled")
	ErrBadDistribution         = sdkerrors.Register(ModuleName, 9, "community pool does not have sufficient coins to distribute")
	ErrInvalidProposalAmount   = sdkerrors.Register(ModuleName, 10, "invalid community pool spend proposal amount")
	ErrEmptyProposalRecipient  = sdkerrors.Register(ModuleName, 11, "invalid community pool spend proposal recipient")
	ErrNoValidatorExists       = sdkerrors.Register(ModuleName, 12, "validator does not exist")
	ErrNoDelegationExists      = sdkerrors.Register(ModuleName, 13, "delegation does not exist")
)
