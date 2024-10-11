package main

import (
	"context"
	"flag"
	"fmt"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/acc"
	"github.com/mmcloughlin/addchain/acc/parse"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/internal/cli"
)

// eval subcommand.
type eval struct {
	cli.Command
}

func (*eval) Name() string     { return "eval" }
func (*eval) Synopsis() string { return "evaluate an addition chain script" }
func (*eval) Usage() string {
	return `Usage: eval [<filename>]

Evaluate an addition chain script.

`
}

func (cmd *eval) Execute(_ context.Context, f *flag.FlagSet, _ ...interface{}) (status subcommands.ExitStatus) {
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

	// Generate intermediate representation.
	p, err := acc.Translate(s)
	if err != nil {
		return cmd.Error(err)
	}

	// Evaluate and compile.
	if err := pass.Eval(p); err != nil {
		return cmd.Error(err)
	}

	// Dump.
	for n, op := range p.Program {
		fmt.Printf("[%3d] %3d+%3d\t%x\n", n+1, op.I, op.J, p.Chain[n+1])
	}

	doubles, adds := p.Program.Count()
	fmt.Printf("total: %d\tdoubles: \t%d adds: %d\n", doubles+adds, doubles, adds)

	return subcommands.ExitSuccess
}
