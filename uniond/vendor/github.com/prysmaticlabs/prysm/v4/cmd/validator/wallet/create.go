package wallet

import (
	"fmt"
	"os"
	"strings"

	"github.com/manifoldco/promptui"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/userprompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/urfave/cli/v2"
)

const (
	// #nosec G101 -- Not sensitive data
	newMnemonicPassphraseYesNoText = "(Advanced) Do you want to setup a '25th word' passphrase for your mnemonic? [y/n]"
	// #nosec G101 -- Not sensitive data
	newMnemonicPassphrasePromptText = "(Advanced) Setup a passphrase '25th word' for your mnemonic " +
		"(WARNING: You cannot recover your keys from your mnemonic if you forget this passphrase!)"
)

func walletCreate(c *cli.Context) error {
	keymanagerKind, err := inputKeymanagerKind(c)
	if err != nil {
		return err
	}

	opts, err := ConstructCLIManagerOpts(c, keymanagerKind)
	if err != nil {
		return err
	}

	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		return err
	}
	if _, err := acc.WalletCreate(c.Context); err != nil {
		return errors.Wrap(err, "could not create wallet")
	}
	return nil
}

// ConstructCLIManagerOpts prompts the user for wallet creation input.
func ConstructCLIManagerOpts(cliCtx *cli.Context, keymanagerKind keymanager.Kind) ([]accounts.Option, error) {
	var cliOpts []accounts.Option
	// Get wallet dir and check that no wallet exists at the location.
	walletDir, err := userprompt.InputDirectory(cliCtx, userprompt.WalletDirPromptText, flags.WalletDirFlag)
	if err != nil {
		return []accounts.Option{}, err
	}
	dirExists, err := wallet.Exists(walletDir)
	if err != nil {
		return []accounts.Option{}, err
	}
	if dirExists {
		return []accounts.Option{}, errors.New("a wallet already exists at this location. Please input an" +
			" alternative location for the new wallet or remove the current wallet")
	}

	walletPassword, err := prompt.InputPassword(
		cliCtx,
		flags.WalletPasswordFileFlag,
		wallet.NewWalletPasswordPromptText,
		wallet.ConfirmPasswordPromptText,
		true, /* Should confirm password */
		prompt.ValidatePasswordInput,
	)
	if err != nil {
		return []accounts.Option{}, err
	}
	cliOpts = append(cliOpts, accounts.WithWalletDir(walletDir))
	cliOpts = append(cliOpts, accounts.WithWalletPassword(walletPassword))
	cliOpts = append(cliOpts, accounts.WithKeymanagerType(keymanagerKind))
	cliOpts = append(cliOpts, accounts.WithSkipMnemonicConfirm(cliCtx.Bool(flags.SkipDepositConfirmationFlag.Name)))
	if cliCtx.IsSet(flags.MnemonicLanguageFlag.Name) {
		cliOpts = append(cliOpts, accounts.WithMnemonicLanguage(cliCtx.String(flags.MnemonicLanguageFlag.Name)))
	}

	skipMnemonic25thWord := cliCtx.IsSet(flags.SkipMnemonic25thWordCheckFlag.Name)
	has25thWordFile := cliCtx.IsSet(flags.Mnemonic25thWordFileFlag.Name)
	if keymanagerKind == keymanager.Derived {
		numAccounts, err := inputNumAccounts(cliCtx)
		if err != nil {
			return []accounts.Option{}, errors.Wrap(err, "could not get number of accounts to generate")
		}
		cliOpts = append(cliOpts, accounts.WithNumAccounts(int(numAccounts)))
	}
	if keymanagerKind == keymanager.Derived && !skipMnemonic25thWord && !has25thWordFile {
		resp, err := prompt.ValidatePrompt(
			os.Stdin, newMnemonicPassphraseYesNoText, prompt.ValidateYesOrNo,
		)
		if err != nil {
			return []accounts.Option{}, errors.Wrap(err, "could not validate choice")
		}
		if strings.EqualFold(resp, "y") {
			mnemonicPassphrase, err := prompt.InputPassword(
				cliCtx,
				flags.Mnemonic25thWordFileFlag,
				newMnemonicPassphrasePromptText,
				"Confirm mnemonic passphrase",
				true, /* Should confirm password */
				func(input string) error {
					if strings.TrimSpace(input) == "" {
						return errors.New("input cannot be empty")
					}
					return nil
				},
			)
			if err != nil {
				return []accounts.Option{}, err
			}
			cliOpts = append(cliOpts, accounts.WithMnemonic25thWord(mnemonicPassphrase))
		}
	}
	if keymanagerKind == keymanager.Web3Signer {
		return []accounts.Option{}, errors.New("web3signer keymanager does not require persistent wallets.")
	}
	return cliOpts, nil
}

func inputKeymanagerKind(cliCtx *cli.Context) (keymanager.Kind, error) {
	if cliCtx.IsSet(flags.KeymanagerKindFlag.Name) {
		return keymanager.ParseKind(cliCtx.String(flags.KeymanagerKindFlag.Name))
	}
	promptSelect := promptui.Select{
		Label: "Select a type of wallet",
		Items: []string{
			wallet.KeymanagerKindSelections[keymanager.Local],
			wallet.KeymanagerKindSelections[keymanager.Derived],
			wallet.KeymanagerKindSelections[keymanager.Web3Signer],
		},
	}
	selection, _, err := promptSelect.Run()
	if err != nil {
		return keymanager.Local, fmt.Errorf("could not select wallet type: %w", userprompt.FormatPromptError(err))
	}
	return keymanager.Kind(selection), nil
}

// CreateAndSaveWalletCli from user input with a desired keymanager. If a
// wallet already exists in the path, it suggests the user alternatives
// such as how to edit their existing wallet configuration.
func CreateAndSaveWalletCli(cliCtx *cli.Context) (*wallet.Wallet, error) {
	keymanagerKind, err := inputKeymanagerKind(cliCtx)
	if err != nil {
		return nil, err
	}
	opts, err := ConstructCLIManagerOpts(cliCtx, keymanagerKind)
	if err != nil {
		return nil, err
	}
	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		return nil, err
	}
	w, err := acc.WalletCreate(cliCtx.Context)
	if err != nil {
		return nil, errors.Wrap(err, "could not create wallet")
	}
	return w, nil
}
