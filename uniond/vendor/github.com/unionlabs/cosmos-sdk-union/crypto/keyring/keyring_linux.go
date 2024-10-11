//go:build linux
// +build linux

package keyring

import (
	"fmt"
	"io"

	"github.com/99designs/keyring"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/ledger"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// Linux-only backend options.
const BackendKeyctl = "keyctl"

func KeyctlScopeUser(options *Options)        { setKeyctlScope(options, "user") }
func KeyctlScopeUserSession(options *Options) { setKeyctlScope(options, "usersession") }
func KeyctlScopeSession(options *Options)     { setKeyctlScope(options, "session") }
func KeyctlScopeProcess(options *Options)     { setKeyctlScope(options, "process") }
func KeyctlScopeThread(options *Options)      { setKeyctlScope(options, "thread") }

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
	// KeyctlScope defines the scope of the keyctl's keyring.
	KeyctlScope string
}

func newKeyctlBackendConfig(appName, _ string, _ io.Reader, opts ...Option) keyring.Config {
	options := Options{
		KeyctlScope: keyctlDefaultScope, // currently "process"
	}

	for _, optionFn := range opts {
		optionFn(&options)
	}

	return keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		ServiceName:     appName,
		KeyCtlScope:     options.KeyctlScope,
	}
}

// New creates a new instance of a keyring.
// Keyring options can be applied when generating the new instance.
// Available backends are "os", "file", "kwallet", "memory", "pass", "test", "keyctl".
func New(
	appName, backend, rootDir string, userInput io.Reader, cdc codec.Codec, opts ...Option,
) (Keyring, error) {
	if backend != BackendKeyctl {
		return newKeyringGeneric(appName, backend, rootDir, userInput, cdc, opts...)
	}

	db, err := keyring.Open(newKeyctlBackendConfig(appName, "", userInput, opts...))
	if err != nil {
		return nil, fmt.Errorf("couldn't open keyring for %q: %w", appName, err)
	}

	return newKeystore(db, cdc, backend, opts...), nil
}

func setKeyctlScope(options *Options, scope string) { options.KeyctlScope = scope }

// this is private as it is meant to be here for SDK devs convenience
// as the user does not need to pick any default when he wants to
// initialize keyctl with the default scope.
const keyctlDefaultScope = "process"
