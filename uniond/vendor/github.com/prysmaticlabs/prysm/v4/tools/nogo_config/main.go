package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
)

const exclusionMessage = "Excluded by @prysm//tools/nogo_config tool"

var (
	defaultExclusions = []string{
		"external/.*",
	}
)

var (
	input      = flag.String("input", "", "(required) input file")
	output     = flag.String("output", "", "(required) output file")
	checks     = flag.String("checks", "", "(required) comma separated list of checks to exclude")
	exclusions = flag.String("exclude_files", strings.Join(defaultExclusions, ","), "exclusions file")
	silent     = flag.Bool("silent", false, "disable logging")
)

func main() {
	flag.Parse()

	if *input == "" || *output == "" {
		fmt.Println("Error: input and output must be specified. Review the help text.")
		flag.Usage()
		return
	}

	if *checks == "" {
		fmt.Println("Error: checks must be specified. Review the help text.")
		flag.Usage()
		return
	}

	e := defaultExclusions
	if *exclusions != "" {
		e = strings.Split(*exclusions, ",")
	}

	f, err := os.Open(*input)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}
	defer func() {
		err := f.Close()
		if err != nil {
			panic(err)
		}
	}()
	data, err := io.ReadAll(f)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}

	var c Configs
	if err := json.Unmarshal(data, &c); err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}

	for _, check := range strings.Split(*checks, ",") {
		c.AddExclusion(strings.TrimSpace(check), e)
	}

	out, err := os.Create(*output)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}
	defer func() {
		err := out.Close()
		if err != nil {
			panic(err)
		}
	}()
	if err := json.NewEncoder(out).Encode(c); err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}

	if !*silent {
		fmt.Printf("Wrote %v\n", *output)
	}
}
