package main

import (
	"github.com/spf13/cobra"
)

const (
	cliName        = "bbolt"
	cliDescription = "A simple command line tool for inspecting bbolt databases"
)

func NewRootCommand() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:     cliName,
		Short:   cliDescription,
		Version: "dev",
	}

	rootCmd.AddCommand(
		newVersionCommand(),
		newSurgeryCommand(),
		newInspectCommand(),
		newCheckCommand(),
	)

	return rootCmd
}
