package main

import (
	"fmt"

	"github.com/cockroachdb/errors"
	"github.com/spf13/cobra"
)

var listFsBench = &cobra.Command{
	Use:   "list [<name>] [<name>] ...",
	Short: "List the available file system benchmarks.",
	Long: `
List the available file system benchmarks. If no <name> is supplied
as an argument, then all the available benchmark names are printed.
If one or more <name>s are supplied as arguments, then the benchmark
descriptions are printed out for those names.
`,
	RunE: runListFsBench,
}

func runListFsBench(_ *cobra.Command, args []string) error {
	if len(args) == 0 {
		fmt.Println("Available benchmarks:")
		for name := range benchmarks {
			fmt.Println(name)
		}
	} else {
		for _, v := range args {
			benchStruct, ok := benchmarks[v]
			if !ok {
				return errors.Errorf("trying to print out the description for unknown benchmark: %s", v)
			}
			fmt.Println("Name:", benchStruct.name)
			fmt.Println("Description:", benchStruct.description)
		}
	}
	return nil
}
