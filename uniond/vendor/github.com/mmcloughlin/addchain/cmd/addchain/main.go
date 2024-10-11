// Command addchain generates addition chains.
package main

import (
	"context"
	"flag"
	"os"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/internal/cli"
	"github.com/mmcloughlin/addchain/meta"
)

func main() {
	base := cli.NewBaseCommand("addchain")
	subcommands.Register(subcommands.HelpCommand(), "")
	subcommands.Register(&search{Command: base}, "")
	subcommands.Register(&eval{Command: base}, "")
	subcommands.Register(&format{Command: base}, "")
	subcommands.Register(&generate{Command: base}, "")

	if meta.Meta.BuildVersion != "" {
		subcommands.Register(&version{version: meta.Meta.BuildVersion, Command: base}, "")
	}

	subcommands.Register(&cite{properties: meta.Meta, Command: base}, "")

	flag.Parse()
	ctx := context.Background()
	os.Exit(int(subcommands.Execute(ctx)))
}
