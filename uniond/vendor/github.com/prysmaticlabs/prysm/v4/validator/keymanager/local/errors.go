package local

import "errors"

var (
	ErrNoPasswords            = errors.New("no passwords provided for keystores")
	ErrMismatchedNumPasswords = errors.New("number of passwords does not match number of keystores")
)
