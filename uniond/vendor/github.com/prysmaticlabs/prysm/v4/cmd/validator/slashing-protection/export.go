package historycmd

import (
	"encoding/json"
	"fmt"
	"path/filepath"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/userprompt"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	slashingprotection "github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history"
	"github.com/urfave/cli/v2"
)

const (
	jsonExportFileName = "slashing_protection.json"
)

// Extracts a validator's slashing protection
// history from their database and formats it into an EIP-3076 standard JSON
// file via a CLI entrypoint to make it easy to migrate machines or Ethereum consensus clients.
//
// Steps:
// 1. Parse a path to the validator's datadir from the CLI context.
// 2. Open the validator database.
// 3. Call the function which actually exports the data from
// the validator's db into an EIP standard slashing protection format
// 4. Format and save the JSON file to a user's specified output directory.
func exportSlashingProtectionJSON(cliCtx *cli.Context) error {
	log.Info(
		"This command exports your validator's attestation and proposal history into " +
			"a file that can then be imported into any other Prysm setup across computers",
	)
	var err error
	dataDir := cliCtx.String(cmd.DataDirFlag.Name)
	if !cliCtx.IsSet(cmd.DataDirFlag.Name) {
		dataDir, err = userprompt.InputDirectory(cliCtx, userprompt.DataDirDirPromptText, cmd.DataDirFlag)
		if err != nil {
			return errors.Wrapf(err, "could not read directory value from input")
		}
	}
	// ensure that the validator.db is found under the specified dir or its subdirectories
	found, _, err := file.RecursiveFileFind(kv.ProtectionDbFileName, dataDir)
	if err != nil {
		return errors.Wrapf(err, "error finding validator database at path %s", dataDir)
	}
	if !found {
		return fmt.Errorf(
			"validator.db file (validator database) was not found at path %s, so nothing to export",
			dataDir,
		)
	}

	validatorDB, err := kv.NewKVStore(cliCtx.Context, dataDir, &kv.Config{})
	if err != nil {
		return errors.Wrapf(err, "could not access validator database at path %s", dataDir)
	}
	defer func() {
		if err := validatorDB.Close(); err != nil {
			log.WithError(err).Errorf("Could not close validator DB")
		}
	}()
	eipJSON, err := slashingprotection.ExportStandardProtectionJSON(cliCtx.Context, validatorDB)
	if err != nil {
		return errors.Wrap(err, "could not export slashing protection history")
	}

	// Check if JSON data is empty and issue a warning about common problems to the user.
	if eipJSON == nil || len(eipJSON.Data) == 0 {
		log.Fatal(
			"No slashing protection data was found in your database. This is likely because an older version of " +
				"Prysm would place your validator database in your wallet directory as a validator.db file. Now, " +
				"Prysm keeps its validator database inside the direct/ or derived/ folder in your wallet directory. " +
				"Try running this command again, but add direct/ or derived/ to the path where your wallet " +
				"directory is in and you should obtain your slashing protection history",
		)
	}

	outputDir, err := userprompt.InputDirectory(
		cliCtx,
		"Enter your desired output directory for your slashing protection history file",
		flags.SlashingProtectionExportDirFlag,
	)
	if err != nil {
		return errors.Wrap(err, "could not get slashing protection json file")
	}
	if outputDir == "" {
		return errors.New("output directory not specified")
	}
	exists, err := file.HasDir(outputDir)
	if err != nil {
		return errors.Wrapf(err, "could not check if output directory %s already exists", outputDir)
	}
	if !exists {
		if err := file.MkdirAll(outputDir); err != nil {
			return errors.Wrapf(err, "could not create output directory %s", outputDir)
		}
	}
	outputFilePath := filepath.Join(outputDir, jsonExportFileName)
	log.Infof("Writing slashing protection export JSON file to %s", outputFilePath)
	encoded, err := json.MarshalIndent(eipJSON, "", "\t")
	if err != nil {
		return errors.Wrap(err, "could not JSON marshal slashing protection history")
	}
	if err := file.WriteFile(outputFilePath, encoded); err != nil {
		return errors.Wrapf(err, "could not write file to path %s", outputFilePath)
	}
	log.Infof(
		"Successfully wrote %s. You can import this file using Prysm's "+
			"validator slashing-protection-history import command in another machine",
		outputFilePath,
	)
	return nil
}
