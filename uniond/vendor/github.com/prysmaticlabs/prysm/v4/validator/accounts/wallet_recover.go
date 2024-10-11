package accounts

import (
	"context"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
)

const (
	phraseWordCount = 24
)

var (
	ErrIncorrectWordNumber = errors.New("incorrect number of words provided")
	ErrEmptyMnemonic       = errors.New("phrase cannot be empty")
)

// WalletRecover uses a menmonic seed phrase to recover a wallet into the path provided.
func (acm *CLIManager) WalletRecover(ctx context.Context) (*wallet.Wallet, error) {
	// Ensure that the wallet directory does not contain a wallet already
	dirExists, err := wallet.Exists(acm.walletDir)
	if err != nil {
		return nil, err
	}
	if dirExists {
		return nil, errors.New("a wallet already exists at this location. Please input an" +
			" alternative location for the new wallet or remove the current wallet")
	}
	w := wallet.New(&wallet.Config{
		WalletDir:      acm.walletDir,
		KeymanagerKind: keymanager.Derived,
		WalletPassword: acm.walletPassword,
	})
	if err := w.SaveWallet(); err != nil {
		return nil, errors.Wrap(err, "could not save wallet to disk")
	}
	km, err := derived.NewKeymanager(ctx, &derived.SetupConfig{
		Wallet:           w,
		ListenForChanges: false,
	})
	if err != nil {
		return nil, errors.Wrap(err, "could not make keymanager for given phrase")
	}
	if err := km.RecoverAccountsFromMnemonic(ctx, acm.mnemonic, acm.mnemonicLanguage, acm.mnemonic25thWord, acm.numAccounts); err != nil {
		return nil, err
	}
	log.WithField("wallet-path", w.AccountsDir()).Infof(
		"Successfully recovered HD wallet with %d accounts. Please use `accounts list` to view details for your accounts",
		acm.numAccounts,
	)
	return w, nil
}

// ValidateMnemonic ensures that it is not empty and that the count of the words are
// as specified(currently 24).
func ValidateMnemonic(mnemonic string) error {
	if strings.Trim(mnemonic, " ") == "" {
		return ErrEmptyMnemonic
	}
	words := strings.Split(mnemonic, " ")
	validWordCount := 0
	for _, word := range words {
		if strings.Trim(word, " ") == "" {
			continue
		}
		validWordCount += 1
	}
	if validWordCount != phraseWordCount {
		return errors.Wrapf(ErrIncorrectWordNumber, "phrase must be %d words, entered %d", phraseWordCount, validWordCount)
	}
	return nil
}
