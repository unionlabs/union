// Package errors provides a shared set of errors for use in the SDK,
// aliases functionality in the cosmossdk.io/errors module
// that used to be in this package, and provides some helpers for converting
// errors to ABCI response code.
//
// New code should generally import cosmossdk.io/errors directly
// and define a custom set of errors in custom codespace, rather than importing
// this package.
package errors
