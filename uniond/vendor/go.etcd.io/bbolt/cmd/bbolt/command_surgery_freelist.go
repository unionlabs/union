package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/surgeon"
)

func newSurgeryFreelistCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "freelist <subcommand>",
		Short: "freelist related surgery commands",
	}

	cmd.AddCommand(newSurgeryFreelistAbandonCommand())
	cmd.AddCommand(newSurgeryFreelistRebuildCommand())

	return cmd
}

func newSurgeryFreelistAbandonCommand() *cobra.Command {
	var o surgeryBaseOptions
	abandonFreelistCmd := &cobra.Command{
		Use:   "abandon <bbolt-file>",
		Short: "Abandon the freelist from both meta pages",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryFreelistAbandonFunc(args[0], o)
		},
	}
	o.AddFlags(abandonFreelistCmd.Flags())

	return abandonFreelistCmd
}

func surgeryFreelistAbandonFunc(srcDBPath string, cfg surgeryBaseOptions) error {
	if _, err := checkSourceDBPath(srcDBPath); err != nil {
		return err
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[freelist abandon] copy file failed: %w", err)
	}

	if err := surgeon.ClearFreelist(cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("abandom-freelist command failed: %w", err)
	}

	fmt.Fprintf(os.Stdout, "The freelist was abandoned in both meta pages.\nIt may cause some delay on next startup because bbolt needs to scan the whole db to reconstruct the free list.\n")
	return nil
}

func newSurgeryFreelistRebuildCommand() *cobra.Command {
	var o surgeryBaseOptions
	rebuildFreelistCmd := &cobra.Command{
		Use:   "rebuild <bbolt-file>",
		Short: "Rebuild the freelist",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := o.Validate(); err != nil {
				return err
			}
			return surgeryFreelistRebuildFunc(args[0], o)
		},
	}
	o.AddFlags(rebuildFreelistCmd.Flags())

	return rebuildFreelistCmd
}

func surgeryFreelistRebuildFunc(srcDBPath string, cfg surgeryBaseOptions) error {
	// Ensure source file exists.
	fi, err := checkSourceDBPath(srcDBPath)
	if err != nil {
		return err
	}

	// make sure the freelist isn't present in the file.
	meta, err := readMetaPage(srcDBPath)
	if err != nil {
		return err
	}
	if meta.IsFreelistPersisted() {
		return ErrSurgeryFreelistAlreadyExist
	}

	if err := common.CopyFile(srcDBPath, cfg.outputDBFilePath); err != nil {
		return fmt.Errorf("[freelist rebuild] copy file failed: %w", err)
	}

	// bboltDB automatically reconstruct & sync freelist in write mode.
	db, err := bolt.Open(cfg.outputDBFilePath, fi.Mode(), &bolt.Options{NoFreelistSync: false})
	if err != nil {
		return fmt.Errorf("[freelist rebuild] open db file failed: %w", err)
	}
	err = db.Close()
	if err != nil {
		return fmt.Errorf("[freelist rebuild] close db file failed: %w", err)
	}

	fmt.Fprintf(os.Stdout, "The freelist was successfully rebuilt.\n")
	return nil
}
