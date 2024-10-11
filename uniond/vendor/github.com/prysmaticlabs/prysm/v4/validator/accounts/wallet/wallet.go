package wallet

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	accountsprompt "github.com/prysmaticlabs/prysm/v4/validator/accounts/userprompt"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	remoteweb3signer "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

const (
	// KeymanagerConfigFileName for the keymanager used by the wallet: imported, derived, remote, or web3signer.
	KeymanagerConfigFileName = "keymanageropts.json"
	// NewWalletPasswordPromptText for wallet creation.
	NewWalletPasswordPromptText = "New wallet password"
	// PasswordPromptText for wallet unlocking.
	PasswordPromptText = "Wallet password"
	// ConfirmPasswordPromptText for confirming a wallet password.
	ConfirmPasswordPromptText = "Confirm password"
	// DefaultWalletPasswordFile used to store a wallet password with appropriate permissions
	// if a user signs up via the Prysm web UI via RPC.
	DefaultWalletPasswordFile = "walletpassword.txt"
	// CheckExistsErrMsg for when there is an error while checking for a wallet
	CheckExistsErrMsg = "could not check if wallet exists"
	// CheckValidityErrMsg for when there is an error while checking wallet validity
	CheckValidityErrMsg = "could not check if wallet is valid"
	// InvalidWalletErrMsg for when a directory does not contain a valid wallet
	InvalidWalletErrMsg = "directory does not contain valid wallet"
)

var (
	// ErrNoWalletFound signifies there was no wallet directory found on-disk.
	ErrNoWalletFound = errors.New(
		"no wallet found. You can create a new wallet with `validator wallet create`. " +
			"If you already did, perhaps you created a wallet in a custom directory, which you can specify using " +
			"`--wallet-dir=/path/to/my/wallet`",
	)
	// KeymanagerKindSelections as friendly text.
	KeymanagerKindSelections = map[keymanager.Kind]string{
		keymanager.Local:      "Imported Wallet (Recommended)",
		keymanager.Derived:    "HD Wallet",
		keymanager.Web3Signer: "Consensys Web3Signer (Advanced)",
	}
	// ValidateExistingPass checks that an input cannot be empty.
	ValidateExistingPass = func(input string) error {
		if input == "" {
			return errors.New("password input cannot be empty")
		}
		return nil
	}
)

// Config to open a wallet programmatically.
type Config struct {
	WalletDir      string
	KeymanagerKind keymanager.Kind
	WalletPassword string
}

// Wallet is a primitive in Prysm's account management which
// has the capability of creating new accounts, reading existing accounts,
// and providing secure access to Ethereum proof of stake secrets depending on an
// associated keymanager (either imported, derived, or remote signing enabled).
type Wallet struct {
	walletDir      string
	accountsPath   string
	configFilePath string
	walletPassword string
	keymanagerKind keymanager.Kind
}

// New creates a struct from config values.
func New(cfg *Config) *Wallet {
	accountsPath := filepath.Join(cfg.WalletDir, cfg.KeymanagerKind.String())
	return &Wallet{
		walletDir:      cfg.WalletDir,
		accountsPath:   accountsPath,
		keymanagerKind: cfg.KeymanagerKind,
		walletPassword: cfg.WalletPassword,
	}
}

// Exists checks if directory at walletDir exists
func Exists(walletDir string) (bool, error) {
	dirExists, err := file.HasDir(walletDir)
	if err != nil {
		return false, errors.Wrap(err, "could not parse wallet directory")
	}
	isValid, err := IsValid(walletDir)
	if errors.Is(err, ErrNoWalletFound) {
		return false, nil
	} else if err != nil {
		return false, errors.Wrap(err, "could not check if dir is valid")
	}
	return dirExists && isValid, nil
}

// IsValid checks if a folder contains a single key directory such as `derived`, `remote` or `imported`.
// Returns true if one of those subdirectories exist, false otherwise.
func IsValid(walletDir string) (bool, error) {
	expanded, err := file.ExpandPath(walletDir)
	if err != nil {
		return false, err
	}
	f, err := os.Open(expanded) // #nosec G304
	if err != nil {
		if strings.Contains(err.Error(), "no such file") ||
			strings.Contains(err.Error(), "cannot find the file") ||
			strings.Contains(err.Error(), "cannot find the path") {
			return false, nil
		}
		return false, err
	}
	defer func() {
		if err := f.Close(); err != nil {
			log.Debugf("Could not close directory: %s", expanded)
		}
	}()
	names, err := f.Readdirnames(-1)
	if err != nil {
		return false, err
	}

	if len(names) == 0 {
		return false, ErrNoWalletFound
	}

	// Count how many wallet types we have in the directory
	numWalletTypes := 0
	for _, name := range names {
		// Nil error means input name is `derived` or `imported`
		_, err = keymanager.ParseKind(name)
		if err == nil {
			numWalletTypes++
		}
	}
	return numWalletTypes == 1, nil
}

// OpenWalletOrElseCli tries to open the wallet and if it fails or no wallet
// is found, invokes a callback function.
func OpenWalletOrElseCli(cliCtx *cli.Context, otherwise func(cliCtx *cli.Context) (*Wallet, error)) (*Wallet, error) {
	exists, err := Exists(cliCtx.String(flags.WalletDirFlag.Name))
	if err != nil {
		return nil, errors.Wrap(err, CheckExistsErrMsg)
	}
	if !exists {
		return otherwise(cliCtx)
	}
	isValid, err := IsValid(cliCtx.String(flags.WalletDirFlag.Name))
	if errors.Is(err, ErrNoWalletFound) {
		// reprompts the user for a valid dir
		return otherwise(cliCtx)
	}
	if err != nil {
		return nil, errors.Wrap(err, CheckValidityErrMsg)
	}
	if !isValid {
		return nil, errors.New(InvalidWalletErrMsg)
	}

	walletDir, err := accountsprompt.InputDirectory(cliCtx, accountsprompt.WalletDirPromptText, flags.WalletDirFlag)
	if err != nil {
		return nil, err
	}
	walletPassword, err := InputPassword(
		cliCtx,
		flags.WalletPasswordFileFlag,
		PasswordPromptText,
		false, /* Do not confirm password */
		ValidateExistingPass,
	)
	if err != nil {
		return nil, err
	}
	return OpenWallet(cliCtx.Context, &Config{
		WalletDir:      walletDir,
		WalletPassword: walletPassword,
	})
}

// OpenOrCreateNewWallet takes a cli and returns a wallet either opening an existing valid wallet or creating a new one.
func OpenOrCreateNewWallet(cliCtx *cli.Context) (*Wallet, error) {
	walletDir, err := accountsprompt.InputDirectory(cliCtx, accountsprompt.WalletDirPromptText, flags.WalletDirFlag)
	if err != nil {
		return nil, err
	}
	exists, err := Exists(walletDir)
	if err != nil {
		return nil, errors.Wrap(err, CheckExistsErrMsg)
	}
	if exists {
		isValid, err := IsValid(walletDir)
		if err != nil {
			return nil, errors.Wrap(err, CheckValidityErrMsg)
		}
		if !isValid {
			return nil, errors.New(InvalidWalletErrMsg)
		}
		walletPassword, err := InputPassword(
			cliCtx,
			flags.WalletPasswordFileFlag,
			PasswordPromptText,
			false, /* Do not confirm password */
			ValidateExistingPass,
		)
		if err != nil {
			return nil, err
		}
		return OpenWallet(cliCtx.Context, &Config{
			WalletDir:      walletDir,
			WalletPassword: walletPassword,
		})
	}
	// create a new wallet in the dir
	walletPassword, err := prompt.InputPassword(
		cliCtx,
		flags.WalletPasswordFileFlag,
		NewWalletPasswordPromptText,
		ConfirmPasswordPromptText,
		true, /* Should confirm password */
		prompt.ValidatePasswordInput,
	)
	if err != nil {
		return nil, err
	}
	w := New(&Config{
		KeymanagerKind: keymanager.Local,
		WalletDir:      walletDir,
		WalletPassword: walletPassword,
	})
	if err := w.SaveWallet(); err != nil {
		return nil, errors.Wrap(err, "could not save wallet to disk")
	}
	log.WithField("wallet-path", walletDir).Info(
		"Successfully created new wallet",
	)
	return w, nil
}

// NewWalletForWeb3Signer returns a new wallet for web3 signer which is temporary and not stored locally.
func NewWalletForWeb3Signer() *Wallet {
	// wallet is just a temporary wallet for web3 signer used to call initialize keymanager.
	return &Wallet{
		walletDir:      "",
		accountsPath:   "",
		keymanagerKind: keymanager.Web3Signer,
		walletPassword: "",
	}
}

// OpenWallet instantiates a wallet from a specified path. It checks the
// type of keymanager associated with the wallet by reading files in the wallet
// path, if applicable. If a wallet does not exist, returns an appropriate error.
func OpenWallet(_ context.Context, cfg *Config) (*Wallet, error) {
	exists, err := Exists(cfg.WalletDir)
	if err != nil {
		return nil, errors.Wrap(err, CheckExistsErrMsg)
	}
	if !exists {
		return nil, ErrNoWalletFound
	}
	valid, err := IsValid(cfg.WalletDir)
	// ErrNoWalletFound represents both a directory that does not exist as well as an empty directory
	if errors.Is(err, ErrNoWalletFound) {
		return nil, ErrNoWalletFound
	}
	if err != nil {
		return nil, errors.Wrap(err, CheckValidityErrMsg)
	}
	if !valid {
		return nil, errors.New(InvalidWalletErrMsg)
	}

	keymanagerKind, err := readKeymanagerKindFromWalletPath(cfg.WalletDir)
	if err != nil {
		return nil, errors.Wrap(err, "could not read keymanager kind for wallet")
	}
	accountsPath := filepath.Join(cfg.WalletDir, keymanagerKind.String())
	return &Wallet{
		walletDir:      cfg.WalletDir,
		accountsPath:   accountsPath,
		keymanagerKind: keymanagerKind,
		walletPassword: cfg.WalletPassword,
	}, nil
}

// SaveWallet persists the wallet's directories to disk.
func (w *Wallet) SaveWallet() error {
	if err := file.MkdirAll(w.accountsPath); err != nil {
		return errors.Wrap(err, "could not create wallet directory")
	}
	return nil
}

// KeymanagerKind used by the wallet.
func (w *Wallet) KeymanagerKind() keymanager.Kind {
	return w.keymanagerKind
}

// AccountsDir for the wallet.
func (w *Wallet) AccountsDir() string {
	return w.accountsPath
}

// Password for the wallet.
func (w *Wallet) Password() string {
	return w.walletPassword
}

// InitializeKeymanager reads a keymanager config from disk at the wallet path,
// unmarshals it based on the wallet's keymanager kind, and returns its value.
func (w *Wallet) InitializeKeymanager(ctx context.Context, cfg iface.InitKeymanagerConfig) (keymanager.IKeymanager, error) {
	var km keymanager.IKeymanager
	var err error
	switch w.KeymanagerKind() {
	case keymanager.Local:
		km, err = local.NewKeymanager(ctx, &local.SetupConfig{
			Wallet:           w,
			ListenForChanges: cfg.ListenForChanges,
		})
		if err != nil {
			return nil, errors.Wrap(err, "could not initialize imported keymanager")
		}
	case keymanager.Derived:
		km, err = derived.NewKeymanager(ctx, &derived.SetupConfig{
			Wallet:           w,
			ListenForChanges: cfg.ListenForChanges,
		})
		if err != nil {
			return nil, errors.Wrap(err, "could not initialize derived keymanager")
		}
	case keymanager.Web3Signer:
		config := cfg.Web3SignerConfig
		if config == nil {
			return nil, errors.New("web3signer config is nil")
		}
		// TODO(9883): future work needs to address how initialize keymanager is called for web3signer.
		// an error may be thrown for genesis validators root for some InitializeKeymanager calls.
		if !bytesutil.IsValidRoot(config.GenesisValidatorsRoot) {
			return nil, errors.New("web3signer requires a genesis validators root value")
		}
		km, err = remoteweb3signer.NewKeymanager(ctx, config)
		if err != nil {
			return nil, errors.Wrap(err, "could not initialize web3signer keymanager")
		}
	default:
		return nil, fmt.Errorf("keymanager kind not supported: %s", w.keymanagerKind)
	}
	return km, nil
}

// WriteFileAtPath within the wallet directory given the desired path, filename, and raw data.
func (w *Wallet) WriteFileAtPath(_ context.Context, filePath, fileName string, data []byte) (bool /* exited previously */, error) {
	accountPath := filepath.Join(w.accountsPath, filePath)
	hasDir, err := file.HasDir(accountPath)
	if err != nil {
		return false, err
	}
	if !hasDir {
		if err := file.MkdirAll(accountPath); err != nil {
			return false, errors.Wrapf(err, "could not create path: %s", accountPath)
		}
	}
	fullPath := filepath.Join(accountPath, fileName)
	existedPreviously := file.Exists(fullPath)
	if err := file.WriteFile(fullPath, data); err != nil {
		return false, errors.Wrapf(err, "could not write %s", filePath)
	}
	log.WithFields(logrus.Fields{
		"path":     fullPath,
		"fileName": fileName,
	}).Debug("Wrote new file at path")
	return existedPreviously, nil
}

// ReadFileAtPath within the wallet directory given the desired path and filename.
func (w *Wallet) ReadFileAtPath(_ context.Context, filePath, fileName string) ([]byte, error) {
	accountPath := filepath.Join(w.accountsPath, filePath)
	hasDir, err := file.HasDir(accountPath)
	if err != nil {
		return nil, err
	}
	if !hasDir {
		if err := file.MkdirAll(accountPath); err != nil {
			return nil, errors.Wrapf(err, "could not create path: %s", accountPath)
		}
	}
	fullPath := filepath.Join(accountPath, fileName)
	matches, err := filepath.Glob(fullPath)
	if err != nil {
		return []byte{}, errors.Wrap(err, "could not find file")
	}
	if len(matches) == 0 {
		return []byte{}, fmt.Errorf("no files found in path: %s", fullPath)
	}
	rawData, err := os.ReadFile(matches[0])
	if err != nil {
		return nil, errors.Wrapf(err, "could not read path: %s", filePath)
	}
	return rawData, nil
}

// FileNameAtPath return the full file name for the requested file. It allows for finding the file
// with a regex pattern.
func (w *Wallet) FileNameAtPath(_ context.Context, filePath, fileName string) (string, error) {
	accountPath := filepath.Join(w.accountsPath, filePath)
	if err := file.MkdirAll(accountPath); err != nil {
		return "", errors.Wrapf(err, "could not create path: %s", accountPath)
	}
	fullPath := filepath.Join(accountPath, fileName)
	matches, err := filepath.Glob(fullPath)
	if err != nil {
		return "", errors.Wrap(err, "could not find file")
	}
	if len(matches) == 0 {
		return "", fmt.Errorf("no files found in path: %s", fullPath)
	}
	fullFileName := filepath.Base(matches[0])
	return fullFileName, nil
}

// ReadKeymanagerConfigFromDisk opens a keymanager config file
// for reading if it exists at the wallet path.
func (w *Wallet) ReadKeymanagerConfigFromDisk(_ context.Context) (io.ReadCloser, error) {
	configFilePath := filepath.Join(w.accountsPath, KeymanagerConfigFileName)
	if !file.Exists(configFilePath) {
		return nil, fmt.Errorf("no keymanager config file found at path: %s", w.accountsPath)
	}
	w.configFilePath = configFilePath
	return os.Open(configFilePath) // #nosec G304
}

// WriteKeymanagerConfigToDisk takes an encoded keymanager config file
// and writes it to the wallet path.
func (w *Wallet) WriteKeymanagerConfigToDisk(_ context.Context, encoded []byte) error {
	configFilePath := filepath.Join(w.accountsPath, KeymanagerConfigFileName)
	// Write the config file to disk.
	if err := file.WriteFile(configFilePath, encoded); err != nil {
		return errors.Wrapf(err, "could not write config to path: %s", configFilePath)
	}
	log.WithField("configFilePath", configFilePath).Debug("Wrote keymanager config file to disk")
	return nil
}

func readKeymanagerKindFromWalletPath(walletPath string) (keymanager.Kind, error) {
	walletItem, err := os.Open(walletPath) // #nosec G304
	if err != nil {
		return 0, err
	}
	defer func() {
		if err := walletItem.Close(); err != nil {
			log.WithField(
				"path", walletPath,
			).Errorf("Could not close wallet directory: %v", err)
		}
	}()
	list, err := walletItem.Readdirnames(0) // 0 to read all files and folders.
	if err != nil {
		return 0, fmt.Errorf("could not read files in directory: %s", walletPath)
	}
	for _, n := range list {
		keymanagerKind, err := keymanager.ParseKind(n)
		if err == nil {
			return keymanagerKind, nil
		}
	}
	return 0, errors.New("no keymanager folder (imported, remote, derived) found in wallet path")
}

// InputPassword prompts for a password and optionally for password confirmation.
// The password is validated according to custom rules.
func InputPassword(
	cliCtx *cli.Context,
	passwordFileFlag *cli.StringFlag,
	promptText string,
	confirmPassword bool,
	passwordValidator func(input string) error,
) (string, error) {
	if cliCtx.IsSet(passwordFileFlag.Name) {
		passwordFilePathInput := cliCtx.String(passwordFileFlag.Name)
		data, err := file.ReadFileAsBytes(passwordFilePathInput)
		if err != nil {
			return "", errors.Wrap(err, "could not read file as bytes")
		}
		enteredPassword := strings.TrimRight(string(data), "\r\n")
		if err := passwordValidator(enteredPassword); err != nil {
			return "", errors.Wrap(err, "password did not pass validation")
		}
		return enteredPassword, nil
	}
	var hasValidPassword bool
	var walletPassword string
	var err error
	for !hasValidPassword {
		walletPassword, err = prompt.PasswordPrompt(promptText, passwordValidator)
		if err != nil {
			return "", fmt.Errorf("could not read account password: %w", err)
		}

		if confirmPassword {
			passwordConfirmation, err := prompt.PasswordPrompt(ConfirmPasswordPromptText, passwordValidator)
			if err != nil {
				return "", fmt.Errorf("could not read password confirmation: %w", err)
			}
			if walletPassword != passwordConfirmation {
				log.Error("Passwords do not match")
				continue
			}
			hasValidPassword = true
		} else {
			return walletPassword, nil
		}
	}
	return walletPassword, nil
}
