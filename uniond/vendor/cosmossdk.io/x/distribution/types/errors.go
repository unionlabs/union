package types

import "cosmossdk.io/errors"

// x/distribution module sentinel errors
var (
	ErrEmptyDelegatorAddr      = errors.Register(ModuleName, 2, "delegator address is empty")
	ErrEmptyWithdrawAddr       = errors.Register(ModuleName, 3, "withdraw address is empty")
	ErrEmptyValidatorAddr      = errors.Register(ModuleName, 4, "validator address is empty")
	ErrEmptyDelegationDistInfo = errors.Register(ModuleName, 5, "no delegation distribution info")
	ErrNoValidatorDistInfo     = errors.Register(ModuleName, 6, "no validator distribution info")
	ErrNoValidatorCommission   = errors.Register(ModuleName, 7, "no validator commission to withdraw")
	ErrSetWithdrawAddrDisabled = errors.Register(ModuleName, 8, "set withdraw address disabled")
	ErrBadDistribution         = errors.Register(ModuleName, 9, "community pool does not have sufficient coins to distribute")
	ErrInvalidProposalAmount   = errors.Register(ModuleName, 10, "invalid community pool spend proposal amount")
	ErrEmptyProposalRecipient  = errors.Register(ModuleName, 11, "invalid community pool spend proposal recipient")
	ErrNoValidatorExists       = errors.Register(ModuleName, 12, "validator does not exist")
	ErrNoDelegationExists      = errors.Register(ModuleName, 13, "delegation does not exist")
	ErrInvalidProposalContent  = errors.Register(ModuleName, 14, "invalid proposal content")
	ErrInvalidSigner           = errors.Register(ModuleName, 15, "expected authority account as only signer for proposal message")
)
