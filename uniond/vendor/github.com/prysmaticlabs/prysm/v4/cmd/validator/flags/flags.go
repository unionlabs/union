// Package flags contains all configuration runtime flags for
// the validator service.
package flags

import (
	"fmt"
	"path/filepath"
	"runtime"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/urfave/cli/v2"
)

const (
	// WalletDefaultDirName for accounts.
	WalletDefaultDirName = "prysm-wallet-v2"
	// DefaultGatewayHost for the validator client.
	DefaultGatewayHost = "127.0.0.1"
)

var (
	// DisableAccountMetricsFlag disables the prometheus metrics for validator accounts, default false.
	DisableAccountMetricsFlag = &cli.BoolFlag{
		Name: "disable-account-metrics",
		Usage: `Disables prometheus metrics for validator accounts. Operators with high volumes 
		of validating keys may wish to disable granular prometheus metrics as it increases
		the data cardinality.`,
	}
	// BeaconRPCProviderFlag defines a beacon node RPC endpoint.
	BeaconRPCProviderFlag = &cli.StringFlag{
		Name:  "beacon-rpc-provider",
		Usage: "Beacon node RPC provider endpoint.",
		Value: "127.0.0.1:4000",
	}
	// BeaconRPCGatewayProviderFlag defines a beacon node JSON-RPC endpoint.
	BeaconRPCGatewayProviderFlag = &cli.StringFlag{
		Name:  "beacon-rpc-gateway-provider",
		Usage: "Beacon node RPC gateway provider endpoint.",
		Value: "127.0.0.1:3500",
	}
	// BeaconRESTApiProviderFlag defines a beacon node REST API endpoint.
	BeaconRESTApiProviderFlag = &cli.StringFlag{
		Name:  "beacon-rest-api-provider",
		Usage: "Beacon node REST API provider endpoint.",
		Value: "http://127.0.0.1:3500",
	}
	// CertFlag defines a flag for the node's TLS certificate.
	CertFlag = &cli.StringFlag{
		Name:  "tls-cert",
		Usage: "Certificate for secure gRPC. Pass this and the tls-key flag in order to use gRPC securely.",
	}
	// EnableRPCFlag enables controlling the validator client via gRPC (without web UI).
	EnableRPCFlag = &cli.BoolFlag{
		Name:  "rpc",
		Usage: "Enables the RPC server for the validator client (without Web UI).",
		Value: false,
	}
	// RPCHost defines the host on which the RPC server should listen.
	RPCHost = &cli.StringFlag{
		Name:  "rpc-host",
		Usage: "Host on which the RPC server should listen.",
		Value: "127.0.0.1",
	}
	// RPCPort defines a validator client RPC port to open.
	RPCPort = &cli.IntFlag{
		Name:  "rpc-port",
		Usage: "RPC port exposed by a validator client.",
		Value: 7000,
	}
	// SlasherRPCProviderFlag defines a slasher node RPC endpoint.
	SlasherRPCProviderFlag = &cli.StringFlag{
		Name:  "slasher-rpc-provider",
		Usage: "Slasher node RPC provider endpoint.",
		Value: "127.0.0.1:4002",
	}
	// SlasherCertFlag defines a flag for the slasher node's TLS certificate.
	SlasherCertFlag = &cli.StringFlag{
		Name:  "slasher-tls-cert",
		Usage: "Certificate for secure slasher gRPC. Pass this and the tls-key flag in order to use gRPC securely.",
	}
	// DisablePenaltyRewardLogFlag defines the ability to not log reward/penalty information during deployment
	DisablePenaltyRewardLogFlag = &cli.BoolFlag{
		Name:  "disable-rewards-penalties-logging",
		Usage: "Disables reward/penalty logging during cluster deployment.",
	}
	// GraffitiFlag defines the graffiti value included in proposed blocks
	GraffitiFlag = &cli.StringFlag{
		Name:  "graffiti",
		Usage: "String to include in proposed blocks.",
	}
	// GrpcRetriesFlag defines the number of times to retry a failed gRPC request.
	GrpcRetriesFlag = &cli.UintFlag{
		Name:  "grpc-retries",
		Usage: "Number of attempts to retry gRPC requests.",
		Value: 5,
	}
	// GrpcRetryDelayFlag defines the interval to retry a failed gRPC request.
	GrpcRetryDelayFlag = &cli.DurationFlag{
		Name:  "grpc-retry-delay",
		Usage: "Amount of time between gRPC retry requests.",
		Value: 1 * time.Second,
	}
	// GrpcHeadersFlag defines a list of headers to send with all gRPC requests.
	GrpcHeadersFlag = &cli.StringFlag{
		Name: "grpc-headers",
		Usage: `Comma separated list of key value pairs to pass as gRPC headers for all gRPC calls.
		Example: --grpc-headers=key=value`,
	}
	// GRPCGatewayHost specifies a gRPC gateway host for the validator client.
	GRPCGatewayHost = &cli.StringFlag{
		Name:  "grpc-gateway-host",
		Usage: "Host on which the gateway server runs on.",
		Value: DefaultGatewayHost,
	}
	// GRPCGatewayPort enables a gRPC gateway to be exposed for the validator client.
	GRPCGatewayPort = &cli.IntFlag{
		Name:  "grpc-gateway-port",
		Usage: "Enables gRPC gateway for JSON requests.",
		Value: 7500,
	}
	// GPRCGatewayCorsDomain serves preflight requests when serving gRPC JSON gateway.
	GPRCGatewayCorsDomain = &cli.StringFlag{
		Name: "grpc-gateway-corsdomain",
		Usage: `Comma separated list of domains from which to accept cross origin requests (browser enforced).
		This flag has no effect if not used with --grpc-gateway-port.
`,
		Value: "http://localhost:7500,http://127.0.0.1:7500,http://0.0.0.0:7500,http://localhost:4242,http://127.0.0.1:4242,http://localhost:4200,http://0.0.0.0:4242,http://127.0.0.1:4200,http://0.0.0.0:4200,http://localhost:3000,http://0.0.0.0:3000,http://127.0.0.1:3000"}
	// MonitoringPortFlag defines the http port used to serve prometheus metrics.
	MonitoringPortFlag = &cli.IntFlag{
		Name:  "monitoring-port",
		Usage: "Port used to listening and respond metrics for Prometheus.",
		Value: 8081,
	}
	// WalletDirFlag defines the path to a wallet directory for Prysm accounts.
	WalletDirFlag = &cli.StringFlag{
		Name:  "wallet-dir",
		Usage: "Path to a wallet directory on-disk for Prysm validator accounts.",
		Value: filepath.Join(DefaultValidatorDir(), WalletDefaultDirName),
	}
	// AccountPasswordFileFlag is path to a file containing a password for a validator account.
	AccountPasswordFileFlag = &cli.StringFlag{
		Name:  "account-password-file",
		Usage: "Path to a plain-text, .txt file containing a password for a validator account.",
	}
	// WalletPasswordFileFlag is the path to a file containing your wallet password.
	WalletPasswordFileFlag = &cli.StringFlag{
		Name:  "wallet-password-file",
		Usage: "Path to a plain-text, .txt file containing your wallet password.",
	}
	// Mnemonic25thWordFileFlag defines a path to a file containing a "25th" word mnemonic passphrase for advanced users.
	Mnemonic25thWordFileFlag = &cli.StringFlag{
		Name:  "mnemonic-25th-word-file",
		Usage: "(Advanced) Path to a plain-text, `.txt` file containing a 25th word passphrase for your mnemonic for HD wallets.",
	}
	// SkipMnemonic25thWordCheckFlag allows for skipping a check for mnemonic 25th word passphrases for HD wallets.
	SkipMnemonic25thWordCheckFlag = &cli.StringFlag{
		Name:  "skip-mnemonic-25th-word-check",
		Usage: "Allows for skipping the check for a mnemonic 25th word passphrase for HD wallets.",
	}
	// ImportPrivateKeyFileFlag allows for directly importing a private key hex string as an account.
	ImportPrivateKeyFileFlag = &cli.StringFlag{
		Name:  "import-private-key-file",
		Usage: "Path to a plain-text, .txt file containing a hex string representation of a private key to import.",
	}
	// MnemonicFileFlag is used to enter a file to mnemonic phrase for new wallet creation, non-interactively.
	MnemonicFileFlag = &cli.StringFlag{
		Name:  "mnemonic-file",
		Usage: "File to retrieve mnemonic for non-interactively passing a mnemonic phrase into wallet recover.",
	}
	// MnemonicLanguageFlag is used to specify the language of the mnemonic.
	MnemonicLanguageFlag = &cli.StringFlag{
		Name:  "mnemonic-language",
		Usage: "Allows specifying mnemonic language. Supported languages are: english|chinese_traditional|chinese_simplified|czech|french|japanese|korean|italian|spanish.",
	}
	// ShowDepositDataFlag for accounts.
	ShowDepositDataFlag = &cli.BoolFlag{
		Name:  "show-deposit-data",
		Usage: "Displays raw eth1 tx deposit data for validator accounts.",
		Value: false,
	}
	// ShowPrivateKeysFlag for accounts.
	ShowPrivateKeysFlag = &cli.BoolFlag{
		Name:  "show-private-keys",
		Usage: "Displays the private keys for validator accounts.",
		Value: false,
	}
	// ListValidatorIndices for accounts.
	ListValidatorIndices = &cli.BoolFlag{
		Name:  "list-validator-indices",
		Usage: "Lists validator indices.",
		Value: false,
	}
	// NumAccountsFlag defines the amount of accounts to generate for derived wallets.
	NumAccountsFlag = &cli.IntFlag{
		Name:  "num-accounts",
		Usage: "Number of accounts to generate for derived wallets.",
		Value: 1,
	}
	// DeletePublicKeysFlag defines a comma-separated list of hex string public keys
	// for accounts which a user desires to delete from their wallet.
	DeletePublicKeysFlag = &cli.StringFlag{
		Name:  "delete-public-keys",
		Usage: "Comma separated list of public key hex strings to specify which validator accounts to delete.",
		Value: "",
	}
	// BackupPublicKeysFlag defines a comma-separated list of hex string public keys
	// for accounts which a user desires to backup from their wallet.
	BackupPublicKeysFlag = &cli.StringFlag{
		Name:  "backup-public-keys",
		Usage: "Comma separated list of public key hex strings to specify which validator accounts to backup.",
		Value: "",
	}
	// VoluntaryExitPublicKeysFlag defines a comma-separated list of hex string public keys
	// for accounts on which a user wants to perform a voluntary exit.
	VoluntaryExitPublicKeysFlag = &cli.StringFlag{
		Name: "public-keys",
		Usage: "Comma separated list of public key hex strings to specify on which validator accounts to perform " +
			"a voluntary exit.",
		Value: "",
	}
	// ExitAllFlag allows stakers to select all validating keys for exit. This will still require the staker
	// to confirm a userprompt for this action given it is a dangerous one.
	ExitAllFlag = &cli.BoolFlag{
		Name:  "exit-all",
		Usage: "Exits all validators. This will still require the staker to confirm a userprompt for the action.",
	}
	// ForceExitFlag to exit without displaying the confirmation prompt.
	ForceExitFlag = &cli.BoolFlag{
		Name:  "force-exit",
		Usage: "Exits without displaying the confirmation prompt.",
	}
	VoluntaryExitJSONOutputPath = &cli.StringFlag{
		Name: "exit-json-output-dir",
		Usage: "Output directory to write voluntary exits as individual unencrypted JSON " +
			"files. If this flag is provided, voluntary exits will be written to the provided " +
			"directory and will not be broadcasted.",
	}
	// BackupPasswordFile for encrypting accounts a user wishes to back up.
	BackupPasswordFile = &cli.StringFlag{
		Name:  "backup-password-file",
		Usage: "Path to a plain-text, .txt file containing the desired password for your backed up accounts.",
		Value: "",
	}
	// BackupDirFlag defines the path for the zip backup of the wallet will be created.
	BackupDirFlag = &cli.StringFlag{
		Name:  "backup-dir",
		Usage: "Path to a directory where accounts will be backed up into a zip file.",
		Value: DefaultValidatorDir(),
	}
	// SlashingProtectionJSONFileFlag is used to enter the file path of the slashing protection JSON.
	SlashingProtectionJSONFileFlag = &cli.StringFlag{
		Name:  "slashing-protection-json-file",
		Usage: "Path to an EIP-3076 compliant JSON file containing a user's slashing protection history.",
	}
	// KeysDirFlag defines the path for a directory where keystores to be imported at stored.
	KeysDirFlag = &cli.StringFlag{
		Name:  "keys-dir",
		Usage: "Path to a directory where keystores to be imported are stored.",
	}

	// RemoteSignerCertPathFlag defines the path to a client.crt file for a wallet to connect to
	// a secure signer via TLS and gRPC.
	RemoteSignerCertPathFlag = &cli.StringFlag{
		Name:  "remote-signer-crt-path",
		Usage: "/path/to/client.crt for establishing a secure, TLS gRPC connection to a remote signer server.",
		Value: "",
	}
	// RemoteSignerKeyPathFlag defines the path to a client.key file for a wallet to connect to
	// a secure signer via TLS and gRPC.
	RemoteSignerKeyPathFlag = &cli.StringFlag{
		Name:  "remote-signer-key-path",
		Usage: "/path/to/client.key for establishing a secure, TLS gRPC connection to a remote signer server.",
		Value: "",
	}
	// RemoteSignerCACertPathFlag defines the path to a ca.crt file for a wallet to connect to
	// a secure signer via TLS and gRPC.
	RemoteSignerCACertPathFlag = &cli.StringFlag{
		Name:  "remote-signer-ca-crt-path",
		Usage: "/path/to/ca.crt for establishing a secure, TLS gRPC connection to a remote signer server.",
		Value: "",
	}
	// Web3SignerURLFlag defines the URL for a web3signer to connect to.
	// example:--validators-external-signer-url=http://localhost:9000
	// web3signer documentation can be found in Consensys' web3signer project docs
	Web3SignerURLFlag = &cli.StringFlag{
		Name:  "validators-external-signer-url",
		Usage: "URL for consensys' web3signer software to use with the Prysm validator client.",
		Value: "",
	}

	// Web3SignerPublicValidatorKeysFlag defines a comma-separated list of hex string public keys or external url for web3signer to use for validator signing.
	// example with external url: --validators-external-signer-public-keys= https://web3signer.com/api/v1/eth2/publicKeys
	// example with public key: --validators-external-signer-public-keys=0xa99a...e44c,0xb89b...4a0b
	// web3signer documentation can be found in Consensys' web3signer project docs```
	Web3SignerPublicValidatorKeysFlag = &cli.StringSliceFlag{
		Name:  "validators-external-signer-public-keys",
		Usage: "Comma separated list of public keys OR an external url endpoint for the validator to retrieve public keys from for usage with web3signer.",
	}

	// KeymanagerKindFlag defines the kind of keymanager desired by a user during wallet creation.
	KeymanagerKindFlag = &cli.StringFlag{
		Name:  "keymanager-kind",
		Usage: "Kind of keymanager, either imported, derived, or remote, specified during wallet creation.",
		Value: "",
	}
	// SkipDepositConfirmationFlag skips the y/n confirmation userprompt for sending a deposit to the deposit contract.
	SkipDepositConfirmationFlag = &cli.BoolFlag{
		Name:  "skip-deposit-confirmation",
		Usage: "Skips the y/n confirmation userprompt for sending a deposit to the deposit contract.",
		Value: false,
	}
	// EnableWebFlag enables controlling the validator client via the Prysm web ui. This is a work in progress.
	EnableWebFlag = &cli.BoolFlag{
		Name:  "web",
		Usage: "(Work in progress): Enables the web portal for the validator client.",
		Value: false,
	}
	// SlashingProtectionExportDirFlag allows specifying the outpt directory
	// for a validator's slashing protection history.
	SlashingProtectionExportDirFlag = &cli.StringFlag{
		Name:  "slashing-protection-export-dir",
		Usage: "Allows users to specify the output directory to export their slashing protection EIP-3076 standard JSON File.",
		Value: "",
	}
	// GraffitiFileFlag specifies the file path to load graffiti values.
	GraffitiFileFlag = &cli.StringFlag{
		Name:  "graffiti-file",
		Usage: "Path to a YAML file with graffiti values.",
	}
	// ProposerSettingsFlag defines the path or URL to a file with proposer config.
	ProposerSettingsFlag = &cli.StringFlag{
		Name: "proposer-settings-file",
		Usage: `Sets path to a YAML or JSON file containing validator settings used when proposing blocks such as
		fee recipient and gas limit. File format found in docs.`,
		Value: "",
	}
	// ProposerSettingsURLFlag defines the path or URL to a file with proposer config.
	ProposerSettingsURLFlag = &cli.StringFlag{
		Name: "proposer-settings-url",
		Usage: `Sets URL to a REST endpoint containing validator settings used when proposing blocks such as
		fee recipient and gas limit. File format found in docs`,
		Value: "",
	}

	// SuggestedFeeRecipientFlag defines the address of the fee recipient.
	SuggestedFeeRecipientFlag = &cli.StringFlag{
		Name: "suggested-fee-recipient",
		Usage: `Sets ALL validators' mapping to a suggested eth address to receive gas fees when proposing a block.
		Note that this is only a suggestion when integrating with a Builder API, which may choose to specify
		a different fee recipient as payment for the blocks it builds.For additional setting overrides use the 
		--` + ProposerSettingsFlag.Name + " or --" + ProposerSettingsURLFlag.Name + " flags.",
		Value: params.BeaconConfig().EthBurnAddressHex,
	}

	// EnableBuilderFlag enables the periodic validator registration API calls that will update the custom builder with validator settings.
	EnableBuilderFlag = &cli.BoolFlag{
		Name: "enable-builder",
		Usage: `Enables builder validator registration APIs for the validator client to update settings
		such as fee recipient and gas limit. This flag is not required if using proposer
		settings config file.`,
		Value:   false,
		Aliases: []string{"enable-validator-registration"},
	}

	// BuilderGasLimitFlag defines the gas limit for the builder to use for constructing a payload.
	BuilderGasLimitFlag = &cli.StringFlag{
		Name:  "suggested-gas-limit",
		Usage: "Sets gas limit for the builder to use for constructing a payload for all the validators.",
		Value: fmt.Sprint(params.BeaconConfig().DefaultBuilderGasLimit),
	}

	// ValidatorsRegistrationBatchSizeFlag sets the maximum size for one batch of validator registrations. Use a non-positive value to disable batching.
	ValidatorsRegistrationBatchSizeFlag = &cli.IntFlag{
		Name:  "validators-registration-batch-size",
		Usage: "Sets the maximum size for one batch of validator registrations. Use a non-positive value to disable batching.",
		Value: 0,
	}
)

// DefaultValidatorDir returns OS-specific default validator directory.
func DefaultValidatorDir() string {
	// Try to place the data folder in the user's home dir
	home := file.HomeDir()
	if home != "" {
		if runtime.GOOS == "darwin" {
			return filepath.Join(home, "Library", "Eth2Validators")
		} else if runtime.GOOS == "windows" {
			return filepath.Join(home, "AppData", "Local", "Eth2Validators")
		} else {
			return filepath.Join(home, ".eth2validators")
		}
	}
	// As we cannot guess a stable location, return empty and handle later
	return ""
}
