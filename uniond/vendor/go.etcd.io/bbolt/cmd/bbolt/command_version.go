package main

import (
	"fmt"
	"runtime"

	"github.com/spf13/cobra"
	"go.etcd.io/bbolt/version"
)

func newVersionCommand() *cobra.Command {
	versionCmd := &cobra.Command{
		Use:   "version",
		Short: "print the current version of bbolt",
		Long:  "print the current version of bbolt",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Printf("bbolt Version: %s\n", version.Version)
			fmt.Printf("Go Version: %s\n", runtime.Version())
			fmt.Printf("Go OS/Arch: %s/%s\n", runtime.GOOS, runtime.GOARCH)
		},
	}

	return versionCmd
}
