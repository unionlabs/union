//go:build gofuzz || go1.18

package tests

import (
	"bytes"
	"testing"

	"github.com/cosmos/go-bip39"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
)

func mnemonicToSeed(mnemonic string) []byte {
	return bip39.NewSeed(mnemonic, "" /* Default passphrase */)
}

func FuzzCryptoHDDerivePrivateKeyForPath(f *testing.F) {
	f.Fuzz(func(t *testing.T, in []byte) {
		splits := bytes.Split(in, []byte("*"))
		if len(splits) == 1 {
			return
		}
		mnemonic, path := splits[0], splits[1]
		if len(path) > 1e5 {
			// Deriving a private key takes non-trivial time proportional
			// to the path length. Skip the longer ones that trigger timeouts
			// on fuzzing infrastructure.
			return
		}
		seed := mnemonicToSeed(string(mnemonic))
		master, ch := hd.ComputeMastersFromSeed(seed)
		_, _ = hd.DerivePrivateKeyForPath(master, ch, string(path))
	})
}
