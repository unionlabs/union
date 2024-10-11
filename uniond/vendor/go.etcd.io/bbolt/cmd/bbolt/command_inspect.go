package main

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/cobra"

	bolt "go.etcd.io/bbolt"
)

func newInspectCommand() *cobra.Command {
	inspectCmd := &cobra.Command{
		Use:   "inspect <bbolt-file>",
		Short: "inspect the structure of the database",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			return inspectFunc(args[0])
		},
	}

	return inspectCmd
}

func inspectFunc(srcDBPath string) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	db, err := bolt.Open(srcDBPath, 0600, &bolt.Options{ReadOnly: true})
	if err != nil {
		return err
	}
	defer db.Close()

	return db.View(func(tx *bolt.Tx) error {
		bs := tx.Inspect()
		out, err := json.MarshalIndent(bs, "", "    ")
		if err != nil {
			return err
		}
		fmt.Fprintln(os.Stdout, string(out))
		return nil
	})
}
