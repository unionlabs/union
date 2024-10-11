package main

import (
	"embed"
	"log"
	"os"

	"github.com/urfave/cli/v2"
)

var (
	dirFlag = &cli.StringFlag{
		Name:     "dir",
		Value:    "",
		Usage:    "Target directory",
		Required: true,
	}
)

//go:embed data
var specFS embed.FS

var specDirs = map[string][]string{
	"specs/phase0": {
		"beacon-chain.md",
		"fork-choice.md",
		"validator.md",
		"weak-subjectivity.md",
	},
	"ssz": {
		"merkle-proofs.md",
	},
}

func main() {
	app := &cli.App{
		Name:        "Specs checker utility",
		Description: "Checks that specs pseudo code used in comments is up to date",
		Usage:       "helps keeping specs pseudo code up to date!",
		Commands: []*cli.Command{
			{
				Name:  "check",
				Usage: "Checks that all doc strings",
				Flags: []cli.Flag{
					dirFlag,
				},
				Action: check,
			},
			{
				Name:   "download",
				Usage:  "Downloads the latest specs docs",
				Action: download,
				Flags: []cli.Flag{
					dirFlag,
				},
			},
		},
	}

	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}
