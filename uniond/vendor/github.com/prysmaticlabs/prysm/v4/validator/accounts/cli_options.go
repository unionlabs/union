package accounts

import (
	"io"
	"time"

	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"google.golang.org/grpc"
)

// Option type for configuring the accounts cli manager.
type Option func(acc *CLIManager) error

// WithWallet provides a wallet to the accounts cli manager.
func WithWallet(wallet *wallet.Wallet) Option {
	return func(acc *CLIManager) error {
		acc.wallet = wallet
		return nil
	}
}

// WithKeymanager provides a keymanager to the accounts cli manager.
func WithKeymanager(km keymanager.IKeymanager) Option {
	return func(acc *CLIManager) error {
		acc.keymanager = km
		return nil
	}
}

// WithKeymanagerType provides a keymanager to the accounts cli manager.
func WithKeymanagerType(k keymanager.Kind) Option {
	return func(acc *CLIManager) error {
		acc.keymanagerKind = k
		return nil
	}
}

// WithShowDepositData enables displaying deposit data in the accounts cli manager.
func WithShowDepositData() Option {
	return func(acc *CLIManager) error {
		acc.showDepositData = true
		return nil
	}
}

// WithShowPrivateKeys enables displaying private keys in the accounts cli manager.
func WithShowPrivateKeys() Option {
	return func(acc *CLIManager) error {
		acc.showPrivateKeys = true
		return nil
	}
}

// WithListValidatorIndices enables displaying validator indices in the accounts cli manager.
func WithListValidatorIndices() Option {
	return func(acc *CLIManager) error {
		acc.listValidatorIndices = true
		return nil
	}
}

// WithGRPCDialOpts adds grpc opts needed to connect to beacon nodes in the accounts cli manager.
func WithGRPCDialOpts(opts []grpc.DialOption) Option {
	return func(acc *CLIManager) error {
		acc.dialOpts = opts
		return nil
	}
}

// WithGRPCHeaders adds grpc headers used when connecting to beacon nodes in the accounts cli manager.
func WithGRPCHeaders(headers []string) Option {
	return func(acc *CLIManager) error {
		acc.grpcHeaders = headers
		return nil
	}
}

// WithBeaconRPCProvider provides a beacon node endpoint to the accounts cli manager.
func WithBeaconRPCProvider(provider string) Option {
	return func(acc *CLIManager) error {
		acc.beaconRPCProvider = provider
		return nil
	}
}

// WithBeaconRESTApiProvider provides a beacon node REST API endpoint to the accounts cli manager.
func WithBeaconRESTApiProvider(beaconApiEndpoint string) Option {
	return func(acc *CLIManager) error {
		acc.beaconApiEndpoint = beaconApiEndpoint
		acc.beaconApiTimeout = time.Second * 30
		return nil
	}
}

// WithWalletKeyCount tracks the number of keys in a wallet.
func WithWalletKeyCount(walletKeyCount int) Option {
	return func(acc *CLIManager) error {
		acc.walletKeyCount = walletKeyCount
		return nil
	}
}

// WithDeletePublicKeys indicates whether to delete the public keys.
func WithDeletePublicKeys(deletePublicKeys bool) Option {
	return func(acc *CLIManager) error {
		acc.deletePublicKeys = deletePublicKeys
		return nil
	}
}

// WithReadPasswordFile indicates whether to read the password from a file.
func WithReadPasswordFile(readPasswordFile bool) Option {
	return func(acc *CLIManager) error {
		acc.readPasswordFile = readPasswordFile
		return nil
	}
}

// WithImportPrivateKeys indicates whether to import private keys as accounts.
func WithImportPrivateKeys(importPrivateKeys bool) Option {
	return func(acc *CLIManager) error {
		acc.importPrivateKeys = importPrivateKeys
		return nil
	}
}

// WithSkipMnemonicConfirm indicates whether to skip the mnemonic confirmation.
func WithSkipMnemonicConfirm(s bool) Option {
	return func(acc *CLIManager) error {
		acc.skipMnemonicConfirm = s
		return nil
	}
}

// WithMnemonicLanguage specifies the language used for the mnemonic passphrase.
func WithMnemonicLanguage(mnemonicLanguage string) Option {
	return func(acc *CLIManager) error {
		acc.mnemonicLanguage = mnemonicLanguage
		return nil
	}
}

// WithPrivateKeyFile specifies the private key path.
func WithPrivateKeyFile(privateKeyFile string) Option {
	return func(acc *CLIManager) error {
		acc.privateKeyFile = privateKeyFile
		return nil
	}
}

// WithKeysDir specifies the directory keys are read from.
func WithKeysDir(keysDir string) Option {
	return func(acc *CLIManager) error {
		acc.keysDir = keysDir
		return nil
	}
}

// WithPasswordFilePath specifies where the password is stored.
func WithPasswordFilePath(passwordFilePath string) Option {
	return func(acc *CLIManager) error {
		acc.passwordFilePath = passwordFilePath
		return nil
	}
}

// WithBackupsDir specifies the directory backups are written to.
func WithBackupsDir(backupsDir string) Option {
	return func(acc *CLIManager) error {
		acc.backupsDir = backupsDir
		return nil
	}
}

// WithBackupsPassword specifies the password for backups.
func WithBackupsPassword(backupsPassword string) Option {
	return func(acc *CLIManager) error {
		acc.backupsPassword = backupsPassword
		return nil
	}
}

// WithFilteredPubKeys adds public key strings parsed from CLI.
func WithFilteredPubKeys(filteredPubKeys []bls.PublicKey) Option {
	return func(acc *CLIManager) error {
		acc.filteredPubKeys = filteredPubKeys
		return nil
	}
}

// WithRawPubKeys adds raw public key bytes parsed from CLI.
func WithRawPubKeys(rawPubKeys [][]byte) Option {
	return func(acc *CLIManager) error {
		acc.rawPubKeys = rawPubKeys
		return nil
	}
}

// WithFormattedPubKeys adds formatted public key strings parsed from CLI.
func WithFormattedPubKeys(formattedPubKeys []string) Option {
	return func(acc *CLIManager) error {
		acc.formattedPubKeys = formattedPubKeys
		return nil
	}
}

func WithExitJSONOutputPath(outputPath string) Option {
	return func(acc *CLIManager) error {
		acc.exitJSONOutputPath = outputPath
		return nil
	}
}

// WithWalletDir specifies the password for backups.
func WithWalletDir(walletDir string) Option {
	return func(acc *CLIManager) error {
		acc.walletDir = walletDir
		return nil
	}
}

// WithWalletPassword specifies the password for backups.
func WithWalletPassword(walletPassword string) Option {
	return func(acc *CLIManager) error {
		acc.walletPassword = walletPassword
		return nil
	}
}

// WithMnemonic specifies the password for backups.
func WithMnemonic(mnemonic string) Option {
	return func(acc *CLIManager) error {
		acc.mnemonic = mnemonic
		return nil
	}
}

// WithMnemonic25thWord specifies the password for backups.
func WithMnemonic25thWord(mnemonic25thWord string) Option {
	return func(acc *CLIManager) error {
		acc.mnemonic25thWord = mnemonic25thWord
		return nil
	}
}

// WithNumAccounts specifies the number of accounts.
func WithNumAccounts(numAccounts int) Option {
	return func(acc *CLIManager) error {
		acc.numAccounts = numAccounts
		return nil
	}
}

// WithCustomReader changes the default reader
func WithCustomReader(reader io.Reader) Option {
	return func(acc *CLIManager) error {
		acc.inputReader = reader
		return nil
	}
}
