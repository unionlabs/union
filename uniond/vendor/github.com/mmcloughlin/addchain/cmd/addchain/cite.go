package main

import (
	"context"
	"flag"
	"os"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/internal/cli"
	"github.com/mmcloughlin/addchain/meta"
)

// cite subcommand.
type cite struct {
	properties *meta.Properties

	cli.Command
}

func (*cite) Name() string     { return "cite" }
func (*cite) Synopsis() string { return "output addchain citation" }
func (*cite) Usage() string {
	return `Usage: cite

Output citation for addchain.

`
}

func (cmd *cite) Execute(_ context.Context, f *flag.FlagSet, _ ...interface{}) (status subcommands.ExitStatus) {
	// Check citable.
	if err := cmd.properties.CheckCitable(); err != nil {
		return cmd.Error(err)
	}

	// Generate citation.
	if err := cmd.properties.WriteCitation(os.Stdout); err != nil {
		return cmd.Error(err)
	}

	return subcommands.ExitSuccess
}
