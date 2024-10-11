package feegrant

import "cosmossdk.io/errors"

// Codes for governance errors
const (
	DefaultCodespace = ModuleName
)

var (
	// ErrFeeLimitExceeded error if there are not enough allowance to cover the fees
	ErrFeeLimitExceeded = errors.Register(DefaultCodespace, 2, "fee limit exceeded")
	// ErrFeeLimitExpired error if the allowance has expired
	ErrFeeLimitExpired = errors.Register(DefaultCodespace, 3, "fee allowance expired")
	// ErrInvalidDuration error if the Duration is invalid or doesn't match the expiration
	ErrInvalidDuration = errors.Register(DefaultCodespace, 4, "invalid duration")
	// ErrNoAllowance error if there is no allowance for that pair
	ErrNoAllowance = errors.Register(DefaultCodespace, 5, "no allowance")
	// ErrNoMessages error if there is no message
	ErrNoMessages = errors.Register(DefaultCodespace, 6, "allowed messages are empty")
	// ErrMessageNotAllowed error if message is not allowed
	ErrMessageNotAllowed = errors.Register(DefaultCodespace, 7, "message not allowed")
)
