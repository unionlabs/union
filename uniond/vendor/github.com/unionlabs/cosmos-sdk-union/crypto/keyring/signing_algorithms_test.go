package keyring

import (
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
)

func TestNewSigningAlgoByString(t *testing.T) {
	tests := []struct {
		name         string
		algoStr      string
		isSupported  bool
		expectedAlgo SignatureAlgo
		expectedErr  error
	}{
		{
			"supported algorithm",
			"secp256k1",
			true,
			hd.Secp256k1,
			nil,
		},
		{
			"not supported",
			"notsupportedalgo",
			false,
			nil,
			ErrUnsupportedSigningAlgo,
		},
	}

	list := SigningAlgoList{hd.Secp256k1}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			algorithm, err := NewSigningAlgoFromString(tt.algoStr, list)
			if tt.isSupported {
				require.Equal(t, hd.Secp256k1, algorithm)
			} else {
				require.ErrorIs(t, err, tt.expectedErr)
			}
		})
	}
}

func TestDerive(t *testing.T) {
	tests := []struct {
		name            string
		algo            SignatureAlgo
		hdPath          string
		mnemonic        string
		bip39Passphrase string
		derivedPriv     string
	}{
		{
			name:            "secp256k1",
			algo:            hd.Secp256k1,
			hdPath:          "m/44'/118'/0'/0/0",
			mnemonic:        "circle music snake select deal march this romance until often welcome rich staff trigger drip exit there reopen denial insect hockey just wealth process",
			bip39Passphrase: "",
			derivedPriv:     "1b5884fab5c22aeffef369f3454076cec534c75a4ee71add1245e2c8342994a2",
		},
		{
			name:            "secp256k1 with bip39Passphrase",
			algo:            hd.Secp256k1,
			hdPath:          "m/44'/118'/0'/0/0",
			mnemonic:        "circle music snake select deal march this romance until often welcome rich staff trigger drip exit there reopen denial insect hockey just wealth process",
			bip39Passphrase: "test",
			derivedPriv:     "d5f925b9472b3793e839eae7722e5cef6766a91fad55cc42abea5e457ed5f648",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			derivedPriv, err := tt.algo.Derive()(tt.mnemonic, tt.bip39Passphrase, tt.hdPath)
			require.NoError(t, err)

			decodedPriv, err := hex.DecodeString(tt.derivedPriv)
			require.NoError(t, err)
			require.Equal(t, derivedPriv, decodedPriv)
		})
	}
}

func TestAltSigningAlgoList_Contains(t *testing.T) {
	list := SigningAlgoList{hd.Secp256k1}

	require.True(t, list.Contains(hd.Secp256k1))
	require.False(t, list.Contains(notSupportedAlgo{}))
}

func TestAltSigningAlgoList_String(t *testing.T) {
	list := SigningAlgoList{hd.Secp256k1, notSupportedAlgo{}}
	require.Equal(t, fmt.Sprintf("%s,notSupported", hd.Secp256k1Type), list.String())
}

type notSupportedAlgo struct{}

func (n notSupportedAlgo) Name() hd.PubKeyType {
	return "notSupported"
}

func (n notSupportedAlgo) Derive() hd.DeriveFn {
	return hd.Secp256k1.Derive()
}

func (n notSupportedAlgo) Generate() hd.GenerateFn {
	return hd.Secp256k1.Generate()
}
