package main

import (
	"os"

	"github.com/spf13/cobra"

	"cosmossdk.io/tools/confix"
)

func main() {
	cmd := &cobra.Command{
		Use:   "condiff f1 f2",
		Short: "Diff the keyspaces of the TOML documents in files f1 and f2",
		Long: `Diff the keyspaces of the TOML documents in files f1 and f2.
The output prints one line per key that differs:

   -S name    -- section exists in f1 but not f2
   +S name    -- section exists in f2 but not f1
   -M name    -- mapping exists in f1 but not f2
   +M name    -- mapping exists in f2 but not f1

Comments, order, and values are ignored for comparison purposes.`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			lhs, err := confix.LoadConfig(args[0])
			if err != nil {
				return err
			}

			rhs, err := confix.LoadConfig(args[1])
			if err != nil {
				return err
			}

			confix.PrintDiff(cmd.OutOrStdout(), confix.DiffKeys(lhs, rhs))
			return nil
		},
	}

	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}
