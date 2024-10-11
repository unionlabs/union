//go:build !linux
// +build !linux

package keyring

import (
	"io"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/ledger"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// Options define the options of the Keyring.
type Options struct {
	// supported signing algorithms for keyring
	SupportedAlgos SigningAlgoList
	// supported signing algorithms for Ledger
	SupportedAlgosLedger SigningAlgoList
	// define Ledger Derivation function
	LedgerDerivation func() (ledger.SECP256K1, error)
	// define Ledger key generation function
	LedgerCreateKey func([]byte) types.PubKey
	// define Ledger app name
	LedgerAppName string
	// indicate whether Ledger should skip DER Conversion on signature,
	// depending on which format (DER or BER) the Ledger app returns signatures
	LedgerSigSkipDERConv bool
}

func New(
	appName, backend, rootDir string, userInput io.Reader, cdc codec.Codec, opts ...Option,
) (Keyring, error) {
	return newKeyringGeneric(appName, backend, rootDir, userInput, cdc, opts...)
}
