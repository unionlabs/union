package db

import (
	"context"
	"path"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	"github.com/urfave/cli/v2"
)

// MigrateUp for a validator database.
func MigrateUp(cliCtx *cli.Context) error {
	dataDir := cliCtx.String(cmd.DataDirFlag.Name)

	if !file.Exists(path.Join(dataDir, kv.ProtectionDbFileName)) {
		return errors.New("No validator db found at path, nothing to migrate")
	}

	ctx := context.Background()
	log.Info("Opening DB")
	validatorDB, err := kv.NewKVStore(ctx, dataDir, &kv.Config{})
	if err != nil {
		return err
	}
	log.Info("Running migrations")
	return validatorDB.RunUpMigrations(ctx)
}

// MigrateDown for a validator database.
func MigrateDown(cliCtx *cli.Context) error {
	dataDir := cliCtx.String(cmd.DataDirFlag.Name)

	if !file.Exists(path.Join(dataDir, kv.ProtectionDbFileName)) {
		return errors.New("No validator db found at path, nothing to rollback")
	}

	ctx := context.Background()
	log.Info("Opening DB")
	validatorDB, err := kv.NewKVStore(ctx, dataDir, &kv.Config{})
	if err != nil {
		return err
	}
	log.Info("Running migrations")
	return validatorDB.RunDownMigrations(ctx)
}
