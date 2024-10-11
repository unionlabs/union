package derived

import (
	"context"
	"fmt"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	mock "github.com/prysmaticlabs/prysm/v4/validator/accounts/testing"
	constant "github.com/prysmaticlabs/prysm/v4/validator/testing"
	"github.com/tyler-smith/go-bip39"
	util "github.com/wealdtech/go-eth2-util"
)

const (
	password = "secretPassw0rd$1999"
)

// We test that using a '25th word' mnemonic passphrase leads to different
// public keys derived than not specifying the passphrase.
func TestDerivedKeymanager_MnemnonicPassphrase_DifferentResults(t *testing.T) {
	ctx := context.Background()
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	km, err := NewKeymanager(ctx, &SetupConfig{
		Wallet:           wallet,
		ListenForChanges: false,
	})
	require.NoError(t, err)
	numAccounts := 5
	err = km.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, DefaultMnemonicLanguage, "mnemonicpass", numAccounts)
	require.NoError(t, err)
	without25thWord, err := km.FetchValidatingPublicKeys(ctx)
	require.NoError(t, err)
	wallet = &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	km, err = NewKeymanager(ctx, &SetupConfig{
		Wallet:           wallet,
		ListenForChanges: false,
	})
	require.NoError(t, err)
	// No mnemonic passphrase this time.
	err = km.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)
	with25thWord, err := km.FetchValidatingPublicKeys(ctx)
	require.NoError(t, err)
	for i, k := range with25thWord {
		without := without25thWord[i]
		assert.DeepNotEqual(t, k, without)
	}
}

func TestDerivedKeymanager_RecoverSeedRoundTrip(t *testing.T) {
	mnemonicEntropy := make([]byte, 32)
	n, err := rand.NewGenerator().Read(mnemonicEntropy)
	require.NoError(t, err)
	require.Equal(t, n, len(mnemonicEntropy))
	mnemonic, err := bip39.NewMnemonic(mnemonicEntropy)
	require.NoError(t, err)
	wanted := bip39.NewSeed(mnemonic, "")

	got, err := seedFromMnemonic(mnemonic, DefaultMnemonicLanguage, "" /* no passphrase */)
	require.NoError(t, err)
	// Ensure the derived seed matches.
	assert.DeepEqual(t, wanted, got)
}

func TestDerivedKeymanager_FetchValidatingPublicKeys(t *testing.T) {
	derivedSeed, err := seedFromMnemonic(constant.TestMnemonic, DefaultMnemonicLanguage, "")
	require.NoError(t, err)
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	ctx := context.Background()
	dr, err := NewKeymanager(ctx, &SetupConfig{
		Wallet:           wallet,
		ListenForChanges: false,
	})
	require.NoError(t, err)
	numAccounts := 5
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)

	// Fetch the public keys.
	publicKeys, err := dr.FetchValidatingPublicKeys(ctx)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(publicKeys))

	wantedPubKeys := make([][fieldparams.BLSPubkeyLength]byte, numAccounts)
	for i := 0; i < numAccounts; i++ {
		privKey, err := util.PrivateKeyFromSeedAndPath(derivedSeed, fmt.Sprintf(ValidatingKeyDerivationPathTemplate, i))
		require.NoError(t, err)
		var pubKey [fieldparams.BLSPubkeyLength]byte
		copy(pubKey[:], privKey.PublicKey().Marshal())
		wantedPubKeys[i] = pubKey
	}

	// FetchValidatingPublicKeys is also used in generating the output of account list
	// therefore the results must be in the same order as the order in which the accounts were derived
	for i, key := range wantedPubKeys {
		assert.Equal(t, key, publicKeys[i])
	}
}

func TestDerivedKeymanager_FetchValidatingPrivateKeys(t *testing.T) {
	derivedSeed, err := seedFromMnemonic(constant.TestMnemonic, DefaultMnemonicLanguage, "")
	require.NoError(t, err)
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	ctx := context.Background()
	dr, err := NewKeymanager(ctx, &SetupConfig{
		Wallet:           wallet,
		ListenForChanges: false,
	})
	require.NoError(t, err)
	numAccounts := 5
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)

	// Fetch the private keys.
	privateKeys, err := dr.FetchValidatingPrivateKeys(ctx)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(privateKeys))

	wantedPrivKeys := make([][32]byte, numAccounts)
	for i := 0; i < numAccounts; i++ {
		privKey, err := util.PrivateKeyFromSeedAndPath(derivedSeed, fmt.Sprintf(ValidatingKeyDerivationPathTemplate, i))
		require.NoError(t, err)
		var privKeyBytes [32]byte
		copy(privKeyBytes[:], privKey.Marshal())
		wantedPrivKeys[i] = privKeyBytes
	}

	// FetchValidatingPrivateKeys is also used in generating the output of account list
	// therefore the results must be in the same order as the order in which the accounts were derived
	for i, key := range wantedPrivKeys {
		assert.Equal(t, key, privateKeys[i])
	}
}

func TestDerivedKeymanager_Sign(t *testing.T) {
	wallet := &mock.Wallet{
		Files:            make(map[string]map[string][]byte),
		AccountPasswords: make(map[string]string),
		WalletPassword:   password,
	}
	ctx := context.Background()
	dr, err := NewKeymanager(ctx, &SetupConfig{
		Wallet:           wallet,
		ListenForChanges: false,
	})
	require.NoError(t, err)
	numAccounts := 5
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)

	pubKeys, err := dr.FetchValidatingPublicKeys(ctx)
	require.NoError(t, err)
	// We prepare naive data to sign.
	data := []byte("eth2data")
	signRequest := &validatorpb.SignRequest{
		PublicKey:   pubKeys[0][:],
		SigningRoot: data,
	}
	sig, err := dr.Sign(ctx, signRequest)
	require.NoError(t, err)
	pubKey, err := bls.PublicKeyFromBytes(pubKeys[0][:])
	require.NoError(t, err)
	wrongPubKey, err := bls.PublicKeyFromBytes(pubKeys[1][:])
	require.NoError(t, err)

	// Check if the signature verifies.
	assert.Equal(t, true, sig.Verify(pubKey, data))
	// Check if the bad signature fails.
	assert.Equal(t, false, sig.Verify(wrongPubKey, data))
}

func TestDerivedKeymanager_Sign_NoPublicKeySpecified(t *testing.T) {
	req := &validatorpb.SignRequest{
		PublicKey: nil,
	}
	dr := &Keymanager{}
	_, err := dr.Sign(context.Background(), req)
	assert.ErrorContains(t, "nil public key", err)
}

func TestDerivedKeymanager_Sign_NoPublicKeyInCache(t *testing.T) {
	req := &validatorpb.SignRequest{
		PublicKey: []byte("hello world"),
	}
	dr := &Keymanager{}
	_, err := dr.Sign(context.Background(), req)
	assert.ErrorContains(t, "no signing key found", err)
}
