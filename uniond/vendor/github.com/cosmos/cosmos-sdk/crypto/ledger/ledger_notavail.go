//go:build !cgo || !ledger
// +build !cgo !ledger

// test_ledger_mock

package ledger

import (
	"errors"
)

// If ledger support (build tag) has been enabled, which implies a CGO dependency,
// set the discoverLedger function which is responsible for loading the Ledger
// device at runtime or returning an error.
func init() {
	options.discoverLedger = func() (SECP256K1, error) {
		return nil, errors.New("support for ledger devices is not available in this executable")
	}

	initOptionsDefault()
}
