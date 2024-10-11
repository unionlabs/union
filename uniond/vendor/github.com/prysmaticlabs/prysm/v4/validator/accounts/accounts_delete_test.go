package accounts

import (
	"bytes"
	"context"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestDelete(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	// import keys
	numAccounts := 5
	keystores := make([]*keymanager.Keystore, numAccounts)
	passwords := make([]string, numAccounts)
	for i := 0; i < numAccounts; i++ {
		keystores[i] = createRandomKeystore(t, password)
		passwords[i] = password
	}
	pubkey1, err := hexutil.Decode("0x" + keystores[0].Pubkey)
	require.NoError(t, err)
	p1, err := bls.PublicKeyFromBytes(pubkey1)
	require.NoError(t, err)
	pubkey2, err := hexutil.Decode("0x" + keystores[1].Pubkey)
	require.NoError(t, err)
	p2, err := bls.PublicKeyFromBytes(pubkey2)
	require.NoError(t, err)

	walletDir, passwordsDir, walletPasswordFile := setupWalletAndPasswordsDir(t)
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		keymanagerKind:     keymanager.Local,
		walletPasswordFile: walletPasswordFile,
	})

	var stdin bytes.Buffer
	stdin.Write([]byte("Y"))
	opts := []Option{
		WithWalletDir(walletDir),
		WithKeymanagerType(keymanager.Local),
		WithWalletPassword("Passwordz0320$"),
		WithCustomReader(&stdin),
		WithFilteredPubKeys([]bls.PublicKey{p1, p2}),
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
	acc.keymanager = km

	_, err = km.ImportKeystores(cliCtx.Context, keystores, passwords)
	require.NoError(t, err)

	// test delete
	err = acc.Delete(ctx)
	require.NoError(t, err)
	keys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, len(keys), 3)
	assert.LogsContain(t, hook, "Attempted to delete accounts.")
}
