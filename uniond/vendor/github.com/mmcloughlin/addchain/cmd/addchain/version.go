package main

import (
	"context"
	"flag"
	"fmt"
	"runtime"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/internal/cli"
)

// version subcommand.
type version struct {
	version string

	cli.Command
}

func (*version) Name() string     { return "version" }
func (*version) Synopsis() string { return "print addchain version" }
func (*version) Usage() string {
	return `Usage: version

Print the version of the addchain tool.

`
}

func (cmd *version) Execute(_ context.Context, f *flag.FlagSet, _ ...interface{}) (status subcommands.ExitStatus) {
	fmt.Printf("addchain version %s %s/%s\n", cmd.version, runtime.GOOS, runtime.GOARCH)
	return subcommands.ExitSuccess
}
