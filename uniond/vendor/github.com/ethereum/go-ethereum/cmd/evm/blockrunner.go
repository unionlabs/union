// Copyright 2023 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"sort"

	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers/logger"
	"github.com/ethereum/go-ethereum/tests"
	"github.com/urfave/cli/v2"
)

var blockTestCommand = &cli.Command{
	Action:    blockTestCmd,
	Name:      "blocktest",
	Usage:     "executes the given blockchain tests",
	ArgsUsage: "<file>",
}

func blockTestCmd(ctx *cli.Context) error {
	if len(ctx.Args().First()) == 0 {
		return errors.New("path-to-test argument required")
	}

	var tracer vm.EVMLogger
	// Configure the EVM logger
	if ctx.Bool(MachineFlag.Name) {
		tracer = logger.NewJSONLogger(&logger.Config{
			EnableMemory:     !ctx.Bool(DisableMemoryFlag.Name),
			DisableStack:     ctx.Bool(DisableStackFlag.Name),
			DisableStorage:   ctx.Bool(DisableStorageFlag.Name),
			EnableReturnData: !ctx.Bool(DisableReturnDataFlag.Name),
		}, os.Stderr)
	}
	// Load the test content from the input file
	src, err := os.ReadFile(ctx.Args().First())
	if err != nil {
		return err
	}
	var tests map[string]tests.BlockTest
	if err = json.Unmarshal(src, &tests); err != nil {
		return err
	}
	// run them in order
	var keys []string
	for key := range tests {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, name := range keys {
		test := tests[name]
		if err := test.Run(false, rawdb.HashScheme, tracer); err != nil {
			return fmt.Errorf("test %v: %w", name, err)
		}
	}
	return nil
}
