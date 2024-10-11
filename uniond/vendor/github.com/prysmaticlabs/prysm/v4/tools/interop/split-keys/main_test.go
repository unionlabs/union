package main

import (
	"bufio"
	"context"
	"encoding/hex"
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
)

const testMnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

var (
	testPrivKeys = [][]byte{
		hexDecodeOrDie("3ec45abb2792f1f287ab1434acfde9d7aac879eb74c45cf7b59d25f15ba7a650"),
		hexDecodeOrDie("3b6e255c01a33ccce39927196c7f96ee512e29b9aefcfe98132c2df2e2f04043"),
		hexDecodeOrDie("39f52a9ac0a2eb05b9633ff2e125bdb1313776f40418bb7b2d82b22ab4ca534a"),
		hexDecodeOrDie("04ef1acec58a2d7ee1c0f12d4083df2990f83a798a84ad4393b2ce6322d377d5"),
		hexDecodeOrDie("66e05a4dea6ee3292d35f281a542c4931070b064cbe0f4436461db29208426b7"),
	}
	testPubKeys = [][]byte{
		hexDecodeOrDie("b3e445d43871965d890a398f719348a1405ac72e35b92727cc570026f54471af7ea7b2040622a8fd0b5bfb2a209b5911"),
		hexDecodeOrDie("aeb399bf5648b0e9980c1731824c269631a41320c3d7f730c40587e1a37a5e1c8b5755fd90080a7b3fb90d3fd419c0a7"),
		hexDecodeOrDie("92f46b0dcc7db24f4946b5773b5525efa0bbb0810088588323d9de84f0e42f22df96cbb97065b49a2006c653ec8060f4"),
		hexDecodeOrDie("9948ea3862b8889636c3caeaa1b9877a12cffca9bf6a1ef2264fa7e69604d55c56b4f519062e6785d21d3c9593c2adcd"),
		hexDecodeOrDie("849b4bcd8670f81909baad27c4d9c8d9b956b19192f12af8fe57d30731fa11a55a97a1ab72bb1cfd76c1461dcaba714a"),
	}
)

func Test_generateKeysFromMnemonicList(t *testing.T) {
	rdr := strings.NewReader(testMnemonic)
	scanner := bufio.NewScanner(rdr)
	keysPerMnemonic := 5
	pubKeys, privKeys, err := generateKeysFromMnemonicList(scanner, keysPerMnemonic)
	require.NoError(t, err)
	require.Equal(t, keysPerMnemonic, len(pubKeys))
	require.Equal(t, keysPerMnemonic, len(privKeys))

	// Text the generated keys match some predetermined ones for the test.
	for i, key := range privKeys {
		require.DeepEqual(t, testPrivKeys[i], key)
	}
	for i, key := range pubKeys {
		require.DeepEqual(t, testPubKeys[i], key)
	}
}

func Test_spreadKeysAcrossImportedWallets(t *testing.T) {
	walletPassword := "Sr0ngPass0q0z929301"
	tmpDir := filepath.Join(t.TempDir(), "testwallets")

	// Spread 5 keys across 5 wallets, meaning there is 1
	// key per wallet stored on disk.
	numWallets := 5
	keysPerWallet := 1
	err := spreadKeysAcrossLocalWallets(
		testPubKeys,
		testPrivKeys,
		numWallets,
		keysPerWallet,
		tmpDir,
		walletPassword,
	)
	require.NoError(t, err)
	ctx := context.Background()
	for i := 0; i < numWallets; i++ {
		w, err := wallet.OpenWallet(ctx, &wallet.Config{
			WalletDir:      filepath.Join(tmpDir, fmt.Sprintf("wallet_%d", i)),
			KeymanagerKind: keymanager.Local,
			WalletPassword: walletPassword,
		})
		require.NoError(t, err)
		km, err := local.NewKeymanager(ctx, &local.SetupConfig{
			Wallet: w,
		})
		require.NoError(t, err)
		pubKeys, err := km.FetchValidatingPublicKeys(ctx)
		require.NoError(t, err)
		require.Equal(t, 1, len(pubKeys))
		require.DeepEqual(t, testPubKeys[i], pubKeys[0][:])
	}
}

func hexDecodeOrDie(str string) []byte {
	decoded, err := hex.DecodeString(str)
	if err != nil {
		panic(err)
	}
	return decoded
}
