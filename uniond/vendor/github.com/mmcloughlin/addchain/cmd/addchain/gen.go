package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"io/ioutil"
	"strings"

	"github.com/google/subcommands"

	"github.com/mmcloughlin/addchain/acc/parse"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/internal/cli"
	"github.com/mmcloughlin/addchain/internal/gen"
)

// generate subcommand.
type generate struct {
	cli.Command

	typ    string
	tmpl   string
	output string
}

func (*generate) Name() string     { return "gen" }
func (*generate) Synopsis() string { return "generate output from an addition chain program" }
func (*generate) Usage() string {
	return `Usage: gen [-type <name>] [-tmpl <file>] [-out <file>] [<filename>]

Generate output from an addition chain program.

`
}

func (cmd *generate) SetFlags(f *flag.FlagSet) {
	defaulttype := "listing"
	if !gen.IsBuiltinTemplate(defaulttype) {
		panic("bad default template")
	}
	f.StringVar(&cmd.typ, "type", defaulttype, fmt.Sprintf("`name` of a builtin template (%s)", strings.Join(gen.BuiltinTemplateNames(), ",")))
	f.StringVar(&cmd.tmpl, "tmpl", "", "template `file` (overrides type)")
	f.StringVar(&cmd.output, "out", "", "output `file` (default stdout)")
}

func (cmd *generate) Execute(_ context.Context, f *flag.FlagSet, _ ...interface{}) (status subcommands.ExitStatus) {
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

	// Prepare template data.
	cfg := gen.Config{
		Allocator: pass.Allocator{
			Input:  "x",
			Output: "z",
			Format: "t%d",
		},
	}

	data, err := gen.PrepareData(cfg, s)
	if err != nil {
		return cmd.Error(err)
	}

	// Load template.
	tmpl, err := cmd.LoadTemplate()
	if err != nil {
		return cmd.Error(err)
	}

	// Open output.
	_, w, err := cli.OpenOutput(cmd.output)
	if err != nil {
		return cmd.Error(err)
	}
	defer cmd.CheckClose(&status, w)

	// Generate.
	if err := gen.Generate(w, tmpl, data); err != nil {
		return cmd.Error(err)
	}

	return subcommands.ExitSuccess
}

func (cmd *generate) LoadTemplate() (string, error) {
	// Explicit filename has precedence.
	if cmd.tmpl != "" {
		b, err := ioutil.ReadFile(cmd.tmpl)
		if err != nil {
			return "", err
		}
		return string(b), nil
	}

	// Lookup type name in builtin templates.
	if cmd.typ == "" {
		return "", errors.New("no builtin template specified")
	}

	s, err := gen.BuiltinTemplate(cmd.typ)
	if err != nil {
		return "", err
	}

	return s, nil
}
