package accounts

import (
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/userprompt"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	"github.com/urfave/cli/v2"
)

const backupPromptText = "Enter the directory where your backup.zip file will be written to"

func accountsBackup(c *cli.Context) error {
	w, km, err := walletWithKeymanager(c)
	if err != nil {
		return err
	}
	dialOpts := client.ConstructDialOptions(
		c.Int(cmd.GrpcMaxCallRecvMsgSizeFlag.Name),
		c.String(flags.CertFlag.Name),
		c.Uint(flags.GrpcRetriesFlag.Name),
		c.Duration(flags.GrpcRetryDelayFlag.Name),
	)
	grpcHeaders := strings.Split(c.String(flags.GrpcHeadersFlag.Name), ",")

	opts := []accounts.Option{
		accounts.WithWallet(w),
		accounts.WithKeymanager(km),
		accounts.WithGRPCDialOpts(dialOpts),
		accounts.WithBeaconRPCProvider(c.String(flags.BeaconRPCProviderFlag.Name)),
		accounts.WithBeaconRESTApiProvider(c.String(flags.BeaconRESTApiProviderFlag.Name)),
		accounts.WithGRPCHeaders(grpcHeaders),
	}

	// Get full set of public keys from the keymanager.
	publicKeys, err := km.FetchValidatingPublicKeys(c.Context)
	if err != nil {
		return errors.Wrap(err, "could not fetch validating public keys")
	}
	// Filter keys either from CLI flag or from interactive session.
	filteredPubKeys, err := accounts.FilterPublicKeysFromUserInput(
		c,
		flags.BackupPublicKeysFlag,
		publicKeys,
		userprompt.SelectAccountsBackupPromptText,
	)
	if err != nil {
		return errors.Wrap(err, "could not filter public keys for backup")
	}
	opts = append(opts, accounts.WithFilteredPubKeys(filteredPubKeys))

	// Input the directory where they wish to backup their accounts.
	backupsDir, err := userprompt.InputDirectory(c, backupPromptText, flags.BackupDirFlag)
	if err != nil {
		return errors.Wrap(err, "could not parse keys directory")
	}
	// Ask the user for their desired password for their backed up accounts.
	backupsPassword, err := prompt.InputPassword(
		c,
		flags.BackupPasswordFile,
		"Enter a new password for your backed up accounts",
		"Confirm new password",
		true,
		prompt.ValidatePasswordInput,
	)
	if err != nil {
		return errors.Wrap(err, "could not determine password for backed up accounts")
	}

	opts = append(opts, accounts.WithBackupsDir(backupsDir))
	opts = append(opts, accounts.WithBackupsPassword(backupsPassword))

	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		return err
	}
	return acc.Backup(c.Context)
}
