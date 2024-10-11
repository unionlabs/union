package main

import (
	"context"
	"flag"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/acc"
	"github.com/mmcloughlin/addchain/acc/parse"
	"github.com/mmcloughlin/addchain/acc/printer"
	"github.com/mmcloughlin/addchain/internal/cli"
)

// format subcommand.
type format struct {
	cli.Command

	build bool
}

func (*format) Name() string     { return "fmt" }
func (*format) Synopsis() string { return "format an addition chain script" }
func (*format) Usage() string {
	return `Usage: fmt [<filename>]

Format an addition chain script.

`
}

func (cmd *format) SetFlags(f *flag.FlagSet) {
	f.BoolVar(&cmd.build, "b", false, "rebuild from intermediate representation")
}

func (cmd *format) Execute(_ context.Context, f *flag.FlagSet, _ ...interface{}) (status subcommands.ExitStatus) {
	// Read input.
	input, r, err := cli.OpenInput(f.Arg(0))
	if err != nil {
		return cmd.Error(err)
	}
	defer cmd.CheckClose(&status, r)

	// Parse to a syntax tree.
	s, err := parse.Reader(input, r)
	if err != nil {
		return cmd.Error(err)
	}

	// Rebuild, if configured.
	if cmd.build {
		r, err := acc.Translate(s)
		if err != nil {
			return cmd.Error(err)
		}

		s, err = acc.Build(r)
		if err != nil {
			return cmd.Error(err)
		}
	}

	// Print.
	if err := printer.Print(s); err != nil {
		return cmd.Error(err)
	}

	return subcommands.ExitSuccess
}
