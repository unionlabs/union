package types

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC connection sentinel errors
var (
	ErrConnectionExists              = errorsmod.Register(SubModuleName, 2, "connection already exists")
	ErrConnectionNotFound            = errorsmod.Register(SubModuleName, 3, "connection not found")
	ErrClientConnectionPathsNotFound = errorsmod.Register(SubModuleName, 4, "light client connection paths not found")
	ErrConnectionPath                = errorsmod.Register(SubModuleName, 5, "connection path is not associated to the given light client")
	ErrInvalidConnectionState        = errorsmod.Register(SubModuleName, 6, "invalid connection state")
	ErrInvalidCounterparty           = errorsmod.Register(SubModuleName, 7, "invalid counterparty connection")
	ErrInvalidConnection             = errorsmod.Register(SubModuleName, 8, "invalid connection")
	ErrInvalidVersion                = errorsmod.Register(SubModuleName, 9, "invalid connection version")
	ErrVersionNegotiationFailed      = errorsmod.Register(SubModuleName, 10, "connection version negotiation failed")
	ErrInvalidConnectionIdentifier   = errorsmod.Register(SubModuleName, 11, "invalid connection identifier")
)
