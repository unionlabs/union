package authz

import "cosmossdk.io/errors"

// x/authz module sentinel errors
var (
	// ErrNoAuthorizationFound error if there is no authorization found given a grant key
	ErrNoAuthorizationFound = errors.Register(ModuleName, 2, "authorization not found")
	// ErrInvalidExpirationTime error if the set expiration time is in the past
	ErrInvalidExpirationTime = errors.Register(ModuleName, 3, "expiration time of authorization should be more than current time")
	// ErrUnknownAuthorizationType error for unknown authorization type
	ErrUnknownAuthorizationType = errors.Register(ModuleName, 4, "unknown authorization type")
	// ErrNoGrantKeyFound error if the requested grant key does not exist
	ErrNoGrantKeyFound = errors.Register(ModuleName, 5, "grant key not found")
	// ErrAuthorizationExpired error if the authorization has expired
	ErrAuthorizationExpired = errors.Register(ModuleName, 6, "authorization expired")
	// ErrGranteeIsGranter error if the grantee and the granter are the same
	ErrGranteeIsGranter = errors.Register(ModuleName, 7, "grantee and granter should be different")
	// ErrAuthorizationNumOfSigners error if an authorization message does not have only one signer
	ErrAuthorizationNumOfSigners = errors.Register(ModuleName, 9, "authorization can be given to msg with only one signer")
	// ErrNegativeMaxTokens error if the max tokens is negative
	ErrNegativeMaxTokens = errors.Register(ModuleName, 12, "max tokens should be positive")
)
