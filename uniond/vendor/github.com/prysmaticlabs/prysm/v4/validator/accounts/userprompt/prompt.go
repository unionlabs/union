package userprompt

import (
	"github.com/logrusorgru/aurora"
	"github.com/manifoldco/promptui"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/urfave/cli/v2"
)

const (
	// ImportKeysDirPromptText for the import keys cli function.
	ImportKeysDirPromptText = "Enter the directory or filepath where your keystores to import are located"
	// DataDirDirPromptText for the validator database directory.
	DataDirDirPromptText = "Enter the directory of the validator database you would like to use"
	// SlashingProtectionJSONPromptText for the EIP-3076 slashing protection JSON userprompt.
	SlashingProtectionJSONPromptText = "Enter the filepath of your EIP-3076 Slashing Protection JSON from your previously used validator client"
	// WalletDirPromptText for the wallet.
	WalletDirPromptText = "Enter a wallet directory"
	// SelectAccountsDeletePromptText --
	SelectAccountsDeletePromptText = "Select the account(s) you would like to delete"
	// SelectAccountsBackupPromptText --
	SelectAccountsBackupPromptText = "Select the account(s) you wish to backup"
	// SelectAccountsVoluntaryExitPromptText --
	SelectAccountsVoluntaryExitPromptText = "Select the account(s) on which you wish to perform a voluntary exit"
)

var au = aurora.NewAurora(true)

// InputDirectory from the cli.
func InputDirectory(cliCtx *cli.Context, promptText string, flag *cli.StringFlag) (string, error) {
	directory := cliCtx.String(flag.Name)
	if cliCtx.IsSet(flag.Name) {
		return file.ExpandPath(directory)
	}
	// Append and log the appropriate directory name depending on the flag used.
	if flag.Name == flags.WalletDirFlag.Name {
		ok, err := file.HasDir(directory)
		if err != nil {
			return "", errors.Wrapf(err, "could not check if wallet dir %s exists", directory)
		}
		if ok {
			log.Infof("%s %s", au.BrightMagenta("(wallet path)"), directory)
			return directory, nil
		}
	}

	inputtedDir, err := prompt.DefaultPrompt(au.Bold(promptText).String(), directory)
	if err != nil {
		return "", err
	}
	if inputtedDir == directory {
		return directory, nil
	}
	return file.ExpandPath(inputtedDir)
}

// FormatPromptError for the user.
func FormatPromptError(err error) error {
	switch err {
	case promptui.ErrAbort:
		return errors.New("wallet creation aborted, closing")
	case promptui.ErrInterrupt:
		return errors.New("keyboard interrupt, closing")
	case promptui.ErrEOF:
		return errors.New("no input received, closing")
	default:
		return err
	}
}
