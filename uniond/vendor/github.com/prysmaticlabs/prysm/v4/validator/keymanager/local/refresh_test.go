package local

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/google/uuid"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	mock "github.com/prysmaticlabs/prysm/v4/validator/accounts/testing"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

func TestLocalKeymanager_reloadAccountsFromKeystore_MismatchedNumKeys(t *testing.T) {
	password := "Passw03rdz293**%#2"
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	dr := &Keymanager{
		wallet: wallet,
	}
	accountsStore := &accountStore{
		PrivateKeys: [][]byte{[]byte("hello")},
		PublicKeys:  [][]byte{[]byte("hi"), []byte("world")},
	}
	encodedStore, err := json.MarshalIndent(accountsStore, "", "\t")
	require.NoError(t, err)
	encryptor := keystorev4.New()
	cryptoFields, err := encryptor.Encrypt(encodedStore, dr.wallet.Password())
	require.NoError(t, err)
	id, err := uuid.NewRandom()
	require.NoError(t, err)
	keystore := &AccountsKeystoreRepresentation{
		Crypto:  cryptoFields,
		ID:      id.String(),
		Version: encryptor.Version(),
		Name:    encryptor.Name(),
	}
	err = dr.reloadAccountsFromKeystore(keystore)
	assert.ErrorContains(t, "do not match", err)
}

func TestLocalKeymanager_reloadAccountsFromKeystore(t *testing.T) {
	password := "Passw03rdz293**%#2"
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	dr := &Keymanager{
		wallet:              wallet,
		accountsChangedFeed: new(event.Feed),
	}

	numAccounts := 20
	privKeys := make([][]byte, numAccounts)
	pubKeys := make([][]byte, numAccounts)
	for i := 0; i < numAccounts; i++ {
		privKey, err := bls.RandKey()
		require.NoError(t, err)
		privKeys[i] = privKey.Marshal()
		pubKeys[i] = privKey.PublicKey().Marshal()
	}

	accountsStore, err := dr.CreateAccountsKeystore(context.Background(), privKeys, pubKeys)
	require.NoError(t, err)
	require.NoError(t, dr.reloadAccountsFromKeystore(accountsStore))

	// Check that the public keys were added to the public keys cache.
	for i, keyBytes := range pubKeys {
		require.Equal(t, bytesutil.ToBytes48(keyBytes), orderedPublicKeys[i])
	}

	// Check that the secret keys were added to the secret keys cache.
	lock.RLock()
	defer lock.RUnlock()
	for i, keyBytes := range privKeys {
		privKey, ok := secretKeysCache[bytesutil.ToBytes48(pubKeys[i])]
		require.Equal(t, true, ok)
		require.Equal(t, bytesutil.ToBytes48(keyBytes), bytesutil.ToBytes48(privKey.Marshal()))
	}

	// Check the key was added to the global accounts store.
	require.Equal(t, numAccounts, len(dr.accountsStore.PublicKeys))
	require.Equal(t, numAccounts, len(dr.accountsStore.PrivateKeys))
	assert.DeepEqual(t, dr.accountsStore.PublicKeys[0], pubKeys[0])
}
