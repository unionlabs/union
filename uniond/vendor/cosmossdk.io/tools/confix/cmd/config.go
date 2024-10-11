package cmd

import (
	"github.com/spf13/cobra"
)

const (
	tomlSuffix = ".toml"
)

// ConfigCommand contains all the confix commands
// These command can be used to interactively update an application config value.
func ConfigCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "config",
		Short: "Utilities for managing application configuration",
	}

	cmd.AddCommand(
		MigrateCommand(),
		DiffCommand(),
		GetCommand(),
		SetCommand(),
		ViewCommand(),
		HomeCommand(),
	)

	return cmd
}
