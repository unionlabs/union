package common

import "errors"

// ErrZeroKey describes an error due to a zero secret key.
var ErrZeroKey = errors.New("received secret key is zero")

// ErrSecretUnmarshal describes an error which happens during unmarshalling
// a secret key.
var ErrSecretUnmarshal = errors.New("could not unmarshal bytes into secret key")

// ErrInfinitePubKey describes an error due to an infinite public key.
var ErrInfinitePubKey = errors.New("received an infinite public key")
