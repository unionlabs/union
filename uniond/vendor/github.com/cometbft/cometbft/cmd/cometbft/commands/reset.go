package commands

import (
	"os"
	"path/filepath"

	"github.com/spf13/cobra"

	"github.com/cometbft/cometbft/libs/log"
	cmtos "github.com/cometbft/cometbft/libs/os"
	"github.com/cometbft/cometbft/privval"
)

// ResetAllCmd removes the database of this CometBFT core
// instance.
var ResetAllCmd = &cobra.Command{
	Use:     "unsafe-reset-all",
	Aliases: []string{"unsafe_reset_all"},
	Short:   "(unsafe) Remove all the data and WAL, reset this node's validator to genesis state",
	RunE:    resetAllCmd,
}

var keepAddrBook bool

// ResetStateCmd removes the database of the specified CometBFT core instance.
var ResetStateCmd = &cobra.Command{
	Use:     "reset-state",
	Aliases: []string{"reset_state"},
	Short:   "Remove all the data and WAL",
	RunE: func(cmd *cobra.Command, args []string) (err error) {
		config, err = ParseConfig(cmd)
		if err != nil {
			return err
		}

		return resetState(config.DBDir(), logger)
	},
}

func init() {
	ResetAllCmd.Flags().BoolVar(&keepAddrBook, "keep-addr-book", false, "keep the address book intact")
}

// ResetPrivValidatorCmd resets the private validator files.
var ResetPrivValidatorCmd = &cobra.Command{
	Use:     "unsafe-reset-priv-validator",
	Aliases: []string{"unsafe_reset_priv_validator"},
	Short:   "(unsafe) Reset this node's validator to genesis state",
	RunE:    resetPrivValidator,
}

// XXX: this is totally unsafe.
// it's only suitable for testnets.
func resetAllCmd(cmd *cobra.Command, _ []string) (err error) {
	config, err = ParseConfig(cmd)
	if err != nil {
		return err
	}

	return resetAll(
		config.DBDir(),
		config.P2P.AddrBookFile(),
		config.PrivValidatorKeyFile(),
		config.PrivValidatorStateFile(),
		logger,
	)
}

// XXX: this is totally unsafe.
// it's only suitable for testnets.
func resetPrivValidator(cmd *cobra.Command, _ []string) (err error) {
	config, err = ParseConfig(cmd)
	if err != nil {
		return err
	}

	resetFilePV(config.PrivValidatorKeyFile(), config.PrivValidatorStateFile(), logger)
	return nil
}

// resetAll removes address book files plus all data, and resets the privValdiator data.
func resetAll(dbDir, addrBookFile, privValKeyFile, privValStateFile string, logger log.Logger) error {
	if keepAddrBook {
		logger.Info("The address book remains intact")
	} else {
		removeAddrBook(addrBookFile, logger)
	}

	if err := os.RemoveAll(dbDir); err == nil {
		logger.Info("Removed all blockchain history", "dir", dbDir)
	} else {
		logger.Error("Error removing all blockchain history", "dir", dbDir, "err", err)
	}

	if err := cmtos.EnsureDir(dbDir, 0o700); err != nil {
		logger.Error("unable to recreate dbDir", "err", err)
	}

	// recreate the dbDir since the privVal state needs to live there
	resetFilePV(privValKeyFile, privValStateFile, logger)
	return nil
}

// resetState removes address book files plus all databases.
func resetState(dbDir string, logger log.Logger) error {
	blockdb := filepath.Join(dbDir, "blockstore.db")
	state := filepath.Join(dbDir, "state.db")
	wal := filepath.Join(dbDir, "cs.wal")
	evidence := filepath.Join(dbDir, "evidence.db")
	txIndex := filepath.Join(dbDir, "tx_index.db")

	if cmtos.FileExists(blockdb) {
		if err := os.RemoveAll(blockdb); err == nil {
			logger.Info("Removed all blockstore.db", "dir", blockdb)
		} else {
			logger.Error("error removing all blockstore.db", "dir", blockdb, "err", err)
		}
	}

	if cmtos.FileExists(state) {
		if err := os.RemoveAll(state); err == nil {
			logger.Info("Removed all state.db", "dir", state)
		} else {
			logger.Error("error removing all state.db", "dir", state, "err", err)
		}
	}

	if cmtos.FileExists(wal) {
		if err := os.RemoveAll(wal); err == nil {
			logger.Info("Removed all cs.wal", "dir", wal)
		} else {
			logger.Error("error removing all cs.wal", "dir", wal, "err", err)
		}
	}

	if cmtos.FileExists(evidence) {
		if err := os.RemoveAll(evidence); err == nil {
			logger.Info("Removed all evidence.db", "dir", evidence)
		} else {
			logger.Error("error removing all evidence.db", "dir", evidence, "err", err)
		}
	}

	if cmtos.FileExists(txIndex) {
		if err := os.RemoveAll(txIndex); err == nil {
			logger.Info("Removed tx_index.db", "dir", txIndex)
		} else {
			logger.Error("error removing tx_index.db", "dir", txIndex, "err", err)
		}
	}

	if err := cmtos.EnsureDir(dbDir, 0o700); err != nil {
		logger.Error("unable to recreate dbDir", "err", err)
	}
	return nil
}

func resetFilePV(privValKeyFile, privValStateFile string, logger log.Logger) {
	if _, err := os.Stat(privValKeyFile); err == nil {
		pv := privval.LoadFilePVEmptyState(privValKeyFile, privValStateFile)
		pv.Reset()
		logger.Info(
			"Reset private validator file to genesis state",
			"keyFile", privValKeyFile,
			"stateFile", privValStateFile,
		)
	} else {
		pv := privval.GenFilePV(privValKeyFile, privValStateFile)
		pv.Save()
		logger.Info(
			"Generated private validator file",
			"keyFile", privValKeyFile,
			"stateFile", privValStateFile,
		)
	}
}

func removeAddrBook(addrBookFile string, logger log.Logger) {
	if err := os.Remove(addrBookFile); err == nil {
		logger.Info("Removed existing address book", "file", addrBookFile)
	} else if !os.IsNotExist(err) {
		logger.Info("Error removing address book", "file", addrBookFile, "err", err)
	}
}
