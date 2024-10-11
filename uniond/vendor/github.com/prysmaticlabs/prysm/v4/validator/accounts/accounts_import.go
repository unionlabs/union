package accounts

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/google/uuid"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

var derivationPathRegex = regexp.MustCompile(`m_12381_3600_(\d+)_(\d+)_(\d+)`)

// byDerivationPath implements sort.Interface based on a
// derivation path present in a keystore filename, if any. This
// will allow us to sort filenames such as keystore-m_12381_3600_1_0_0.json
// in a directory and import them nicely in order of the derivation path.
type byDerivationPath []string

// Len is the number of elements in the collection.
func (fileNames byDerivationPath) Len() int { return len(fileNames) }

// Less reports whether the element with index i must sort before the element with index j.
func (fileNames byDerivationPath) Less(i, j int) bool {
	// We check if file name at index i has a derivation path
	// in the filename. If it does not, then it is not less than j, and
	// we should swap it towards the end of the sorted list.
	if !derivationPathRegex.MatchString(fileNames[i]) {
		return false
	}
	derivationPathA := derivationPathRegex.FindString(fileNames[i])
	derivationPathB := derivationPathRegex.FindString(fileNames[j])
	if derivationPathA == "" {
		return false
	}
	if derivationPathB == "" {
		return true
	}
	a, err := strconv.Atoi(accountIndexFromFileName(derivationPathA))
	if err != nil {
		return false
	}
	b, err := strconv.Atoi(accountIndexFromFileName(derivationPathB))
	if err != nil {
		return false
	}
	return a < b
}

// Swap swaps the elements with indexes i and j.
func (fileNames byDerivationPath) Swap(i, j int) {
	fileNames[i], fileNames[j] = fileNames[j], fileNames[i]
}

// ImportAccountsConfig defines values to run the import accounts function.
type ImportAccountsConfig struct {
	Keystores       []*keymanager.Keystore
	Importer        keymanager.Importer
	AccountPassword string
}

// Import can import external, EIP-2335 compliant keystore.json files as
// new accounts into the Prysm validator wallet. This uses the CLI to extract
// values necessary to run the function.
func (acm *CLIManager) Import(ctx context.Context) error {
	k, ok := acm.keymanager.(keymanager.Importer)
	if !ok {
		return errors.New("keymanager cannot import keystores")
	}
	log.Info("importing validator keystores...")
	// Check if the user wishes to import a one-off, private key directly
	// as an account into the Prysm validator.
	if acm.importPrivateKeys {
		return importPrivateKeyAsAccount(ctx, acm.wallet, k, acm.privateKeyFile)
	}

	keystoresImported, err := processDirectory(ctx, acm.keysDir, 0)
	if err != nil {
		return errors.Wrap(err, "unable to process directory and import keys")
	}

	var accountsPassword string
	if acm.readPasswordFile {
		data, err := os.ReadFile(acm.passwordFilePath) // #nosec G304
		if err != nil {
			return err
		}
		accountsPassword = string(data)
	} else {
		accountsPassword, err = prompt.PasswordPrompt(
			"Enter the password for your imported accounts", prompt.NotEmpty,
		)
		if err != nil {
			return fmt.Errorf("could not read account password: %w", err)
		}
	}
	fmt.Println("Importing accounts, this may take a while...")
	statuses, err := ImportAccounts(ctx, &ImportAccountsConfig{
		Importer:        k,
		Keystores:       keystoresImported,
		AccountPassword: accountsPassword,
	})
	if err != nil {
		return err
	}
	var successfullyImportedAccounts []string
	for i, status := range statuses {
		switch status.Status {
		case keymanager.StatusImported:
			successfullyImportedAccounts = append(successfullyImportedAccounts, keystoresImported[i].Pubkey)
		case keymanager.StatusDuplicate:
			log.Warnf("Duplicate key %s found in import request, skipped", keystoresImported[i].Pubkey)
		case keymanager.StatusError:
			log.Warnf("Could not import keystore for %s: %s", keystoresImported[i].Pubkey, status.Message)
		}
	}
	if len(successfullyImportedAccounts) == 0 {
		log.Error("no accounts were successfully imported")
	} else {
		log.Infof(
			"Imported accounts %v, view all of them by running `accounts list`",
			successfullyImportedAccounts,
		)
	}

	return nil
}

// Recursive function to process directories and files.
func processDirectory(ctx context.Context, dir string, depth int) ([]*keymanager.Keystore, error) {
	maxdepth := 2
	if depth > maxdepth {
		log.Infof("stopped checking folders for keystores after max depth of %d was reached", maxdepth)
		return nil, nil // Stop recursion after two levels.
	}
	log.Infof("checking directory for keystores: %s", dir)
	isDir, err := file.HasDir(dir)
	if err != nil {
		return nil, errors.Wrap(err, "could not determine if path is a directory")
	}

	keystoresImported := make([]*keymanager.Keystore, 0)

	if isDir {
		files, err := os.ReadDir(dir)
		if err != nil {
			return nil, errors.Wrap(err, "could not read dir")
		}
		if len(files) == 0 {
			return nil, fmt.Errorf("directory %s has no files, cannot import from it", dir)
		}
		for _, f := range files {
			fullPath := filepath.Join(dir, f.Name())
			if f.IsDir() {
				subKeystores, err := processDirectory(ctx, fullPath, depth+1)
				if err != nil {
					return nil, err
				}
				keystoresImported = append(keystoresImported, subKeystores...)
			} else {
				keystore, err := readKeystoreFile(ctx, fullPath)
				if err != nil {
					if strings.Contains(err.Error(), "could not decode keystore json") {
						continue
					}
					return nil, errors.Wrapf(err, "could not import keystore at path: %s", fullPath)
				}
				keystoresImported = append(keystoresImported, keystore)
			}
		}
	} else {
		keystore, err := readKeystoreFile(ctx, dir)
		if err != nil {
			return nil, errors.Wrap(err, "could not import keystore")
		}
		keystoresImported = append(keystoresImported, keystore)
	}

	return keystoresImported, nil
}

// ImportAccounts can import external, EIP-2335 compliant keystore.json files as
// new accounts into the Prysm validator wallet.
func ImportAccounts(ctx context.Context, cfg *ImportAccountsConfig) ([]*keymanager.KeyStatus, error) {
	if cfg.AccountPassword == "" {
		statuses := make([]*keymanager.KeyStatus, len(cfg.Keystores))
		for i, keystore := range cfg.Keystores {
			statuses[i] = &keymanager.KeyStatus{
				Status: keymanager.StatusError,
				Message: fmt.Sprintf(
					"account password is required to import keystore %s",
					keystore.Pubkey,
				),
			}
		}
		return statuses, nil
	}
	passwords := make([]string, len(cfg.Keystores))
	for i := 0; i < len(cfg.Keystores); i++ {
		passwords[i] = cfg.AccountPassword
	}
	return cfg.Importer.ImportKeystores(
		ctx,
		cfg.Keystores,
		passwords,
	)
}

// Imports a one-off file containing a private key as a hex string into
// the Prysm validator's accounts.
func importPrivateKeyAsAccount(ctx context.Context, wallet *wallet.Wallet, importer keymanager.Importer, privKeyFile string) error {
	fullPath, err := file.ExpandPath(privKeyFile)
	if err != nil {
		return errors.Wrapf(err, "could not expand file path for %s", privKeyFile)
	}
	if !file.Exists(fullPath) {
		return fmt.Errorf("file %s does not exist", fullPath)
	}
	privKeyHex, err := os.ReadFile(fullPath) // #nosec G304
	if err != nil {
		return errors.Wrapf(err, "could not read private key file at path %s", fullPath)
	}
	privKeyString := string(privKeyHex)
	if len(privKeyString) > 2 && strings.Contains(privKeyString, "0x") {
		privKeyString = privKeyString[2:] // Strip the 0x prefix, if any.
	}
	privKeyBytes, err := hex.DecodeString(strings.TrimRight(privKeyString, "\r\n"))
	if err != nil {
		return errors.Wrap(
			err, "could not decode file as hex string, does the file contain a valid hex string?",
		)
	}
	privKey, err := bls.SecretKeyFromBytes(privKeyBytes)
	if err != nil {
		return errors.Wrap(err, "not a valid BLS private key")
	}
	keystore, err := createKeystoreFromPrivateKey(privKey, wallet.Password())
	if err != nil {
		return errors.Wrap(err, "could not encrypt private key into a keystore file")
	}
	statuses, err := ImportAccounts(
		ctx,
		&ImportAccountsConfig{
			Importer:        importer,
			AccountPassword: wallet.Password(),
			Keystores:       []*keymanager.Keystore{keystore},
		},
	)
	if err != nil {
		return errors.Wrap(err, "could not import keystore into wallet")
	}
	for _, status := range statuses {
		switch status.Status {
		case keymanager.StatusImported:
			fmt.Printf(
				"Imported account with public key %#x, view all accounts by running `accounts list`\n",
				au.BrightMagenta(bytesutil.Trunc(privKey.PublicKey().Marshal())),
			)
			return nil
		case keymanager.StatusError:
			return fmt.Errorf("could not import keystore for %s: %s", keystore.Pubkey, status.Message)
		case keymanager.StatusDuplicate:
			return fmt.Errorf("duplicate key %s skipped", keystore.Pubkey)
		}
	}

	return nil
}

func readKeystoreFile(_ context.Context, keystoreFilePath string) (*keymanager.Keystore, error) {
	keystoreBytes, err := os.ReadFile(keystoreFilePath) // #nosec G304
	if err != nil {
		return nil, errors.Wrap(err, "could not read keystore file")
	}
	keystoreFile := &keymanager.Keystore{}
	if err := json.Unmarshal(keystoreBytes, keystoreFile); err != nil {
		return nil, errors.Wrap(err, "could not decode keystore json")
	}
	if keystoreFile.Pubkey == "" {
		return nil, errors.New("could not decode keystore json")
	}
	if keystoreFile.Description == "" && keystoreFile.Name != "" {
		keystoreFile.Description = keystoreFile.Name
	}
	return keystoreFile, nil
}

func createKeystoreFromPrivateKey(privKey bls.SecretKey, walletPassword string) (*keymanager.Keystore, error) {
	encryptor := keystorev4.New()
	id, err := uuid.NewRandom()
	if err != nil {
		return nil, err
	}
	cryptoFields, err := encryptor.Encrypt(privKey.Marshal(), walletPassword)
	if err != nil {
		return nil, errors.Wrapf(
			err,
			"could not encrypt private key with public key %#x",
			privKey.PublicKey().Marshal(),
		)
	}
	return &keymanager.Keystore{
		Crypto:      cryptoFields,
		ID:          id.String(),
		Version:     encryptor.Version(),
		Pubkey:      fmt.Sprintf("%x", privKey.PublicKey().Marshal()),
		Description: encryptor.Name(),
	}, nil
}

// Extracts the account index, j, from a derivation path in a file name
// with the format m_12381_3600_j_0_0.
func accountIndexFromFileName(derivationPath string) string {
	derivationPath = derivationPath[13:]
	accIndexEnd := strings.Index(derivationPath, "_")
	return derivationPath[:accIndexEnd]
}
