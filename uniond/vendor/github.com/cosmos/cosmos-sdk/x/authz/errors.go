package authz

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// x/authz module sentinel errors
var (
	// ErrNoAuthorizationFound error if there is no authorization found given a grant key
	ErrNoAuthorizationFound = sdkerrors.Register(ModuleName, 2, "authorization not found")
	// ErrInvalidExpirationTime error if the set expiration time is in the past
	ErrInvalidExpirationTime = sdkerrors.Register(ModuleName, 3, "expiration time of authorization should be more than current time")
	// ErrUnknownAuthorizationType error for unknown authorization type
	ErrUnknownAuthorizationType = sdkerrors.Register(ModuleName, 4, "unknown authorization type")
	// ErrNoGrantKeyFound error if the requested grant key does not exist
	ErrNoGrantKeyFound = sdkerrors.Register(ModuleName, 5, "grant key not found")
	// ErrAuthorizationExpired error if the authorization has expired
	ErrAuthorizationExpired = sdkerrors.Register(ModuleName, 6, "authorization expired")
	// ErrGranteeIsGranter error if the grantee and the granter are the same
	ErrGranteeIsGranter = sdkerrors.Register(ModuleName, 7, "grantee and granter should be different")
	// ErrAuthorizationNumOfSigners error if an authorization message does not have only one signer
	ErrAuthorizationNumOfSigners = sdkerrors.Register(ModuleName, 9, "authorization can be given to msg with only one signer")
	// ErrNegativeMaxTokens error if the max tokens is negative
	ErrNegativeMaxTokens = sdkerrors.Register(ModuleName, 12, "max tokens should be positive")
)
