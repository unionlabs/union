package db

import (
	"flag"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	dbtest "github.com/prysmaticlabs/prysm/v4/validator/db/testing"
	"github.com/urfave/cli/v2"
)

func TestMigrateUp_NoDBFound(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, "", "")
	require.NoError(t, set.Set(cmd.DataDirFlag.Name, ""))
	cliCtx := cli.NewContext(&app, set, nil)
	err := MigrateUp(cliCtx)
	assert.ErrorContains(t, "No validator db found at path", err)
}

func TestMigrateUp_OK(t *testing.T) {
	validatorDB := dbtest.SetupDB(t, nil)
	dbPath := validatorDB.DatabasePath()
	require.NoError(t, validatorDB.Close())
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, dbPath, "")
	require.NoError(t, set.Set(cmd.DataDirFlag.Name, dbPath))
	cliCtx := cli.NewContext(&app, set, nil)
	assert.NoError(t, MigrateUp(cliCtx))
}

func TestMigrateDown_NoDBFound(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, "", "")
	require.NoError(t, set.Set(cmd.DataDirFlag.Name, ""))
	cliCtx := cli.NewContext(&app, set, nil)
	err := MigrateDown(cliCtx)
	assert.ErrorContains(t, "No validator db found at path", err)
}

func TestMigrateDown_OK(t *testing.T) {
	validatorDB := dbtest.SetupDB(t, nil)
	dbPath := validatorDB.DatabasePath()
	require.NoError(t, validatorDB.Close())
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, dbPath, "")
	require.NoError(t, set.Set(cmd.DataDirFlag.Name, dbPath))
	cliCtx := cli.NewContext(&app, set, nil)
	assert.NoError(t, MigrateDown(cliCtx))
}
