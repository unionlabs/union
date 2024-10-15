package errors

import "cosmossdk.io/errors"

// groupCodespace is the codespace for all errors defined in group package
const groupCodespace = "group"

var (
	ErrEmpty           = errors.Register(groupCodespace, 2, "value is empty")
	ErrDuplicate       = errors.Register(groupCodespace, 3, "duplicate value")
	ErrMaxLimit        = errors.Register(groupCodespace, 4, "limit exceeded")
	ErrType            = errors.Register(groupCodespace, 5, "invalid type")
	ErrInvalid         = errors.Register(groupCodespace, 6, "invalid value")
	ErrUnauthorized    = errors.Register(groupCodespace, 7, "unauthorized")
	ErrModified        = errors.Register(groupCodespace, 8, "modified")
	ErrExpired         = errors.Register(groupCodespace, 9, "expired")
	ErrMetadataTooLong = errors.Register(groupCodespace, 10, "metadata too long")
	ErrSummaryTooLong  = errors.Register(groupCodespace, 11, "summary too long")
	ErrTitleTooLong    = errors.Register(groupCodespace, 12, "title too long")
)
