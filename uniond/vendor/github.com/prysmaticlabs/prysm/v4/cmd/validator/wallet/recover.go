package wallet

import (
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/userprompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/tyler-smith/go-bip39"
	"github.com/tyler-smith/go-bip39/wordlists"
	"github.com/urfave/cli/v2"
)

const (
	// #nosec G101 -- Not sensitive data
	mnemonicPassphraseYesNoText = "(Advanced) Do you have an optional '25th word' passphrase for your mnemonic? [y/n]"
	// #nosec G101 -- Not sensitive data
	mnemonicPassphrasePromptText = "(Advanced) Enter the '25th word' passphrase for your mnemonic"
)

func walletRecover(c *cli.Context) error {
	mnemonic, err := inputMnemonic(c)
	if err != nil {
		return errors.Wrap(err, "could not get mnemonic phrase")
	}
	opts := []accounts.Option{
		accounts.WithMnemonic(mnemonic),
	}

	skipMnemonic25thWord := c.IsSet(flags.SkipMnemonic25thWordCheckFlag.Name)
	has25thWordFile := c.IsSet(flags.Mnemonic25thWordFileFlag.Name)
	if !skipMnemonic25thWord && !has25thWordFile {
		resp, err := prompt.ValidatePrompt(
			os.Stdin, mnemonicPassphraseYesNoText, prompt.ValidateYesOrNo,
		)
		if err != nil {
			return errors.Wrap(err, "could not validate choice")
		}
		if strings.EqualFold(resp, "y") {
			mnemonicPassphrase, err := prompt.InputPassword(
				c,
				flags.Mnemonic25thWordFileFlag,
				mnemonicPassphrasePromptText,
				"Confirm mnemonic passphrase",
				false, /* Should confirm password */
				func(input string) error {
					if strings.TrimSpace(input) == "" {
						return errors.New("input cannot be empty")
					}
					return nil
				},
			)
			if err != nil {
				return err
			}
			opts = append(opts, accounts.WithMnemonic25thWord(mnemonicPassphrase))
		}
	}
	walletDir, err := userprompt.InputDirectory(c, userprompt.WalletDirPromptText, flags.WalletDirFlag)
	if err != nil {
		return err
	}
	walletPassword, err := prompt.InputPassword(
		c,
		flags.WalletPasswordFileFlag,
		wallet.NewWalletPasswordPromptText,
		wallet.ConfirmPasswordPromptText,
		true, /* Should confirm password */
		prompt.ValidatePasswordInput,
	)
	if err != nil {
		return err
	}
	numAccounts, err := inputNumAccounts(c)
	if err != nil {
		return errors.Wrap(err, "could not get number of accounts to recover")
	}
	opts = append(opts, accounts.WithWalletDir(walletDir))
	opts = append(opts, accounts.WithWalletPassword(walletPassword))
	opts = append(opts, accounts.WithNumAccounts(int(numAccounts)))

	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		return err
	}
	if _, err = acc.WalletRecover(c.Context); err != nil {
		return err
	}
	log.Infof(
		"Successfully recovered HD wallet with accounts and saved configuration to disk",
	)
	return nil
}

func inputMnemonic(cliCtx *cli.Context) (mnemonicPhrase string, err error) {
	if cliCtx.IsSet(flags.MnemonicFileFlag.Name) {
		mnemonicFilePath := cliCtx.String(flags.MnemonicFileFlag.Name)
		data, err := os.ReadFile(mnemonicFilePath) // #nosec G304 -- ReadFile is safe
		if err != nil {
			return "", err
		}
		enteredMnemonic := string(data)
		if err := accounts.ValidateMnemonic(enteredMnemonic); err != nil {
			return "", errors.Wrap(err, "mnemonic phrase did not pass validation")
		}
		return enteredMnemonic, nil
	}
	allowedLanguages := map[string][]string{
		"chinese_simplified":  wordlists.ChineseSimplified,
		"chinese_traditional": wordlists.ChineseTraditional,
		"czech":               wordlists.Czech,
		"english":             wordlists.English,
		"french":              wordlists.French,
		"japanese":            wordlists.Japanese,
		"korean":              wordlists.Korean,
		"italian":             wordlists.Italian,
		"spanish":             wordlists.Spanish,
	}
	languages := make([]string, 0)
	for k := range allowedLanguages {
		languages = append(languages, k)
	}
	sort.Strings(languages)
	selectedLanguage, err := prompt.ValidatePrompt(
		os.Stdin,
		fmt.Sprintf("Enter the language of your seed phrase: %s", strings.Join(languages, ", ")),
		func(input string) error {
			if _, ok := allowedLanguages[input]; !ok {
				return errors.New("input not in the list of allowed languages")
			}
			return nil
		},
	)
	if err != nil {
		return "", fmt.Errorf("could not get mnemonic language: %w", err)
	}
	bip39.SetWordList(allowedLanguages[selectedLanguage])
	mnemonicPhrase, err = prompt.ValidatePrompt(
		os.Stdin,
		"Enter the seed phrase for the wallet you would like to recover",
		accounts.ValidateMnemonic)
	if err != nil {
		return "", fmt.Errorf("could not get mnemonic phrase: %w", err)
	}
	return mnemonicPhrase, nil
}

func inputNumAccounts(cliCtx *cli.Context) (int64, error) {
	if cliCtx.IsSet(flags.NumAccountsFlag.Name) {
		numAccounts := cliCtx.Int64(flags.NumAccountsFlag.Name)
		if numAccounts <= 0 {
			return 0, errors.New("must recover at least 1 account")
		}
		return numAccounts, nil
	}
	numAccounts, err := prompt.ValidatePrompt(os.Stdin, "Enter how many accounts you would like to generate from the mnemonic", prompt.ValidateNumber)
	if err != nil {
		return 0, err
	}
	numAccountsInt, err := strconv.Atoi(numAccounts)
	if err != nil {
		return 0, err
	}
	if numAccountsInt <= 0 {
		return 0, errors.New("must recover at least 1 account")
	}
	return int64(numAccountsInt), nil
}
