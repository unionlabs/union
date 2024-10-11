package keyring

import (
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"

	"github.com/99designs/keyring"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/bcrypt"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	cosmosbcrypt "github.com/cosmos/cosmos-sdk/crypto/keys/bcrypt"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

const (
	someKey = "theKey"
	theID   = "theID"
	otherID = "otherID"
)

func init() {
	crypto.BcryptSecurityParameter = 1
}

func getCodec() codec.Codec {
	registry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)
	return codec.NewProtoCodec(registry)
}

func TestNewKeyring(t *testing.T) {
	cdc := getCodec()

	tests := []struct {
		name        string
		appName     string
		backend     string
		dir         string
		userInput   io.Reader
		cdc         codec.Codec
		expectedErr error
	}{
		{
			name:        "file backend",
			appName:     "cosmos",
			backend:     BackendFile,
			dir:         t.TempDir(),
			userInput:   strings.NewReader(""),
			cdc:         cdc,
			expectedErr: nil,
		},
		{
			name:        "unknown backend",
			appName:     "cosmos",
			backend:     "unknown",
			dir:         t.TempDir(),
			userInput:   strings.NewReader(""),
			cdc:         cdc,
			expectedErr: ErrUnknownBacked,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.appName, tt.backend, tt.dir, tt.userInput, tt.cdc)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.Nil(t, kr)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestNewMnemonic(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name          string
		backend       string
		reader        *strings.Reader
		userInput     string
		path          string
		algo          SignatureAlgo
		uid           string
		language      Language
		expectedError error
	}{
		{
			name:          "create new mnemonic",
			backend:       BackendMemory,
			reader:        strings.NewReader(""),
			userInput:     "password\npassword\n",
			path:          sdk.FullFundraiserPath,
			algo:          hd.Secp256k1,
			uid:           "foo",
			language:      English,
			expectedError: nil,
		},
		{
			name:          "not supported algo",
			backend:       BackendMemory,
			reader:        strings.NewReader(""),
			userInput:     "password\npassword\n",
			path:          sdk.FullFundraiserPath,
			algo:          notSupportedAlgo{},
			uid:           "foo",
			language:      English,
			expectedError: ErrUnsupportedSigningAlgo,
		},
		{
			name:          "unsupported language",
			backend:       BackendMemory,
			reader:        strings.NewReader(""),
			userInput:     "password\npassword\n",
			path:          sdk.FullFundraiserPath,
			algo:          hd.Secp256k1,
			uid:           "foo",
			language:      Spanish,
			expectedError: ErrUnsupportedLanguage,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New("cosmos", tt.backend, t.TempDir(), tt.reader, cdc)
			require.NoError(t, err)
			tt.reader.Reset(tt.userInput)
			k, _, err := kr.NewMnemonic(tt.uid, tt.language, tt.path, DefaultBIP39Passphrase, tt.algo)
			if tt.expectedError == nil {
				require.NoError(t, err)
				require.Equal(t, tt.uid, k.Name)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedError))
			}
		})
	}
}

func TestKeyringDirectory(t *testing.T) {
	dir := t.TempDir()
	kb, err := New("keybasename", "test", dir, nil, getCodec())
	require.NoError(t, err)

	// create some random directory inside the keyring directory to check migrate ignores
	// all files other than *.info
	newPath := filepath.Join(dir, "random")
	require.NoError(t, os.Mkdir(newPath, 0o755))
	items, err := os.ReadDir(dir)
	require.NoError(t, err)
	require.GreaterOrEqual(t, len(items), 1)
	keys, err := kb.List()
	require.NoError(t, err)
	require.Empty(t, keys)

	_, _, err = kb.NewMnemonic("uid", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)

	items, err = os.ReadDir(dir)
	require.NoError(t, err)
	require.GreaterOrEqual(t, len(items), 2)
}

func TestNewKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
	}{
		{
			name:    "key creation",
			backend: BackendTest,
			uid:     "newKey",
		},
		{
			name:    "in memory key creation",
			backend: BackendMemory,
			uid:     "newKey",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			// Check empty state
			l, err := kb.List()
			require.NoError(t, err)
			require.Empty(t, l)

			_, err = kb.Key(tt.uid)
			require.Error(t, err)

			r, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			require.Equal(t, tt.uid, r.Name)

			k, err := kb.Key(tt.uid)
			require.NoError(t, err)

			addr, err := accAddr(k)
			require.NoError(t, err)
			_, err = kb.KeyByAddress(addr)
			require.NoError(t, err)

			addr, err = codectestutil.CodecOptions{}.GetAddressCodec().StringToBytes("cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t")
			require.NoError(t, err)
			_, err = kb.KeyByAddress(addr)
			require.NotNil(t, err)

			// list shows them in order
			keyS, err := kb.List()
			require.NoError(t, err)
			require.Equal(t, 1, len(keyS))
			require.Equal(t, tt.uid, keyS[0].Name)
		})
	}
}

func TestGetPub(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
	}{
		{
			name:    "correct get",
			backend: BackendTest,
			uid:     "getKey",
		},
		{
			name:    "in memory correct get",
			backend: BackendMemory,
			uid:     "getMemoryKey",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			r, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			require.Equal(t, tt.uid, r.Name)

			k, err := kb.Key(tt.uid)
			require.NoError(t, err)
			_, err = k.GetPubKey()
			require.NoError(t, err)

			keyS, err := kb.List()
			require.NoError(t, err)
			require.Equal(t, 1, len(keyS))
		})
	}
}

func TestDeleteKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
	}{
		{
			name:    "delete",
			backend: BackendTest,
			uid:     "key",
		},
		{
			name:    "in memory delete",
			backend: BackendMemory,
			uid:     "key",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			r, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			require.Equal(t, tt.uid, r.Name)

			err = kb.Delete(tt.uid)
			require.NoError(t, err)
			list, err := kb.List()
			require.NoError(t, err)
			require.Empty(t, list)
		})
	}
}

func TestOfflineKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
	}{
		{
			name:    "offline creation",
			backend: BackendTest,
			uid:     "offline",
		},
		{
			name:    "in memory offline creation",
			backend: BackendMemory,
			uid:     "offline",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			priv := ed25519.GenPrivKey()
			pub := priv.PubKey()
			k, err := kb.SaveOfflineKey(tt.uid, pub)
			require.NoError(t, err)
			require.Equal(t, tt.uid, k.Name)

			key, err := k.GetPubKey()
			require.NoError(t, err)
			require.Equal(t, pub, key)

			require.NotNil(t, k.GetOffline())
			keys, err := kb.List()
			require.NoError(t, err)
			require.Equal(t, 1, len(keys))

			err = kb.Delete(tt.uid)
			require.NoError(t, err)
			keys, err = kb.List()
			require.NoError(t, err)
			require.Empty(t, keys)
		})
	}
}

func TestSignVerifyKeyRing(t *testing.T) {
	dir := t.TempDir()
	cdc := getCodec()

	kb, err := New("keybasename", "test", dir, nil, cdc)
	require.NoError(t, err)
	algo := hd.Secp256k1

	n1, n2, n3 := "some dude", "a dudette", "dude-ish"

	// create two users and get their info
	kr1, _, err := kb.NewMnemonic(n1, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, algo)
	require.Nil(t, err)

	kr2, _, err := kb.NewMnemonic(n2, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, algo)
	require.Nil(t, err)

	// let's try to sign some messages
	d1 := []byte("my first message")
	d2 := []byte("some other important info!")
	d3 := []byte("feels like I forgot something...")

	// try signing both data with both ..
	s11, pub1, err := kb.Sign(n1, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.NoError(t, err)

	key1, err := kr1.GetPubKey()
	require.NoError(t, err)
	require.NotNil(t, key1)
	require.Equal(t, key1, pub1)

	s12, pub1, err := kb.Sign(n1, d2, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	require.Equal(t, key1, pub1)

	s21, pub2, err := kb.Sign(n2, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)

	key2, err := kr2.GetPubKey()
	require.NoError(t, err)
	require.NotNil(t, key2)
	require.Equal(t, key2, pub2)

	s22, pub2, err := kb.Sign(n2, d2, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	require.Equal(t, key2, pub2)

	// let's try to validate and make sure it only works when everything is proper
	cases := []struct {
		key   types.PubKey
		data  []byte
		sig   []byte
		valid bool
	}{
		// proper matches
		{key1, d1, s11, true},
		// change data, pubkey, or signature leads to fail
		{key1, d2, s11, false},
		{key2, d1, s11, false},
		{key1, d1, s21, false},
		// make sure other successes
		{key1, d2, s12, true},
		{key2, d1, s21, true},
		{key2, d2, s22, true},
	}

	for i, tc := range cases {
		valid := tc.key.VerifySignature(tc.data, tc.sig)
		require.Equal(t, tc.valid, valid, "%d", i)
	}

	// Now try to sign data with a secret-less key
	// Import a public key
	armor, err := kb.ExportPubKeyArmor(n2)
	require.NoError(t, err)
	require.NoError(t, kb.Delete(n2))

	require.NoError(t, kb.ImportPubKey(n3, armor))
	i3, err := kb.Key(n3)
	require.NoError(t, err)
	require.Equal(t, i3.Name, n3)

	_, _, err = kb.Sign(n3, d3, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Error(t, err)
	require.Equal(t, "cannot sign with offline keys", err.Error())
}

func TestExportPrivKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name              string
		uid               string
		backend           string
		encryptPassphrase string
		createKey         func(keystore2 Keyring) (*Record, string, error)
		expectedErr       error
	}{
		{
			name:              "correct export",
			uid:               "correctTest",
			backend:           BackendTest,
			encryptPassphrase: "myPassphrase",
			createKey: func(keystore Keyring) (*Record, string, error) {
				return keystore.NewMnemonic("correctTest", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase,
					hd.Secp256k1)
			},
			expectedErr: nil,
		},
		{
			name:              "correct in memory export",
			uid:               "inMemory",
			backend:           BackendMemory,
			encryptPassphrase: "myPassphrase",
			createKey: func(keystore Keyring) (*Record, string, error) {
				return keystore.NewMnemonic("inMemory", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase,
					hd.Secp256k1)
			},
			expectedErr: nil,
		},
		{
			name:              "key is not created",
			uid:               "noKeyTest",
			backend:           BackendTest,
			encryptPassphrase: "myPassphrase",
			createKey: func(keystore Keyring) (*Record, string, error) {
				return nil, "", nil
			},
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("testExport", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			_, _, err = tt.createKey(kb)
			require.NoError(t, err)
			_, err = kb.ExportPrivKeyArmor(tt.uid, tt.encryptPassphrase)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestImportPrivKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name              string
		uid               string
		backend           string
		encryptPassphrase string
		armor             string
		expectedErr       error
	}{
		{
			name:              "correct import",
			uid:               "testOne",
			backend:           BackendTest,
			encryptPassphrase: "this passphrase has been used for all test vectors",
			armor:             "-----BEGIN TENDERMINT PRIVATE KEY-----\nkdf: bcrypt\nsalt: 6BC5D5187F9DF241E1A1243EECFF9C17\ntype: secp256k1\n\nGDPpPfrSVZloiwufbal19fmd75QeiqwToZ949SwmnxxM03qL75xXVf3tTD/BrF4l\nFs14HuhwntDBM2xgZvymTBk2edHlEI20Phv6oC0=\n=/zZh\n-----END TENDERMINT PRIVATE KEY-----",
			expectedErr:       nil,
		},
		{
			name:              "correct import",
			uid:               "inMemory",
			backend:           BackendMemory,
			encryptPassphrase: "this passphrase has been used for all test vectors",
			armor:             "-----BEGIN TENDERMINT PRIVATE KEY-----\nkdf: bcrypt\nsalt: 6BC5D5187F9DF241E1A1243EECFF9C17\ntype: secp256k1\n\nGDPpPfrSVZloiwufbal19fmd75QeiqwToZ949SwmnxxM03qL75xXVf3tTD/BrF4l\nFs14HuhwntDBM2xgZvymTBk2edHlEI20Phv6oC0=\n=/zZh\n-----END TENDERMINT PRIVATE KEY-----",
			expectedErr:       nil,
		},
		{
			name:              "wrong armor",
			uid:               "testWrongArmor",
			backend:           BackendTest,
			encryptPassphrase: "this passphrase has been used for all test vectors",
			armor:             "-----BEGIN TENDERMINT PRIVATE KEY-----\nkdf: bcrypt\nsalt: 7BC5D5187F9DF241E1A1243EECFF9C17\ntype: secp256k1\n\nGDPpPfrSVZloiwufbal19fmd75QeiqwToZ949SwmnxxM03qL75xXVf3tTD/BrF4l\nFs14HuhwntDBM2xgZvymTBk2edHlEI20Phv6oC0=\n=/zZh\n-----END TENDERMINT PRIVATE KEY-----",
			expectedErr:       sdkerrors.ErrWrongPassword,
		},
		{
			name:              "incorrect passphrase",
			uid:               "testIncorrectPassphrase",
			backend:           BackendTest,
			encryptPassphrase: "wrong passphrase",
			armor:             "-----BEGIN TENDERMINT PRIVATE KEY-----\nkdf: bcrypt\nsalt: 6BC5D5187F9DF241E1A1243EECFF9C17\ntype: secp256k1\n\nGDPpPfrSVZloiwufbal19fmd75QeiqwToZ949SwmnxxM03qL75xXVf3tTD/BrF4l\nFs14HuhwntDBM2xgZvymTBk2edHlEI20Phv6oC0=\n=/zZh\n-----END TENDERMINT PRIVATE KEY-----",
			expectedErr:       sdkerrors.ErrWrongPassword,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("TestExport", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			err = kb.ImportPrivKey(tt.uid, tt.armor, tt.encryptPassphrase)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestImportPrivKeyHex(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		uid         string
		backend     string
		hexKey      string
		algo        string
		expectedErr error
	}{
		{
			name:        "correct import",
			uid:         "hexImport",
			backend:     BackendTest,
			hexKey:      "0xa3e57952e835ed30eea86a2993ac2a61c03e74f2085b3635bd94aa4d7ae0cfdf",
			algo:        "secp256k1",
			expectedErr: nil,
		},
		{
			name:        "correct import without prefix",
			uid:         "hexImport",
			backend:     BackendTest,
			hexKey:      "a3e57952e835ed30eea86a2993ac2a61c03e74f2085b3635bd94aa4d7ae0cfdf",
			algo:        "secp256k1",
			expectedErr: nil,
		},
		{
			name:        "wrong hex length",
			uid:         "hexImport",
			backend:     BackendTest,
			hexKey:      "0xae57952e835ed30eea86a2993ac2a61c03e74f2085b3635bd94aa4d7ae0cfdf",
			algo:        "secp256k1",
			expectedErr: hex.ErrLength,
		},
		{
			name:        "unsupported algo",
			uid:         "hexImport",
			backend:     BackendTest,
			hexKey:      "0xa3e57952e835ed30eea86a2993ac2a61c03e74f2085b3635bd94aa4d7ae0cfdf",
			algo:        "notSupportedAlgo",
			expectedErr: ErrUnsupportedSigningAlgo,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("TestExport", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			err = kb.ImportPrivKeyHex(tt.uid, tt.hexKey, tt.algo)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestExportImportPrivKeyArmor(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name              string
		uid               string
		backend           string
		userInput         io.Reader
		encryptPassphrase string
		importUID         string
		importPassphrase  string
	}{
		{
			name:              "export import",
			uid:               "testOne",
			backend:           BackendTest,
			userInput:         nil,
			encryptPassphrase: "apassphrase",
			importUID:         "importedKey",
			importPassphrase:  "apassphrase",
		},
		{
			name:              "memory export import",
			uid:               "inMemory",
			backend:           BackendMemory,
			userInput:         nil,
			encryptPassphrase: "apassphrase",
			importUID:         "importedKey",
			importPassphrase:  "apassphrase",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("TestExport", tt.backend, t.TempDir(), tt.userInput, cdc)
			require.NoError(t, err)
			k, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			require.NotNil(t, k)
			require.Equal(t, k.Name, tt.uid)

			record, err := kb.Key(tt.uid)
			require.NoError(t, err)
			require.Equal(t, record.Name, tt.uid)
			key, err := k.GetPubKey()
			require.NoError(t, err)

			armor, err := kb.ExportPrivKeyArmor(tt.uid, tt.encryptPassphrase)
			require.NoError(t, err)

			// Import while key has not been deleted
			err = kb.ImportPrivKey(tt.uid, armor, tt.importPassphrase)
			require.Error(t, err)
			require.True(t, errors.Is(err, ErrOverwriteKey))

			err = kb.Delete(tt.uid)
			require.NoError(t, err)

			err = kb.ImportPrivKey(tt.importUID, armor, tt.importPassphrase)
			require.NoError(t, err)

			importedRecord, err := kb.Key(tt.importUID)
			require.NoError(t, err)
			require.Equal(t, importedRecord.Name, tt.importUID)
			importedKey, err := importedRecord.GetPubKey()
			require.NoError(t, err)

			require.Equal(t, key.Address(), importedKey.Address())

			addr, err := record.GetAddress()
			require.NoError(t, err)
			importedAddr, err := importedRecord.GetAddress()
			require.NoError(t, err)
			require.True(t, addr.Equals(importedAddr))
		})
	}
}

func TestImportExportPrivKeyByAddress(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name             string
		uid              string
		backend          string
		passphrase       string
		importPassphrase string
		expectedErr      error
	}{
		{
			name:             "correct import export",
			uid:              "okTest",
			backend:          BackendTest,
			passphrase:       "exportKey",
			importPassphrase: "exportKey",
			expectedErr:      nil,
		},
		{
			name:             "correct in memory import export",
			uid:              "inMemory",
			backend:          BackendMemory,
			passphrase:       "exportKey",
			importPassphrase: "exportKey",
			expectedErr:      nil,
		},
		{
			name:             "wrong passphrase import",
			uid:              "incorrectPass",
			backend:          BackendTest,
			passphrase:       "exportKey",
			importPassphrase: "incorrectPassphrase",
			expectedErr:      sdkerrors.ErrWrongPassword,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			addr, err := mnemonic.GetAddress()
			require.NoError(t, err)
			armor, err := kr.ExportPrivKeyArmorByAddress(addr, tt.passphrase)
			require.NoError(t, err)

			// Should fail importing private key on existing key.
			err = kr.ImportPrivKey(tt.uid, armor, tt.passphrase)
			require.True(t, errors.Is(err, ErrOverwriteKey))

			err = kr.Delete(tt.uid)
			require.NoError(t, err)

			err = kr.ImportPrivKey(tt.uid, armor, tt.importPassphrase)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestExportPubkey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		uid         string
		backend     string
		exportUID   string
		getPubkey   func(r *Record) (types.PubKey, error)
		codec       codec.Codec
		expectedErr error
	}{
		{
			name:      "correct export",
			uid:       "correctExport",
			backend:   BackendTest,
			exportUID: "correctExport",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: nil,
		},
		{
			name:      "wrong uid at export",
			uid:       "wrongUID",
			backend:   BackendTest,
			exportUID: "notAValidUID",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:      "previous space on export uid",
			uid:       "prefixSpace",
			backend:   BackendTest,
			exportUID: " prefixSpace",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:      "export uid with suffix space",
			uid:       "suffixSpace",
			backend:   BackendTest,
			exportUID: "suffixSpace ",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:      "correct in memory export",
			uid:       "inMemory",
			backend:   BackendMemory,
			exportUID: "inMemory",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: nil,
		},
		{
			name:      "in memory wrong uid at export",
			uid:       "wrongUid",
			backend:   BackendMemory,
			exportUID: "notAValidUid",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:      "in memory previous space on export uid",
			uid:       "prefixSpace",
			backend:   BackendMemory,
			exportUID: " prefixSpace",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:      "in memory export uid with suffix space",
			uid:       "suffixSpace",
			backend:   BackendMemory,
			exportUID: "suffixSpace ",
			getPubkey: func(r *Record) (types.PubKey, error) {
				return r.GetPubKey()
			},
			codec:       cdc,
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybase", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			k, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			require.NotNil(t, k)
			_, err = tt.getPubkey(k)
			require.NoError(t, err)
			_, err = kb.ExportPubKeyArmor(tt.exportUID)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestImportPubKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		uid         string
		backend     string
		armor       string
		expectedErr string
	}{
		{
			name:        "correct import",
			uid:         "correctTest",
			backend:     BackendTest,
			armor:       "-----BEGIN TENDERMINT PUBLIC KEY-----\nversion: 0.0.1\ntype: secp256k1\n\nCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQOlcgxiZM4cR0LA\nwum483+L6zRnXC6zEKtQ4FEa6z0VrA==\n=CqBG\n-----END TENDERMINT PUBLIC KEY-----",
			expectedErr: "",
		},
		{
			name:        "modified armor",
			uid:         "modified",
			backend:     BackendTest,
			armor:       "-----BEGIN TENDERMINT PUBLIC KEY-----\nversion: 0.0.1\ntype: secp256k1\n\nCh8vY29zbW8zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQOlcgxiZM4cR0LA\nwum483+L6zRnXC6zEKtQ4FEa6z0VrA==\n=CqBG\n-----END TENDERMINT PUBLIC KEY-----",
			expectedErr: "couldn't unarmor bytes: openpgp: invalid data: armor invalid",
		},
		{
			name:        "empty armor",
			uid:         "empty",
			backend:     BackendTest,
			armor:       "",
			expectedErr: "couldn't unarmor bytes: EOF",
		},
		{
			name:        "correct in memory import",
			uid:         "inMemory",
			backend:     BackendMemory,
			armor:       "-----BEGIN TENDERMINT PUBLIC KEY-----\nversion: 0.0.1\ntype: secp256k1\n\nCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQOlcgxiZM4cR0LA\nwum483+L6zRnXC6zEKtQ4FEa6z0VrA==\n=CqBG\n-----END TENDERMINT PUBLIC KEY-----",
			expectedErr: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			err = kb.ImportPubKey(tt.uid, tt.armor)
			if tt.expectedErr == "" {
				require.NoError(t, err)
			} else {
				require.ErrorContains(t, err, tt.expectedErr)
			}
		})
	}
}

func TestExportImportPubKeyKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name      string
		uid       string
		backend   string
		importUID string
	}{
		{
			name:      "complete export import",
			uid:       "testOne",
			backend:   BackendTest,
			importUID: "importedKey",
		},
		{
			name:      "in memory export import",
			uid:       "inMemory",
			backend:   BackendMemory,
			importUID: "importedKey",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			k, _, err := kb.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.Nil(t, err)
			require.NotNil(t, k)
			require.Equal(t, k.Name, tt.uid)

			key, err := k.GetPubKey()
			require.NoError(t, err)

			record, err := kb.Key(tt.uid)
			require.NoError(t, err)
			require.Equal(t, record.Name, tt.uid)

			pk, err := record.GetPubKey()
			require.NoError(t, err)
			require.Equal(t, key.Address(), pk.Address())

			// Export the public key only
			armor, err := kb.ExportPubKeyArmor(tt.uid)
			require.NoError(t, err)
			err = kb.Delete(tt.uid)
			require.NoError(t, err)

			// Import it under a different name
			err = kb.ImportPubKey(tt.importUID, armor)
			require.NoError(t, err)

			// Ensure consistency
			record2, err := kb.Key(tt.importUID)
			require.NoError(t, err)
			key2, err := record2.GetPubKey()
			require.NoError(t, err)

			// Compare the public keys
			require.True(t, key.Equals(key2))

			// Ensure keys cannot be overwritten
			err = kb.ImportPubKey(tt.importUID, armor)
			require.NotNil(t, err)
		})
	}
}

func TestImportExportPubKeyByAddress(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
	}{
		{
			name:    "import export",
			backend: BackendTest,
			uid:     "okTest",
		},
		{
			name:    "in memory import export",
			backend: BackendMemory,
			uid:     "okTest",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			addr, err := mnemonic.GetAddress()
			require.NoError(t, err)
			armor, err := kr.ExportPubKeyArmorByAddress(addr)
			require.NoError(t, err)

			// Should fail importing private key on existing key.
			err = kr.ImportPubKey(tt.uid, armor)
			require.True(t, errors.Is(err, ErrOverwriteKey))

			err = kr.Delete(tt.uid)
			require.NoError(t, err)

			err = kr.ImportPubKey(tt.uid, armor)
			require.NoError(t, err)
		})
	}
}

func TestAltKeyring_UnsafeExportPrivKeyHex(t *testing.T) {
	cdc := getCodec()
	kr, err := New(t.Name(), BackendTest, t.TempDir(), nil, cdc)
	require.NoError(t, err)

	uid := theID

	_, _, err = kr.NewMnemonic(uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)

	privKey, err := kr.(keystore).ExportPrivateKeyObject(uid)

	require.NoError(t, err)
	require.Equal(t, 64, len(hex.EncodeToString(privKey.Bytes())))

	// test error on non existing key
	_, err = kr.(keystore).ExportPrivateKeyObject("non-existing")
	require.Error(t, err)
}

func TestNewAccount(t *testing.T) {
	cdc := getCodec()

	tests := []struct {
		name             string
		backend          string
		uid              string
		bip39Passphrease string
		hdpath           string
		algo             SignatureAlgo
		mnemonic         string
		expectedErr      error
	}{
		{
			name:             "correct mnemonic",
			uid:              "correctTest",
			backend:          BackendTest,
			hdpath:           sdk.FullFundraiserPath,
			bip39Passphrease: "",
			algo:             hd.Secp256k1,
			mnemonic:         "aunt imitate maximum student guard unhappy guard rotate marine panel negative merit record priority zoo voice mixture boost describe fruit often occur expect teach",
			expectedErr:      nil,
		},
		{
			name:             "correct in memory mnemonic",
			uid:              "inMemory",
			backend:          BackendMemory,
			hdpath:           sdk.FullFundraiserPath,
			bip39Passphrease: "",
			algo:             hd.Secp256k1,
			mnemonic:         "aunt imitate maximum student guard unhappy guard rotate marine panel negative merit record priority zoo voice mixture boost describe fruit often occur expect teach",
			expectedErr:      nil,
		},
		{
			name:             "unsupported Algo",
			uid:              "correctTest",
			backend:          BackendTest,
			hdpath:           sdk.FullFundraiserPath,
			bip39Passphrease: "",
			algo:             notSupportedAlgo{},
			mnemonic:         "aunt imitate maximum student guard unhappy guard rotate marine panel negative merit record priority zoo voice mixture boost describe fruit often occur expect teach",
			expectedErr:      ErrUnsupportedSigningAlgo,
		},
		{
			name:             "wrong mnemonic",
			uid:              "wrongMnemonic",
			backend:          BackendTest,
			hdpath:           sdk.FullFundraiserPath,
			bip39Passphrease: "",
			algo:             hd.Secp256k1,
			mnemonic:         "fresh enact fresh ski large bicycle marine abandon motor end pact mixture annual elite bind fan write warrior adapt common manual cool happy dutch",
			expectedErr:      errors.New("invalid byte at position"),
		},
		{
			name:             "in memory invalid mnemonic",
			uid:              "memoryInvalid",
			backend:          BackendMemory,
			hdpath:           sdk.FullFundraiserPath,
			bip39Passphrease: "",
			algo:             hd.Secp256k1,
			mnemonic:         "malarkey pair crucial catch public canyon evil outer stage ten gym tornado",
			expectedErr:      errors.New("invalid mnemonic"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kb, err := New("keybasename", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			k1, err := kb.NewAccount(tt.uid, tt.mnemonic, DefaultBIP39Passphrase, tt.hdpath, tt.algo)
			if tt.expectedErr == nil {
				require.NoError(t, err)
				require.Equal(t, tt.uid, k1.Name)
			} else {
				require.Error(t, err)
				require.ErrorContains(t, err, err.Error())
			}
		})
	}
}

func TestInMemoryWithKeyring(t *testing.T) {
	priv := types.PrivKey(secp256k1.GenPrivKey())
	pub := priv.PubKey()

	cdc := getCodec()
	_, err := NewLocalRecord("test record", priv, pub)
	require.NoError(t, err)

	multi := multisig.NewLegacyAminoPubKey(
		1, []types.PubKey{
			pub,
		},
	)

	appName := "test-app"

	legacyMultiInfo, err := NewLegacyMultiInfo(appName, multi)
	require.NoError(t, err)
	serializedLegacyMultiInfo := MarshalInfo(legacyMultiInfo)

	kb := NewInMemoryWithKeyring(keyring.NewArrayKeyring([]keyring.Item{
		{
			Key:         appName + ".info",
			Data:        serializedLegacyMultiInfo,
			Description: "test description",
		},
	}), cdc)

	t.Run("key exists", func(t *testing.T) {
		_, err := kb.Key(appName)
		require.NoError(t, err)
	})

	t.Run("key deleted", func(t *testing.T) {
		err := kb.Delete(appName)
		require.NoError(t, err)

		t.Run("key is gone", func(t *testing.T) {
			_, err := kb.Key(appName)
			require.Error(t, err)
		})
	})
}

func TestInMemoryCreateMultisig(t *testing.T) {
	cdc := getCodec()
	kb, err := New("keybasename", "memory", "", nil, cdc)
	require.NoError(t, err)
	multi := multisig.NewLegacyAminoPubKey(
		1, []types.PubKey{
			secp256k1.GenPrivKey().PubKey(),
		},
	)
	_, err = kb.SaveMultisig("multi", multi)
	require.NoError(t, err)
}

// TestInMemorySignVerify does some detailed checks on how we sign and validate
// signatures
func TestInMemorySignVerify(t *testing.T) {
	cdc := getCodec()
	cstore := NewInMemory(cdc)
	algo := hd.Secp256k1

	n1, n2, n3 := "some dude", "a dudette", "dude-ish"

	// create two users and get their info
	kr1, _, err := cstore.NewMnemonic(n1, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, algo)
	require.Nil(t, err)

	kr2, _, err := cstore.NewMnemonic(n2, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, algo)
	require.Nil(t, err)

	// let's try to sign some messages
	d1 := []byte("my first message")
	d2 := []byte("some other important info!")
	d3 := []byte("feels like I forgot something...")

	// try signing both data with both ..
	s11, pub1, err := cstore.Sign(n1, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	key1, err := kr1.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, key1, pub1)

	s12, pub1, err := cstore.Sign(n1, d2, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	require.Equal(t, key1, pub1)

	s21, pub2, err := cstore.Sign(n2, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	key2, err := kr2.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, key2, pub2)

	s22, pub2, err := cstore.Sign(n2, d2, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Nil(t, err)
	require.Equal(t, key2, pub2)

	// let's try to validate and make sure it only works when everything is proper
	cases := []struct {
		key   types.PubKey
		data  []byte
		sig   []byte
		valid bool
	}{
		// proper matches
		{key1, d1, s11, true},
		// change data, pubkey, or signature leads to fail
		{key1, d2, s11, false},
		{key2, d1, s11, false},
		{key1, d1, s21, false},
		// make sure other successes
		{key1, d2, s12, true},
		{key2, d1, s21, true},
		{key2, d2, s22, true},
	}

	for i, tc := range cases {
		valid := tc.key.VerifySignature(tc.data, tc.sig)
		require.Equal(t, tc.valid, valid, "%d", i)
	}

	// Import a public key
	armor, err := cstore.ExportPubKeyArmor(n2)
	require.Nil(t, err)
	err = cstore.Delete(n2)
	require.NoError(t, err)
	err = cstore.ImportPubKey(n3, armor)
	require.NoError(t, err)
	i3, err := cstore.Key(n3)
	require.NoError(t, err)
	require.Equal(t, i3.Name, n3)

	// Now try to sign data with a secret-less key
	_, _, err = cstore.Sign(n3, d3, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Error(t, err)
	require.Equal(t, "cannot sign with offline keys", err.Error())
}

// TestInMemorySeedPhrase verifies restoring from a seed phrase
func TestInMemorySeedPhrase(t *testing.T) {
	// make the storage with reasonable defaults
	cdc := getCodec()
	tests := []struct {
		name      string
		uid       string
		importUID string
	}{
		{
			name:      "correct in memory seed",
			uid:       "okTest",
			importUID: "imported",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cstore := NewInMemory(cdc)

			// make sure key works with initial password
			k, mnemonic, err := cstore.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.Nil(t, err, "%+v", err)
			require.Equal(t, tt.uid, k.Name)
			require.NotEmpty(t, mnemonic)

			// now, let us delete this key
			err = cstore.Delete(tt.uid)
			require.Nil(t, err, "%+v", err)
			_, err = cstore.Key(tt.uid)
			require.NotNil(t, err)

			// let us re-create it from the mnemonic-phrase
			hdPath := hd.NewFundraiserParams(0, sdk.CoinType, 0).String()
			k1, err := cstore.NewAccount(tt.importUID, mnemonic, DefaultBIP39Passphrase, hdPath, hd.Secp256k1)
			require.NoError(t, err)
			require.Equal(t, tt.importUID, k1.Name)
			key, err := k.GetPubKey()
			require.NoError(t, err)
			key1, err := k1.GetPubKey()
			require.NoError(t, err)
			require.Equal(t, key.Address(), key1.Address())
			require.Equal(t, key, key1)
		})
	}
}

func TestKeyChain_ShouldFailWhenAddingSameGeneratedAccount(t *testing.T) {
	cdc := getCodec()
	kr, err := New(t.Name(), BackendTest, t.TempDir(), nil, cdc)
	require.NoError(t, err)

	// Given we create a mnemonic
	_, seed, err := kr.NewMnemonic("test", English, "", DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)

	require.NoError(t, kr.Delete("test"))

	path := hd.CreateHDPath(118, 0, 0).String()
	_, err = kr.NewAccount("test1", seed, "", path, hd.Secp256k1)
	require.NoError(t, err)

	// Creating another account with different uid but same seed should fail due to have same pub address
	_, err = kr.NewAccount("test2", seed, "", path, hd.Secp256k1)
	require.Error(t, err)
}

func ExampleNew() {
	// Select the encryption and storage for your cryptostore
	cdc := getCodec()
	cstore := NewInMemory(cdc)

	sec := hd.Secp256k1

	// Add keys and see they return in alphabetical order
	bob, _, err := cstore.NewMnemonic("Bob", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, sec)
	if err != nil {
		// this should never happen
		fmt.Println(err)
	} else {
		// return info here just like in List
		fmt.Println(bob.Name)
	}
	_, _, _ = cstore.NewMnemonic("Alice", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, sec)
	_, _, _ = cstore.NewMnemonic("Carl", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, sec)
	records, _ := cstore.List()
	for _, k := range records {
		fmt.Println(k.Name)
	}

	// We need to use passphrase to generate a signature
	tx := []byte("deadbeef")
	sig, pub, err := cstore.Sign("Bob", tx, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	if err != nil {
		fmt.Println("don't accept real passphrase")
	}

	// and we can validate the signature with publicly available info
	bRecord, _ := cstore.Key("Bob")
	key, _ := bRecord.GetPubKey()
	bobKey, _ := bob.GetPubKey()
	if !key.Equals(bobKey) {
		fmt.Println("Get and Create return different keys")
	}

	if pub.Equals(key) {
		fmt.Println("signed by Bob")
	}
	if !pub.VerifySignature(tx, sig) {
		fmt.Println("invalid signature")
	}

	// Output:
	// Bob
	// Alice
	// Bob
	// Carl
	// signed by Bob
}

func TestAltKeyring_List(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uids    []string
	}{
		{
			name:    "correct list",
			backend: BackendTest,
			uids:    []string{"Bkey", "Rkey", "Zkey"},
		},
		{
			name:    "empty list",
			backend: BackendTest,
			uids:    nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New("listKeys", tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			list, err := kr.List()
			require.NoError(t, err)
			require.Empty(t, list)

			for _, uid := range tt.uids {
				_, _, err = kr.NewMnemonic(uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
				require.NoError(t, err)
			}
			list, err = kr.List()
			require.NoError(t, err)
			require.Len(t, list, len(tt.uids))

			for i := range tt.uids {
				require.Equal(t, tt.uids[i], list[i].Name)
			}
		})
	}
}

func TestAltKeyring_Get(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		backend     string
		uid         string
		uidToFind   string
		expectedErr error
	}{
		{
			name:        "correct get",
			backend:     BackendTest,
			uid:         "okTest",
			uidToFind:   "okTest",
			expectedErr: nil,
		},
		{
			name:        "not found key",
			backend:     BackendTest,
			uid:         "notFoundUid",
			uidToFind:   "notFound",
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:        "in memory correct get",
			backend:     BackendMemory,
			uid:         "okTest",
			uidToFind:   "okTest",
			expectedErr: nil,
		},
		{
			name:        "in memory not found key",
			backend:     BackendMemory,
			uid:         "notFoundUid",
			uidToFind:   "notFound",
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.name, tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			key, err := kr.Key(tt.uidToFind)
			if tt.expectedErr == nil {
				require.NoError(t, err)
				requireEqualRenamedKey(t, mnemonic, key, true)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestAltKeyring_KeyByAddress(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		backend     string
		uid         string
		getAddres   func(*Record) (sdk.AccAddress, error)
		expectedErr error
	}{
		{
			name:    "correct get",
			backend: BackendTest,
			uid:     "okTest",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return k.GetAddress()
			},
			expectedErr: nil,
		},
		{
			name:    "not found key",
			backend: BackendTest,
			uid:     "notFoundUid",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return nil, nil
			},
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.name, tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			addr, err := tt.getAddres(mnemonic)
			require.NoError(t, err)

			key, err := kr.KeyByAddress(addr)
			if tt.expectedErr == nil {
				require.NoError(t, err)
				requireEqualRenamedKey(t, mnemonic, key, true)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

func TestAltKeyring_Delete(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		backend     string
		uid         string
		uidToDelete string
		expectedErr error
	}{
		{
			name:        "correct delete",
			backend:     BackendTest,
			uid:         "deleteKey",
			uidToDelete: "deleteKey",
			expectedErr: nil,
		},
		{
			name:        "not found delete",
			backend:     BackendTest,
			uid:         "deleteKey",
			uidToDelete: "notFound",
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:        "in memory correct delete",
			backend:     BackendMemory,
			uid:         "inMemoryDeleteKey",
			uidToDelete: "inMemoryDeleteKey",
			expectedErr: nil,
		},
		{
			name:        "in memory not found delete",
			backend:     BackendMemory,
			uid:         "inMemoryDeleteKey",
			uidToDelete: "notFound",
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			_, _, err = kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			list, err := kr.List()
			require.NoError(t, err)
			require.Len(t, list, 1)

			err = kr.Delete(tt.uidToDelete)
			list, listErr := kr.List()
			require.NoError(t, listErr)
			if tt.expectedErr == nil {
				require.NoError(t, err)
				require.Empty(t, list)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
				require.Len(t, list, 1)
			}
		})
	}
}

func TestAltKeyring_DeleteByAddress(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		backend     string
		uid         string
		getAddres   func(*Record) (sdk.AccAddress, error)
		expectedErr error
	}{
		{
			name:    "correct delete",
			backend: BackendTest,
			uid:     "okTest",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return k.GetAddress()
			},
			expectedErr: nil,
		},
		{
			name:    "not found",
			backend: BackendTest,
			uid:     "notFoundUid",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return nil, nil
			},
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
		{
			name:    "in memory correct delete",
			backend: BackendMemory,
			uid:     "inMemory",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return k.GetAddress()
			},
			expectedErr: nil,
		},
		{
			name:    "in memory not found",
			backend: BackendMemory,
			uid:     "inMemoryNotFoundUid",
			getAddres: func(k *Record) (sdk.AccAddress, error) {
				return nil, nil
			},
			expectedErr: sdkerrors.ErrKeyNotFound,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.name, tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)
			addr, err := tt.getAddres(mnemonic)
			require.NoError(t, err)

			err = kr.DeleteByAddress(addr)
			list, listErr := kr.List()
			require.NoError(t, listErr)
			if tt.expectedErr == nil {
				require.NoError(t, err)
				require.Empty(t, list)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
				require.Len(t, list, 1)
			}
		})
	}
}

// TODO: review
func TestAltKeyring_SaveOfflineKey(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
		pubKey  string
	}{
		{
			name:    "correct save",
			backend: BackendTest,
			uid:     "okSave",
			pubKey:  "cfd96f5e00069b64ddb8bfa433941400ab674db42436ae08bc9c74f3b5ade896",
		},
		{
			name:    "in memory correct save",
			backend: BackendMemory,
			uid:     "memorySave",
			pubKey:  "cfd96f5e00069b64ddb8bfa433941400ab674db42436ae08bc9c74f3b5ade896",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.name, tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			priv := ed25519.GenPrivKey()
			pub := priv.PubKey()
			pub.Bytes()
			k, err := kr.SaveOfflineKey(tt.uid, pub)
			require.NoError(t, err)
			pubKey, err := k.GetPubKey()
			require.NoError(t, err)
			require.Equal(t, pub, pubKey)
			require.Equal(t, tt.uid, k.Name)

			list, err := kr.List()
			require.NoError(t, err)
			require.Len(t, list, 1)
		})
	}
}

func TestNonConsistentKeyring_SavePubKey(t *testing.T) {
	cdc := getCodec()
	kr, err := New(t.Name(), BackendTest, t.TempDir(), nil, cdc)
	require.NoError(t, err)

	list, err := kr.List()
	require.NoError(t, err)
	require.Empty(t, list)

	key := someKey
	priv := ed25519.GenPrivKey()
	pub := priv.PubKey()

	_, err = kr.SaveOfflineKey(key, pub)
	require.NoError(t, err)

	// broken keyring state test
	unsafeKr, ok := kr.(keystore)
	require.True(t, ok)
	// we lost public key for some reason, but still have an address record
	require.NoError(t, unsafeKr.db.Remove(infoKey(key)))
	list, err = kr.List()
	require.NoError(t, err)
	require.Equal(t, 0, len(list))

	k, err := kr.SaveOfflineKey(key, pub)
	require.Nil(t, err)
	pubKey, err := k.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, pub, pubKey)
	require.Equal(t, key, k.Name)

	list, err = kr.List()
	require.NoError(t, err)
	require.Equal(t, 1, len(list))
}

func TestAltKeyring_SaveMultisig(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name      string
		uid       string
		backend   string
		mnemonics []string
	}{
		{
			name:    "correct multisig",
			uid:     "multi",
			backend: BackendTest,
			mnemonics: []string{
				"faint misery damage shoot wedding chat dress joy page stand gun business dance amount amused pond smart rate inner ill loud agree two evil",
				"window surprise chief blame huge umbrella pool home draw staff water brief modify depth whisper hawk floor come fury property pond cluster ethics super",
			},
		},
		{
			name:    "correct in memory multisig",
			uid:     "multiInMemory",
			backend: BackendMemory,
			mnemonics: []string{
				"faint misery damage shoot wedding chat dress joy page stand gun business dance amount amused pond smart rate inner ill loud agree two evil",
				"window surprise chief blame huge umbrella pool home draw staff water brief modify depth whisper hawk floor come fury property pond cluster ethics super",
			},
		},
		{
			name:      "one key multisig",
			uid:       "multi",
			backend:   BackendTest,
			mnemonics: []string{"faint misery damage shoot wedding chat dress joy page stand gun business dance amount amused pond smart rate inner ill loud agree two evil"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)
			pubKeys := make([]types.PubKey, len(tt.mnemonics))
			for i, mnemonic := range tt.mnemonics {
				r, err := kr.NewAccount(strconv.FormatInt(int64(i), 10), mnemonic, DefaultBIP39Passphrase, sdk.FullFundraiserPath, hd.Secp256k1)
				require.NoError(t, err)
				key, err := r.GetPubKey()
				require.NoError(t, err)
				pubKeys[i] = key
			}
			pub := multisig.NewLegacyAminoPubKey(len(tt.mnemonics), pubKeys)
			k, err := kr.SaveMultisig(tt.uid, pub)
			require.Nil(t, err)
			infoKey, err := k.GetPubKey()
			require.NoError(t, err)
			require.Equal(t, pub, infoKey)
			require.Equal(t, tt.uid, k.Name)

			list, err := kr.List()
			require.NoError(t, err)
			require.Len(t, list, len(tt.mnemonics)+1)
		})
	}
}

// TODO: add more tests
func TestAltKeyring_Sign(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
		msg     []byte
		mode    signing.SignMode
	}{
		{
			name:    "correct sign",
			backend: BackendTest,
			uid:     "signKey",
			msg:     []byte("some message"),
			mode:    signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			_, _, err = kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			sign, key, err := kr.Sign(tt.uid, tt.msg, tt.mode)
			require.NoError(t, err)

			require.True(t, key.VerifySignature(tt.msg, sign))
		})
	}
}

func TestAltKeyring_SignByAddress(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name    string
		backend string
		uid     string
		msg     []byte
		mode    signing.SignMode
	}{
		{
			name:    "correct sign by address",
			backend: BackendTest,
			uid:     "signKey",
			msg:     []byte("some message"),
			mode:    signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), tt.backend, t.TempDir(), nil, cdc)
			require.NoError(t, err)

			mnemonic, _, err := kr.NewMnemonic(tt.uid, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
			require.NoError(t, err)

			addr, err := mnemonic.GetAddress()
			require.NoError(t, err)
			sign, key, err := kr.SignByAddress(addr, tt.msg, tt.mode)
			require.NoError(t, err)

			require.True(t, key.VerifySignature(tt.msg, sign))
		})
	}
}

func TestAltKeyring_ConstructorSupportedAlgos(t *testing.T) {
	cdc := getCodec()
	tests := []struct {
		name        string
		algoOptions func(options *Options)
		expectedErr error
	}{
		{
			name: "add new algo",
			algoOptions: func(options *Options) {
				options.SupportedAlgos = SigningAlgoList{
					notSupportedAlgo{},
				}
			},
			expectedErr: nil,
		},
		{
			name:        "not supported algo",
			algoOptions: func(options *Options) {},
			expectedErr: ErrUnsupportedSigningAlgo,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(t.Name(), BackendTest, t.TempDir(), nil, cdc, tt.algoOptions)
			require.NoError(t, err)
			_, _, err = kr.NewMnemonic("test", English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, notSupportedAlgo{})
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}

// TODO: review it
func TestBackendConfigConstructors(t *testing.T) {
	backend := newKWalletBackendKeyringConfig("test", "", nil)
	require.Equal(t, []keyring.BackendType{keyring.KWalletBackend}, backend.AllowedBackends)
	require.Equal(t, "kdewallet", backend.ServiceName)
	require.Equal(t, "test", backend.KWalletAppID)

	backend = newPassBackendKeyringConfig("test", "directory", nil)
	require.Equal(t, []keyring.BackendType{keyring.PassBackend}, backend.AllowedBackends)
	require.Equal(t, "test", backend.ServiceName)
	require.Equal(t, "keyring-test", backend.PassPrefix)
}

func TestRenameKey(t *testing.T) {
	testCases := []struct {
		name string
		run  func(Keyring)
	}{
		{
			name: "rename a key",
			run: func(kr Keyring) {
				oldKeyUID, newKeyUID := "old", "new"
				oldKeyRecord := newKeyRecord(t, kr, oldKeyUID)
				err := kr.Rename(oldKeyUID, newKeyUID) // rename from "old" to "new"
				require.NoError(t, err)
				newRecord, err := kr.Key(newKeyUID) // new key should be in keyring
				require.NoError(t, err)
				requireEqualRenamedKey(t, newRecord, oldKeyRecord, false) // oldKeyRecord and newRecord should be the same except name
				_, err = kr.Key(oldKeyUID)                                // old key should be gone from keyring
				require.Error(t, err)
			},
		},
		{
			name: "can't rename a key that doesn't exist",
			run: func(kr Keyring) {
				err := kr.Rename("bogus", "bogus2")
				require.Error(t, err)
			},
		},
		{
			name: "can't rename a key to an already existing key name",
			run: func(kr Keyring) {
				key1, key2 := "existingKey", "existingKey2" // create 2 keys
				newKeyRecord(t, kr, key1)
				newKeyRecord(t, kr, key2)
				err := kr.Rename(key2, key1)
				require.True(t, errors.Is(err, ErrKeyAlreadyExists))
				assertKeysExist(t, kr, key1, key2) // keys should still exist after failed rename
			},
		},
		{
			name: "can't rename key to itself",
			run: func(kr Keyring) {
				keyName := "keyName"
				newKeyRecord(t, kr, keyName)
				err := kr.Rename(keyName, keyName)
				require.True(t, errors.Is(err, ErrKeyAlreadyExists))
				assertKeysExist(t, kr, keyName)
			},
		},
	}

	for _, tc := range testCases {

		kr := newKeyring(t, "testKeyring")
		t.Run(tc.name, func(t *testing.T) {
			tc.run(kr)
		})
	}
}

// TestChangeBcrypt tests the compatibility from upstream Bcrypt and our own
func TestChangeBcrypt(t *testing.T) {
	pw := []byte("somepasswword!")

	saltBytes := cmtcrypto.CRandBytes(16)
	cosmosHash, err := cosmosbcrypt.GenerateFromPassword(saltBytes, pw, 2)
	require.NoError(t, err)

	bcryptHash, err := bcrypt.GenerateFromPassword(pw, 2)
	require.NoError(t, err)

	// Check the new hash with the old bcrypt, vice-versa and with the same
	// bcrypt version just because.
	err = cosmosbcrypt.CompareHashAndPassword(bcryptHash, pw)
	require.NoError(t, err)

	err = cosmosbcrypt.CompareHashAndPassword(cosmosHash, pw)
	require.NoError(t, err)

	err = bcrypt.CompareHashAndPassword(cosmosHash, pw)
	require.NoError(t, err)

	err = bcrypt.CompareHashAndPassword(bcryptHash, pw)
	require.NoError(t, err)
}

func requireEqualRenamedKey(t *testing.T, key, mnemonic *Record, nameMatch bool) {
	t.Helper()
	if nameMatch {
		require.Equal(t, key.Name, mnemonic.Name)
	}
	keyAddr, err := key.GetAddress()
	require.NoError(t, err)
	mnemonicAddr, err := mnemonic.GetAddress()
	require.NoError(t, err)
	require.Equal(t, keyAddr, mnemonicAddr)

	key1, err := key.GetPubKey()
	require.NoError(t, err)
	key2, err := mnemonic.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, key1, key2)
	require.Equal(t, key.GetType(), mnemonic.GetType())
}

func newKeyring(t *testing.T, name string) Keyring {
	t.Helper()
	cdc := getCodec()
	kr, err := New(name, "test", t.TempDir(), nil, cdc)
	require.NoError(t, err)
	return kr
}

func newKeyRecord(t *testing.T, kr Keyring, name string) *Record {
	t.Helper()
	k, _, err := kr.NewMnemonic(name, English, sdk.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)
	return k
}

func assertKeysExist(t *testing.T, kr Keyring, names ...string) {
	t.Helper()
	for _, n := range names {
		_, err := kr.Key(n)
		require.NoError(t, err)
	}
}

func accAddr(k *Record) (sdk.AccAddress, error) { return k.GetAddress() }
