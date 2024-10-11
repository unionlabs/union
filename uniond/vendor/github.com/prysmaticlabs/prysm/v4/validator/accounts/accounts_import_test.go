package accounts

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
)

func TestImportAccounts_NoPassword(t *testing.T) {
	local.ResetCaches()
	walletDir, passwordsDir, passwordFilePath := setupWalletAndPasswordsDir(t)
	keysDir := filepath.Join(t.TempDir(), "keysDir")
	require.NoError(t, os.MkdirAll(keysDir, os.ModePerm))

	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:           walletDir,
		passwordsDir:        passwordsDir,
		keysDir:             keysDir,
		keymanagerKind:      keymanager.Local,
		walletPasswordFile:  passwordFilePath,
		accountPasswordFile: passwordFilePath,
	})
	opts := []Option{
		WithWalletDir(walletDir),
		WithKeymanagerType(keymanager.Local),
		WithWalletPassword(password),
	}
	acc, err := NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(cliCtx.Context, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	importer, ok := km.(keymanager.Importer)
	require.Equal(t, true, ok)
	resp, err := ImportAccounts(context.Background(), &ImportAccountsConfig{
		Keystores:       []*keymanager.Keystore{{}},
		Importer:        importer,
		AccountPassword: "",
	})
	require.NoError(t, err)
	require.Equal(t, 1, len(resp))
	require.Equal(t, resp[0].Status, keymanager.StatusError)
}

func TestImport_SortByDerivationPath(t *testing.T) {
	local.ResetCaches()
	type test struct {
		name  string
		input []string
		want  []string
	}
	tests := []test{
		{
			name: "Basic sort",
			input: []string{
				"keystore_m_12381_3600_2_0_0.json",
				"keystore_m_12381_3600_1_0_0.json",
				"keystore_m_12381_3600_0_0_0.json",
			},
			want: []string{
				"keystore_m_12381_3600_0_0_0.json",
				"keystore_m_12381_3600_1_0_0.json",
				"keystore_m_12381_3600_2_0_0.json",
			},
		},
		{
			name: "Large digit accounts",
			input: []string{
				"keystore_m_12381_3600_30020330_0_0.json",
				"keystore_m_12381_3600_430490934_0_0.json",
				"keystore_m_12381_3600_0_0_0.json",
				"keystore_m_12381_3600_333_0_0.json",
			},
			want: []string{
				"keystore_m_12381_3600_0_0_0.json",
				"keystore_m_12381_3600_333_0_0.json",
				"keystore_m_12381_3600_30020330_0_0.json",
				"keystore_m_12381_3600_430490934_0_0.json",
			},
		},
		{
			name: "Some filenames with derivation path, others without",
			input: []string{
				"keystore_m_12381_3600_4_0_0.json",
				"keystore.json",
				"keystore-2309023.json",
				"keystore_m_12381_3600_1_0_0.json",
				"keystore_m_12381_3600_3_0_0.json",
			},
			want: []string{
				"keystore_m_12381_3600_1_0_0.json",
				"keystore_m_12381_3600_3_0_0.json",
				"keystore_m_12381_3600_4_0_0.json",
				"keystore.json",
				"keystore-2309023.json",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sort.Sort(byDerivationPath(tt.input))
			assert.DeepEqual(t, tt.want, tt.input)
		})
	}
}

func Test_importPrivateKeyAsAccount(t *testing.T) {
	walletDir, _, passwordFilePath := setupWalletAndPasswordsDir(t)
	privKeyDir := filepath.Join(t.TempDir(), "privKeys")
	require.NoError(t, os.MkdirAll(privKeyDir, os.ModePerm))
	privKeyFileName := filepath.Join(privKeyDir, "privatekey.txt")

	// We create a new private key and save it to a file on disk.
	privKey, err := bls.RandKey()
	require.NoError(t, err)
	privKeyHex := fmt.Sprintf("%x", privKey.Marshal())
	require.NoError(
		t,
		os.WriteFile(privKeyFileName, []byte(privKeyHex), params.BeaconIoConfig().ReadWritePermissions),
	)

	// We instantiate a new wallet from a cli context.
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:          walletDir,
		keymanagerKind:     keymanager.Local,
		walletPasswordFile: passwordFilePath,
		privateKeyFile:     privKeyFileName,
	})
	walletPass := "Passwordz0320$"
	opts := []Option{
		WithWalletDir(walletDir),
		WithKeymanagerType(keymanager.Local),
		WithWalletPassword(walletPass),
	}
	acc, err := NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)
	km, err := local.NewKeymanager(
		cliCtx.Context,
		&local.SetupConfig{
			Wallet:           w,
			ListenForChanges: false,
		},
	)
	require.NoError(t, err)
	assert.NoError(t, importPrivateKeyAsAccount(cliCtx.Context, w, km, privKeyFileName))

	// We re-instantiate the keymanager and check we now have 1 public key.
	km, err = local.NewKeymanager(
		cliCtx.Context,
		&local.SetupConfig{
			Wallet:           w,
			ListenForChanges: false,
		},
	)
	require.NoError(t, err)
	pubKeys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, 1, len(pubKeys))
	assert.DeepEqual(t, pubKeys[0], bytesutil.ToBytes48(privKey.PublicKey().Marshal()))
}

func Test_NameToDescriptionChangeIsOK(t *testing.T) {
	jsonString := `{"version":1, "name":"hmmm"}`
	type Obj struct {
		Version     uint   `json:"version"`
		Description string `json:"description"`
	}
	a := &Obj{}
	require.NoError(t, json.Unmarshal([]byte(jsonString), a))
	require.Equal(t, a.Description, "")
}

func Test_MarshalOmitsName(t *testing.T) {
	type Obj struct {
		Version     uint   `json:"version"`
		Description string `json:"description"`
		Name        string `json:"name,omitempty"`
	}
	a := &Obj{
		Version:     1,
		Description: "hmm",
	}

	bytes, err := json.Marshal(a)
	require.NoError(t, err)
	require.Equal(t, string(bytes), `{"version":1,"description":"hmm"}`)
}
