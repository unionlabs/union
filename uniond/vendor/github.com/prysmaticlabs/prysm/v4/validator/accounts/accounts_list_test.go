package accounts

import (
	"flag"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/google/uuid"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	constant "github.com/prysmaticlabs/prysm/v4/validator/testing"
	"github.com/urfave/cli/v2"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

const (
	passwordFileName = "password.txt"
	password         = "OhWOWthisisatest42!$"
)

type testWalletConfig struct {
	exitAll                 bool
	skipDepositConfirm      bool
	keymanagerKind          keymanager.Kind
	numAccounts             int64
	grpcHeaders             string
	privateKeyFile          string
	accountPasswordFile     string
	walletPasswordFile      string
	backupPasswordFile      string
	backupPublicKeys        string
	voluntaryExitPublicKeys string
	deletePublicKeys        string
	keysDir                 string
	backupDir               string
	passwordsDir            string
	walletDir               string
}

func setupWalletCtx(
	tb testing.TB,
	cfg *testWalletConfig,
) *cli.Context {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.WalletDirFlag.Name, cfg.walletDir, "")
	set.String(flags.KeysDirFlag.Name, cfg.keysDir, "")
	set.String(flags.KeymanagerKindFlag.Name, cfg.keymanagerKind.String(), "")
	set.String(flags.DeletePublicKeysFlag.Name, cfg.deletePublicKeys, "")
	set.String(flags.VoluntaryExitPublicKeysFlag.Name, cfg.voluntaryExitPublicKeys, "")
	set.String(flags.BackupDirFlag.Name, cfg.backupDir, "")
	set.String(flags.BackupPasswordFile.Name, cfg.backupPasswordFile, "")
	set.String(flags.BackupPublicKeysFlag.Name, cfg.backupPublicKeys, "")
	set.String(flags.WalletPasswordFileFlag.Name, cfg.walletPasswordFile, "")
	set.String(flags.AccountPasswordFileFlag.Name, cfg.accountPasswordFile, "")
	set.Int64(flags.NumAccountsFlag.Name, cfg.numAccounts, "")
	set.Bool(flags.SkipDepositConfirmationFlag.Name, cfg.skipDepositConfirm, "")
	set.Bool(flags.SkipMnemonic25thWordCheckFlag.Name, true, "")
	set.Bool(flags.ExitAllFlag.Name, cfg.exitAll, "")
	set.String(flags.GrpcHeadersFlag.Name, cfg.grpcHeaders, "")

	if cfg.privateKeyFile != "" {
		set.String(flags.ImportPrivateKeyFileFlag.Name, cfg.privateKeyFile, "")
		assert.NoError(tb, set.Set(flags.ImportPrivateKeyFileFlag.Name, cfg.privateKeyFile))
	}
	assert.NoError(tb, set.Set(flags.WalletDirFlag.Name, cfg.walletDir))
	assert.NoError(tb, set.Set(flags.SkipMnemonic25thWordCheckFlag.Name, "true"))
	assert.NoError(tb, set.Set(flags.KeysDirFlag.Name, cfg.keysDir))
	assert.NoError(tb, set.Set(flags.KeymanagerKindFlag.Name, cfg.keymanagerKind.String()))
	assert.NoError(tb, set.Set(flags.DeletePublicKeysFlag.Name, cfg.deletePublicKeys))
	assert.NoError(tb, set.Set(flags.VoluntaryExitPublicKeysFlag.Name, cfg.voluntaryExitPublicKeys))
	assert.NoError(tb, set.Set(flags.BackupDirFlag.Name, cfg.backupDir))
	assert.NoError(tb, set.Set(flags.BackupPublicKeysFlag.Name, cfg.backupPublicKeys))
	assert.NoError(tb, set.Set(flags.BackupPasswordFile.Name, cfg.backupPasswordFile))
	assert.NoError(tb, set.Set(flags.WalletPasswordFileFlag.Name, cfg.walletPasswordFile))
	assert.NoError(tb, set.Set(flags.AccountPasswordFileFlag.Name, cfg.accountPasswordFile))
	assert.NoError(tb, set.Set(flags.NumAccountsFlag.Name, strconv.Itoa(int(cfg.numAccounts))))
	assert.NoError(tb, set.Set(flags.SkipDepositConfirmationFlag.Name, strconv.FormatBool(cfg.skipDepositConfirm)))
	assert.NoError(tb, set.Set(flags.ExitAllFlag.Name, strconv.FormatBool(cfg.exitAll)))
	assert.NoError(tb, set.Set(flags.GrpcHeadersFlag.Name, cfg.grpcHeaders))
	return cli.NewContext(&app, set, nil)
}

func setupWalletAndPasswordsDir(t testing.TB) (string, string, string) {
	walletDir := filepath.Join(t.TempDir(), "wallet")
	passwordsDir := filepath.Join(t.TempDir(), "passwords")
	passwordFileDir := filepath.Join(t.TempDir(), "passwordFile")
	require.NoError(t, os.MkdirAll(passwordFileDir, params.BeaconIoConfig().ReadWriteExecutePermissions))
	passwordFilePath := filepath.Join(passwordFileDir, passwordFileName)
	require.NoError(t, os.WriteFile(passwordFilePath, []byte(password), os.ModePerm))
	return walletDir, passwordsDir, passwordFilePath
}

func createRandomKeystore(t testing.TB, password string) *keymanager.Keystore {
	encryptor := keystorev4.New()
	id, err := uuid.NewRandom()
	require.NoError(t, err)
	validatingKey, err := bls.RandKey()
	require.NoError(t, err)
	pubKey := validatingKey.PublicKey().Marshal()
	cryptoFields, err := encryptor.Encrypt(validatingKey.Marshal(), password)
	require.NoError(t, err)
	return &keymanager.Keystore{
		Crypto:      cryptoFields,
		Pubkey:      fmt.Sprintf("%x", pubKey),
		ID:          id.String(),
		Version:     encryptor.Version(),
		Description: encryptor.Name(),
	}
}

func TestListAccounts_LocalKeymanager(t *testing.T) {
	walletDir, passwordsDir, walletPasswordFile := setupWalletAndPasswordsDir(t)
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		keymanagerKind:     keymanager.Local,
		walletPasswordFile: walletPasswordFile,
	})
	opts := []Option{
		WithWalletDir(walletDir),
		WithKeymanagerType(keymanager.Local),
		WithWalletPassword("Passwordz0320$"),
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

	numAccounts := 5
	keystores := make([]*keymanager.Keystore, numAccounts)
	passwords := make([]string, numAccounts)
	for i := 0; i < numAccounts; i++ {
		keystores[i] = createRandomKeystore(t, password)
		passwords[i] = password
	}
	_, err = km.ImportKeystores(cliCtx.Context, keystores, passwords)
	require.NoError(t, err)

	rescueStdout := os.Stdout
	r, writer, err := os.Pipe()
	require.NoError(t, err)
	os.Stdout = writer

	// We call the list local keymanager accounts function.
	require.NoError(
		t,
		km.ListKeymanagerAccounts(cliCtx.Context,
			keymanager.ListKeymanagerAccountConfig{
				ShowDepositData: true,
				ShowPrivateKeys: true,
			}),
	)

	require.NoError(t, writer.Close())
	out, err := io.ReadAll(r)
	require.NoError(t, err)
	os.Stdout = rescueStdout

	// Get stdout content and split to lines
	newLine := fmt.Sprintln()
	lines := strings.Split(string(out), newLine)

	// Expected output example:
	/*
		(keymanager kind) local wallet

		Showing 5 validator accounts
		View the eth1 deposit transaction data for your accounts by running `validator accounts list --show-deposit-data

		Account 0 | fully-evolving-fawn
		[validating public key] 0xa6669aa0381c06470b9a6faf8abf4194ad5148a62e461cbef5a6bc4d292026f58b992c4cf40e50552d301cef19da75b9
		[validating private key] 0x50cabc13435fcbde9d240fe720aff84f8557a6c1c445211b904f1a9620668241
		If you imported your account coming from the Ethereum launchpad, you will find your deposit_data.json in the eth2.0-deposit-cli's validator_keys folder


		Account 1 | preferably-mighty-heron
		[validating public key] 0xa7ea37fa2e2272762ffed8486f09b13cd56d76cf03a2a3e75bc36bd1719add84c20597671750be5bc1ccd3dadfebc30f
		[validating private key] 0x44563da0d11bc6a7219d18217cce8cdd064de3ebee5cdcf8d901c2fae7545116
		If you imported your account coming from the Ethereum eth2 launchpad, you will find your deposit_data.json in the eth2.0-deposit-cli's validator_keys folder


		Account 2 | conversely-good-monitor
		[validating public key] 0xa4c63619fb8cb87f6dd1686c9255f99c68066797bf284488ecbab64b1926d33eefdf96d1ee89ae4a89e84e7fb019d5e5
		[validating private key] 0x4448d0ab17ecd73bbb636ddbfc89b181731f6cd88c33f2cecc0d04cba1a18447
		If you imported your account coming from the Ethereum eth2 launchpad, you will find your deposit_data.json in the eth2.0-deposit-cli's validator_keys folder


		Account 3 | rarely-joint-mako
		[validating public key] 0x91dd8d5bfc22aea398740ebcea66ced159df8d3f1a066d7aba9f0bef4ed6d9687fc1fd1c87bd2b6d12b0788dfb6a7d20
		[validating private key] 0x4d1944bd7375185f70b3e70c68d9e6307f2009de3a4cf47ca5217443ddf81fc9
		If you imported your account coming from the Ethereum eth2 launchpad, you will find your deposit_data.json in the eth2.0-deposit-cli's validator_keys folder


		Account 4 | mainly-useful-catfish
		[validating public key] 0x83c4d722a98b599e2666bbe35146ff44800256190bc662f2dd5efbc0c4c0d57e5d297487a4f9c21a932d3b1b40e8379f
		[validating private key] 0x284cd65030496bf82ee2d52963cd540a1abb2cc738b8164901bbe7e2df4d57bd
		If you imported your account coming from the Ethereum eth2 launchpad, you will find your deposit_data.json in the eth2.0-deposit-cli's validator_keys folder



	*/

	// Expected output format definition
	const prologLength = 4
	const accountLength = 6
	const epilogLength = 2
	const nameOffset = 1
	const keyOffset = 2
	const privkeyOffset = 3

	// Require the output has correct number of lines
	lineCount := prologLength + accountLength*numAccounts + epilogLength
	require.Equal(t, lineCount, len(lines))

	// Assert the keymanager kind is printed on the first line.
	kindString := "local"
	kindFound := strings.Contains(lines[0], kindString)
	assert.Equal(t, true, kindFound, "Keymanager Kind %s not found on the first line", kindString)

	// Get account names and require the correct count
	accountNames, err := km.ValidatingAccountNames()
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(accountNames))

	// Assert that account names are printed on the correct lines
	for i, accountName := range accountNames {
		lineNumber := prologLength + accountLength*i + nameOffset
		accountNameFound := strings.Contains(lines[lineNumber], accountName)
		assert.Equal(t, true, accountNameFound, "Account Name %s not found on line number %d", accountName, lineNumber)
	}

	// Get public keys and require the correct count
	pubKeys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(pubKeys))

	// Assert that public keys are printed on the correct lines
	for i, key := range pubKeys {
		lineNumber := prologLength + accountLength*i + keyOffset
		keyString := fmt.Sprintf("%#x", key)
		keyFound := strings.Contains(lines[lineNumber], keyString)
		assert.Equal(t, true, keyFound, "Public Key %s not found on line number %d", keyString, lineNumber)
	}

	// Get private keys and require the correct count
	privKeys, err := km.FetchValidatingPrivateKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(pubKeys))

	// Assert that private keys are printed on the correct lines
	for i, key := range privKeys {
		lineNumber := prologLength + accountLength*i + privkeyOffset
		keyString := fmt.Sprintf("%#x", key)
		keyFound := strings.Contains(lines[lineNumber], keyString)
		assert.Equal(t, true, keyFound, "Private Key %s not found on line number %d", keyString, lineNumber)
	}

	rescueStdout = os.Stdout
	r, writer, err = os.Pipe()
	require.NoError(t, err)
	os.Stdout = writer

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := validatormock.NewMockValidatorClient(ctrl)
	var pks [][]byte
	for i := range pubKeys {
		pks = append(pks, pubKeys[i][:])
	}
	req := &ethpb.MultipleValidatorStatusRequest{PublicKeys: pks}
	resp := &ethpb.MultipleValidatorStatusResponse{Indices: []types.ValidatorIndex{1, math.MaxUint64, 2}}

	m.
		EXPECT().
		MultipleValidatorStatus(gomock.Any(), gomock.Eq(req)).
		Return(resp, nil)

	require.NoError(
		t,
		listValidatorIndices(
			cliCtx.Context,
			km,
			m,
		),
	)
	require.NoError(t, writer.Close())
	out, err = io.ReadAll(r)
	require.NoError(t, err)
	os.Stdout = rescueStdout

	expectedStdout := au.BrightGreen("Validator indices:").Bold().String() +
		fmt.Sprintf("\n%#x: %d", pubKeys[0][0:4], 1) +
		fmt.Sprintf("\n%#x: %d\n", pubKeys[2][0:4], 2)
	require.Equal(t, expectedStdout, string(out))
}

func TestListAccounts_DerivedKeymanager(t *testing.T) {
	walletDir, passwordsDir, passwordFilePath := setupWalletAndPasswordsDir(t)
	cliCtx := setupWalletCtx(t, &testWalletConfig{
		walletDir:          walletDir,
		passwordsDir:       passwordsDir,
		keymanagerKind:     keymanager.Derived,
		walletPasswordFile: passwordFilePath,
	})
	opts := []Option{
		WithWalletDir(walletDir),
		WithKeymanagerType(keymanager.Derived),
		WithWalletPassword("Passwordz0320$"),
	}
	acc, err := NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(cliCtx.Context)
	require.NoError(t, err)

	km, err := derived.NewKeymanager(
		cliCtx.Context,
		&derived.SetupConfig{
			Wallet:           w,
			ListenForChanges: false,
		},
	)
	require.NoError(t, err)

	numAccounts := 5
	err = km.RecoverAccountsFromMnemonic(cliCtx.Context, constant.TestMnemonic, derived.DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)

	rescueStdout := os.Stdout
	r, writer, err := os.Pipe()
	require.NoError(t, err)
	os.Stdout = writer

	// We call the list local keymanager accounts function.
	require.NoError(t, km.ListKeymanagerAccounts(cliCtx.Context,
		keymanager.ListKeymanagerAccountConfig{ShowPrivateKeys: true}))

	require.NoError(t, writer.Close())
	out, err := io.ReadAll(r)
	require.NoError(t, err)
	os.Stdout = rescueStdout

	// Get stdout content and split to lines
	newLine := fmt.Sprintln()
	lines := strings.Split(string(out), newLine)

	// Expected output example:
	/*
		(keymanager kind) derived, (HD) hierarchical-deterministic
		(derivation format) m / purpose / coin_type / account_index / withdrawal_key / validating_key
		Showing 2 validator accounts

		Account 0 | uniquely-sunny-tarpon
		[withdrawal public key] 0xa5faa97252104b408340b5d8cae3fa01023fa4dc9e7c7b470821433cf3a2a18158410b7d8a6dcdcd176c6552c2526681
		[withdrawal private key] 0x5266fd1f13d7af74614fde4fed3b664bfd529bc4ad91118e3db73647b99546df
		[derivation path] m/12381/3600/0/0
		[validating public key] 0xa7292d8f8d1c1f3d42cacefd2fc4cd3b82651be37c1eb790bbd294a874829f4b7e1c167345dcc1966cc844132b38097e
		[validating private key] 0x590707187dae64b42b8d36a95f3d7e11313ddd8b8d871b09e478e08c9bc8740b
		[derivation path] m/12381/3600/0/0/0

		======================Eth1 Deposit Transaction Data=====================

		0x22895118000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001205a9e92992d6a97ad113d217fa35cbe0659c662afe913ffd3a3ba61d7473be5630000000000000000000000000000000000000000000000000000000000000030a7292d8f8d1c1f3d42cacefd2fc4cd3b82651be37c1eb790bbd294a874829f4b7e1c167345dcc1966cc844132b38097e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020003b8f70706c37fb0b8dcbd95340889bad7d7f29121ea895052a8b216de95e480000000000000000000000000000000000000000000000000000000000000060b6727242b055448defbf54292c65e30ae28ca3aef8a07c8fe674abc0ca42a324be2e7592d3e45bba84ca364d7fe1f0ce073bf8b3692246395aa127cdbf93c64ae9ca48f85cb4b1e519f6821998181de1c7465b2bdcae4ddd0dbc2d02a56219d9

		===================================================================

		Account 1 | usually-obliging-pelican
		[withdrawal public key] 0xb91840d33bb87338bb28605cff837acd50e43a174a8a6d3893108fb91217fa428c12f1b2a25cf3c7aca75d418bcf0384
		[withdrawal private key] 0x72c5ffa7d08fb16cd35a9cb10494dfd49b46842ea1bcc1a4cf46b46680b66810
		[derivation path] m/12381/3600/1/0
		[validating public key] 0x8447f878b701dad4dfa5a884cebc4745b0e8f21340dc56c840826537764dcc54e2e68f80b8d4e5737180212a26211891
		[validating private key] 0x2cd5b1cddc9d96e50a16bea05d0953447655e3dd59fa1bfefad467c73d6c164a
		[derivation path] m/12381/3600/1/0/0

		======================Eth1 Deposit Transaction Data=====================

		0x22895118000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001200a0b9079c33cc40d602a50f5c51f6db30b0f959fc6f58048d6d43319fea6c09000000000000000000000000000000000000000000000000000000000000000308447f878b701dad4dfa5a884cebc4745b0e8f21340dc56c840826537764dcc54e2e68f80b8d4e5737180212a2621189100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000d6ac42bde23388e7428c1247364347c027c3507e461d68b851d506c60364cf0000000000000000000000000000000000000000000000000000000000000060801a2d432595164d7d88ae1695618db511d1507108573b8471098536b2b5a23f6711235f0a9c6fa65ac26cbd0f2d97e013e0c72ab6b5cff406c48d99ec0a2439aa9faa4557d20bb210d451519101616fa20b1ff2c67fae561cdff160fbc7dc98

		===================================================================


	*/

	// Expected output format definition
	const prologLength = 3
	const accountLength = 6
	const epilogLength = 1
	const nameOffset = 1
	const keyOffset = 2
	const validatingPrivateKeyOffset = 3

	// Require the output has correct number of lines
	lineCount := prologLength + accountLength*numAccounts + epilogLength
	require.Equal(t, lineCount, len(lines))

	// Assert the keymanager kind is printed on the first line.
	kindString := w.KeymanagerKind().String()
	kindFound := strings.Contains(lines[0], kindString)
	assert.Equal(t, true, kindFound, "Keymanager Kind %s not found on the first line", kindString)

	// Get account names and require the correct count
	accountNames, err := km.ValidatingAccountNames(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(accountNames))

	// Assert that account names are printed on the correct lines
	for i, accountName := range accountNames {
		lineNumber := prologLength + accountLength*i + nameOffset
		accountNameFound := strings.Contains(lines[lineNumber], accountName)
		assert.Equal(t, true, accountNameFound, "Account Name %s not found on line number %d", accountName, lineNumber)
	}

	// Get public keys and require the correct count
	pubKeys, err := km.FetchValidatingPublicKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(pubKeys))

	// Assert that public keys are printed on the correct lines
	for i, key := range pubKeys {
		lineNumber := prologLength + accountLength*i + keyOffset
		keyString := fmt.Sprintf("%#x", key)
		keyFound := strings.Contains(lines[lineNumber], keyString)
		assert.Equal(t, true, keyFound, "Public Key %s not found on line number %d", keyString, lineNumber)
	}

	// Get validating private keys and require the correct count
	validatingPrivKeys, err := km.FetchValidatingPrivateKeys(cliCtx.Context)
	require.NoError(t, err)
	require.Equal(t, numAccounts, len(pubKeys))

	// Assert that validating private keys are printed on the correct lines
	for i, key := range validatingPrivKeys {
		lineNumber := prologLength + accountLength*i + validatingPrivateKeyOffset
		keyString := fmt.Sprintf("%#x", key)
		keyFound := strings.Contains(lines[lineNumber], keyString)
		assert.Equal(t, true, keyFound, "Validating Private Key %s not found on line number %d", keyString, lineNumber)
	}
}
