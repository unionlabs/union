package node

import (
	"context"
	"flag"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path"
	"path/filepath"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorserviceconfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/db/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	dbTest "github.com/prysmaticlabs/prysm/v4/validator/db/testing"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	remoteweb3signer "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer"
	logtest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

// Test that the sharding node can build with default flag values.
func TestNode_Builds(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("datadir", t.TempDir()+"/datadir", "the node data directory")
	dir := t.TempDir() + "/walletpath"
	passwordDir := t.TempDir() + "/password"
	require.NoError(t, os.MkdirAll(passwordDir, os.ModePerm))
	passwordFile := filepath.Join(passwordDir, "password.txt")
	walletPassword := "$$Passw0rdz2$$"
	require.NoError(t, os.WriteFile(
		passwordFile,
		[]byte(walletPassword),
		os.ModePerm,
	))
	set.String("wallet-dir", dir, "path to wallet")
	set.String("wallet-password-file", passwordFile, "path to wallet password")
	set.String("keymanager-kind", "imported", "keymanager kind")
	set.String("verbosity", "debug", "log verbosity")
	require.NoError(t, set.Set(flags.WalletPasswordFileFlag.Name, passwordFile))
	ctx := cli.NewContext(&app, set, nil)
	opts := []accounts.Option{
		accounts.WithWalletDir(dir),
		accounts.WithKeymanagerType(keymanager.Local),
		accounts.WithWalletPassword(walletPassword),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	_, err = acc.WalletCreate(ctx.Context)
	require.NoError(t, err)

	valClient, err := NewValidatorClient(ctx)
	require.NoError(t, err, "Failed to create ValidatorClient")
	err = valClient.db.Close()
	require.NoError(t, err)
}

func TestGetLegacyDatabaseLocation(t *testing.T) {
	dataDir := t.TempDir()
	dataFile := path.Join(dataDir, "dataFile")
	nonExistingDataFile := path.Join(dataDir, "nonExistingDataFile")
	_, err := os.Create(dataFile)
	require.NoError(t, err, "Failed to create data file")

	walletDir := t.TempDir()
	derivedDir := path.Join(walletDir, "derived")
	err = file.MkdirAll(derivedDir)
	require.NoError(t, err, "Failed to create derived dir")

	derivedDbFile := path.Join(derivedDir, kv.ProtectionDbFileName)
	_, err = os.Create(derivedDbFile)
	require.NoError(t, err, "Failed to create derived db file")

	dbFile := path.Join(walletDir, kv.ProtectionDbFileName)
	_, err = os.Create(dbFile)
	require.NoError(t, err, "Failed to create db file")

	nonExistingWalletDir := t.TempDir()

	testCases := []struct {
		name                      string
		isInteropNumValidatorsSet bool
		isWeb3SignerURLFlagSet    bool
		dataDir                   string
		dataFile                  string
		walletDir                 string
		validatorClient           *ValidatorClient
		wallet                    *wallet.Wallet
		expectedDataDir           string
		expectedDataFile          string
	}{
		{
			name:                      "interop num validators set",
			isInteropNumValidatorsSet: true,
			dataDir:                   dataDir,
			dataFile:                  dataFile,
			expectedDataDir:           dataDir,
			expectedDataFile:          dataFile,
		},
		{
			name:             "dataDir differs from default",
			dataDir:          dataDir,
			dataFile:         dataFile,
			expectedDataDir:  dataDir,
			expectedDataFile: dataFile,
		},
		{
			name:             "dataFile exists",
			dataDir:          cmd.DefaultDataDir(),
			dataFile:         dataFile,
			expectedDataDir:  cmd.DefaultDataDir(),
			expectedDataFile: dataFile,
		},
		{
			name:             "wallet is nil",
			dataDir:          cmd.DefaultDataDir(),
			dataFile:         nonExistingDataFile,
			expectedDataDir:  cmd.DefaultDataDir(),
			expectedDataFile: nonExistingDataFile,
		},
		{
			name:     "web3signer url is not set and legacy data file does not exist",
			dataDir:  cmd.DefaultDataDir(),
			dataFile: nonExistingDataFile,
			wallet: wallet.New(&wallet.Config{
				WalletDir:      nonExistingWalletDir,
				KeymanagerKind: keymanager.Derived,
			}),
			expectedDataDir:  cmd.DefaultDataDir(),
			expectedDataFile: nonExistingDataFile,
		},
		{
			name:     "web3signer url is not set and legacy data file does exist",
			dataDir:  cmd.DefaultDataDir(),
			dataFile: nonExistingDataFile,
			wallet: wallet.New(&wallet.Config{
				WalletDir:      walletDir,
				KeymanagerKind: keymanager.Derived,
			}),
			expectedDataDir:  path.Join(walletDir, "derived"),
			expectedDataFile: path.Join(walletDir, "derived", kv.ProtectionDbFileName),
		},
		{
			name:                   "web3signer url is set and legacy data file does not exist",
			isWeb3SignerURLFlagSet: true,
			dataDir:                cmd.DefaultDataDir(),
			dataFile:               nonExistingDataFile,
			walletDir:              nonExistingWalletDir,
			wallet: wallet.New(&wallet.Config{
				WalletDir:      walletDir,
				KeymanagerKind: keymanager.Derived,
			}),
			expectedDataDir:  cmd.DefaultDataDir(),
			expectedDataFile: nonExistingDataFile,
		},
		{
			name:                   "web3signer url is set and legacy data file does exist",
			isWeb3SignerURLFlagSet: true,
			dataDir:                cmd.DefaultDataDir(),
			dataFile:               nonExistingDataFile,
			walletDir:              walletDir,
			wallet: wallet.New(&wallet.Config{
				WalletDir:      walletDir,
				KeymanagerKind: keymanager.Derived,
			}),
			expectedDataDir:  walletDir,
			expectedDataFile: path.Join(walletDir, kv.ProtectionDbFileName),
		},
	}

	for _, tt := range testCases {
		t.Run(tt.name, func(t *testing.T) {
			validatorClient := &ValidatorClient{wallet: tt.wallet}
			actualDataDir, actualDataFile := validatorClient.getLegacyDatabaseLocation(
				tt.isInteropNumValidatorsSet,
				tt.isWeb3SignerURLFlagSet,
				tt.dataDir,
				tt.dataFile,
				tt.walletDir,
			)

			assert.Equal(t, tt.expectedDataDir, actualDataDir, "data dir should be equal")
			assert.Equal(t, tt.expectedDataFile, actualDataFile, "data file should be equal")
		})

	}

}

// TestClearDB tests clearing the database
func TestClearDB(t *testing.T) {
	hook := logtest.NewGlobal()
	tmp := filepath.Join(t.TempDir(), "datadirtest")
	require.NoError(t, clearDB(context.Background(), tmp, true))
	require.LogsContain(t, hook, "Removing database")
}

// TestWeb3SignerConfig tests the web3 signer config returns the correct values.
func TestWeb3SignerConfig(t *testing.T) {
	pubkey1decoded, err := hexutil.Decode("0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c")
	require.NoError(t, err)
	bytepubkey1 := bytesutil.ToBytes48(pubkey1decoded)

	pubkey2decoded, err := hexutil.Decode("0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b")
	require.NoError(t, err)
	bytepubkey2 := bytesutil.ToBytes48(pubkey2decoded)

	type args struct {
		baseURL          string
		publicKeysOrURLs []string
	}
	tests := []struct {
		name       string
		args       *args
		want       *remoteweb3signer.SetupConfig
		wantErrMsg string
	}{
		{
			name: "happy path with public keys",
			args: &args{
				baseURL: "http://localhost:8545",
				publicKeysOrURLs: []string{"0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c," +
					"0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b"},
			},
			want: &remoteweb3signer.SetupConfig{
				BaseEndpoint:          "http://localhost:8545",
				GenesisValidatorsRoot: nil,
				PublicKeysURL:         "",
				ProvidedPublicKeys: [][48]byte{
					bytepubkey1,
					bytepubkey2,
				},
			},
		},
		{
			name: "happy path with external url",
			args: &args{
				baseURL:          "http://localhost:8545",
				publicKeysOrURLs: []string{"http://localhost:8545/api/v1/eth2/publicKeys"},
			},
			want: &remoteweb3signer.SetupConfig{
				BaseEndpoint:          "http://localhost:8545",
				GenesisValidatorsRoot: nil,
				PublicKeysURL:         "http://localhost:8545/api/v1/eth2/publicKeys",
				ProvidedPublicKeys:    nil,
			},
		},
		{
			name: "Bad base URL",
			args: &args{
				baseURL: "0xa99a76ed7796f7be22d5b7e85deeb7c5677e88,",
				publicKeysOrURLs: []string{"0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c," +
					"0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b"},
			},
			want:       nil,
			wantErrMsg: "web3signer url 0xa99a76ed7796f7be22d5b7e85deeb7c5677e88, is invalid: parse \"0xa99a76ed7796f7be22d5b7e85deeb7c5677e88,\": invalid URI for request",
		},
		{
			name: "Bad publicKeys",
			args: &args{
				baseURL: "http://localhost:8545",
				publicKeysOrURLs: []string{"0xa99a76ed7796f7be22c," +
					"0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b"},
			},
			want:       nil,
			wantErrMsg: "could not decode public key for web3signer: 0xa99a76ed7796f7be22c: hex string of odd length",
		},
		{
			name: "Bad publicKeysURL",
			args: &args{
				baseURL:          "http://localhost:8545",
				publicKeysOrURLs: []string{"localhost"},
			},
			want:       nil,
			wantErrMsg: "could not decode public key for web3signer: localhost: hex string without 0x prefix",
		},
		{
			name: "Base URL missing scheme or host",
			args: &args{
				baseURL:          "localhost:8545",
				publicKeysOrURLs: []string{"localhost"},
			},
			want:       nil,
			wantErrMsg: "web3signer url must be in the format of http(s)://host:port url used: localhost:8545",
		},
		{
			name: "Public Keys URL missing scheme or host",
			args: &args{
				baseURL:          "http://localhost:8545",
				publicKeysOrURLs: []string{"localhost:8545"},
			},
			want:       nil,
			wantErrMsg: "could not decode public key for web3signer: localhost:8545: hex string without 0x prefix",
		},
		{
			name: "incorrect amount of flag calls used with url",
			args: &args{
				baseURL: "http://localhost:8545",
				publicKeysOrURLs: []string{"0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c," +
					"0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b", "http://localhost:8545/api/v1/eth2/publicKeys"},
			},
			want:       nil,
			wantErrMsg: "could not decode public key for web3signer",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			app := cli.App{}
			set := flag.NewFlagSet(tt.name, 0)
			set.String("validators-external-signer-url", tt.args.baseURL, "baseUrl")
			c := &cli.StringSliceFlag{
				Name: "validators-external-signer-public-keys",
			}
			err := c.Apply(set)
			require.NoError(t, err)
			require.NoError(t, set.Set(flags.Web3SignerURLFlag.Name, tt.args.baseURL))
			for _, key := range tt.args.publicKeysOrURLs {
				require.NoError(t, set.Set(flags.Web3SignerPublicValidatorKeysFlag.Name, key))
			}
			cliCtx := cli.NewContext(&app, set, nil)
			got, err := Web3SignerConfig(cliCtx)
			if tt.wantErrMsg != "" {
				require.ErrorContains(t, tt.wantErrMsg, err)
				return
			}
			require.DeepEqual(t, tt.want, got)
		})
	}
}

func TestProposerSettings(t *testing.T) {
	hook := logtest.NewGlobal()

	type proposerSettingsFlag struct {
		dir        string
		url        string
		defaultfee string
		defaultgas string
	}

	type args struct {
		proposerSettingsFlagValues *proposerSettingsFlag
	}
	tests := []struct {
		name                         string
		args                         args
		want                         func() *validatorserviceconfig.ProposerSettings
		urlResponse                  string
		wantErr                      string
		wantLog                      string
		withdb                       func(db iface.ValidatorDB) error
		validatorRegistrationEnabled bool
	}{
		{
			name: "Happy Path default only proposer settings file with builder settings,",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/default-only-proposer-config.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0xae967917c465db8578ca9024c205720b1a3651A9"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
		},
		{
			name: "Happy Path Config file File, bad checksum",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config-badchecksum.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0xae967917c465db8578ca9024c205720b1a3651A9"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0xae967917c465db8578ca9024c205720b1a3651A9"),
						},
					},
				}
			},
			wantErr: "",
			wantLog: "is not a checksum Ethereum address",
		},
		{
			name: "Happy Path Config file File multiple fee recipients",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config-multiple.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				key2, err := hexutil.Decode("0xb057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7b")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
							},
						},
						bytesutil.ToBytes48(key2): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x60155530FCE8a85ec7055A5F8b2bE214B3DaeFd4"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(35000000),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(40000000),
						},
					},
				}
			},
			wantErr: "",
		},
		{
			name: "Happy Path Config URL File",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "./testdata/good-prepare-beacon-proposer-config.json",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
			},
			wantErr: "",
		},
		{
			name: "Happy Path Config YAML file with custom Gas Limit",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.yaml",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: 40000000,
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  false,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			wantErr: "",
		},
		{
			name: "Happy Path Suggested Fee ",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "0x6e35733c5af9B61374A128e6F85f553aF09ff89A",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: nil,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
			},
			wantErr: "",
		},
		{
			name: "Happy Path Suggested Fee , validator registration enabled",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "0x6e35733c5af9B61374A128e6F85f553aF09ff89A",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: nil,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			wantErr:                      "",
			validatorRegistrationEnabled: true,
		},
		{
			name: "Happy Path Suggested Fee , validator registration enabled and default gas",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "0x6e35733c5af9B61374A128e6F85f553aF09ff89A",
					defaultgas: "50000000",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: nil,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: 50000000,
						},
					},
				}
			},
			wantErr:                      "",
			validatorRegistrationEnabled: true,
		},
		{
			name: "Suggested Fee does not Override Config",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.json",
					url:        "",
					defaultfee: "0x6e35733c5af9B61374A128e6F85f553aF09ff89B",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
			},
			wantErr: "",
		},
		{
			name: "Suggested Fee with validator registration does not Override Config",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.json",
					url:        "",
					defaultfee: "0x6e35733c5af9B61374A128e6F85f553aF09ff89B",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			wantErr:                      "",
			validatorRegistrationEnabled: true,
		},
		{
			name: "Enable Builder flag overrides empty config",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			validatorRegistrationEnabled: true,
		},
		{
			name: "Enable Builder flag does override completed builder config",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.yaml",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(40000000),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			validatorRegistrationEnabled: true,
		},
		{
			name: "Only Enable Builder flag",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			validatorRegistrationEnabled: true,
		},
		{
			name: "No Flags but saved to DB with builder and override removed builder data",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
			},
			withdb: func(db iface.ValidatorDB) error {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				settings := &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(40000000),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
				return db.SaveProposerSettings(context.Background(), settings)
			},
		},
		{
			name: "Enable builder flag but saved to DB without builder data now includes builder data",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
							BuilderConfig: &validatorserviceconfig.BuilderConfig{
								Enabled:  true,
								GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				}
			},
			withdb: func(db iface.ValidatorDB) error {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				settings := &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
				return db.SaveProposerSettings(context.Background(), settings)
			},
			validatorRegistrationEnabled: true,
		},
		{
			name: "No flags, but saved to database",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				return &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
			},
			withdb: func(db iface.ValidatorDB) error {
				key1, err := hexutil.Decode("0xa057816155ad77931185101128655c0191bd0214c201ca48ed887f6c4c6adf334070efcd75140eada5ac83a92506dd7a")
				require.NoError(t, err)
				settings := &validatorserviceconfig.ProposerSettings{
					ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
						bytesutil.ToBytes48(key1): {
							FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
								FeeRecipient: common.HexToAddress("0x50155530FCE8a85ec7055A5F8b2bE214B3DaeFd3"),
							},
						},
					},
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress("0x6e35733c5af9B61374A128e6F85f553aF09ff89A"),
						},
					},
				}
				return db.SaveProposerSettings(context.Background(), settings)
			},
		},
		{
			name: "No flags set means empty config",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return nil
			},
			wantErr: "",
		},
		{
			name: "Bad File Path",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/bad-prepare-beacon-proposer-config.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return nil
			},
			wantErr: "failed to unmarshal yaml file",
		},
		{
			name: "Both URL and Dir flags used resulting in error",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/good-prepare-beacon-proposer-config.json",
					url:        "./testdata/good-prepare-beacon-proposer-config.json",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return &validatorserviceconfig.ProposerSettings{}
			},
			wantErr: "cannot specify both",
		},
		{
			name: "Bad Gas value in JSON",
			args: args{
				proposerSettingsFlagValues: &proposerSettingsFlag{
					dir:        "./testdata/bad-gas-value-proposer-settings.json",
					url:        "",
					defaultfee: "",
				},
			},
			want: func() *validatorserviceconfig.ProposerSettings {
				return nil
			},
			wantErr: "failed to unmarshal yaml file",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			app := cli.App{}
			set := flag.NewFlagSet("test", 0)
			if tt.args.proposerSettingsFlagValues.dir != "" {
				set.String(flags.ProposerSettingsFlag.Name, tt.args.proposerSettingsFlagValues.dir, "")
				require.NoError(t, set.Set(flags.ProposerSettingsFlag.Name, tt.args.proposerSettingsFlagValues.dir))
			}
			if tt.args.proposerSettingsFlagValues.url != "" {
				content, err := os.ReadFile(tt.args.proposerSettingsFlagValues.url)
				require.NoError(t, err)
				srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
					w.WriteHeader(200)
					w.Header().Set("Content-Type", "application/json")
					_, err := fmt.Fprintf(w, "%s", content)
					require.NoError(t, err)
				}))
				defer srv.Close()

				set.String(flags.ProposerSettingsURLFlag.Name, tt.args.proposerSettingsFlagValues.url, "")
				require.NoError(t, set.Set(flags.ProposerSettingsURLFlag.Name, srv.URL))
			}
			if tt.args.proposerSettingsFlagValues.defaultfee != "" {
				set.String(flags.SuggestedFeeRecipientFlag.Name, tt.args.proposerSettingsFlagValues.defaultfee, "")
				require.NoError(t, set.Set(flags.SuggestedFeeRecipientFlag.Name, tt.args.proposerSettingsFlagValues.defaultfee))
			}
			if tt.args.proposerSettingsFlagValues.defaultgas != "" {
				set.String(flags.BuilderGasLimitFlag.Name, tt.args.proposerSettingsFlagValues.defaultgas, "")
				require.NoError(t, set.Set(flags.BuilderGasLimitFlag.Name, tt.args.proposerSettingsFlagValues.defaultgas))
			}
			if tt.validatorRegistrationEnabled {
				set.Bool(flags.EnableBuilderFlag.Name, true, "")
			}
			cliCtx := cli.NewContext(&app, set, nil)
			validatorDB := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
			if tt.withdb != nil {
				err := tt.withdb(validatorDB)
				require.NoError(t, err)
			}
			got, err := proposerSettings(cliCtx, validatorDB)
			if tt.wantErr != "" {
				require.ErrorContains(t, tt.wantErr, err)
				return
			}
			if tt.wantLog != "" {
				assert.LogsContain(t, hook,
					tt.wantLog,
				)
			}
			w := tt.want()
			require.DeepEqual(t, w, got)

		})
	}
}

func Test_ProposerSettingsWithOnlyBuilder_DoesNotSaveInDB(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Bool(flags.EnableBuilderFlag.Name, true, "")
	cliCtx := cli.NewContext(&app, set, nil)
	validatorDB := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	got, err := proposerSettings(cliCtx, validatorDB)
	require.NoError(t, err)
	_, err = validatorDB.ProposerSettings(cliCtx.Context)
	require.ErrorContains(t, "no proposer settings found in bucket", err)
	want := &validatorserviceconfig.ProposerSettings{
		DefaultConfig: &validatorserviceconfig.ProposerOption{
			BuilderConfig: &validatorserviceconfig.BuilderConfig{
				Enabled:  true,
				GasLimit: validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
				Relays:   nil,
			},
		},
	}
	require.DeepEqual(t, want, got)
}
