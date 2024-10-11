// Copyright 2014 The go-ethereum Authors
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

// evm executes EVM code snippets.
package main

import (
	"fmt"
	"math/big"
	"os"

	"github.com/ethereum/go-ethereum/cmd/evm/internal/t8ntool"
	"github.com/ethereum/go-ethereum/internal/debug"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/urfave/cli/v2"
)

var (
	DebugFlag = &cli.BoolFlag{
		Name:     "debug",
		Usage:    "output full trace logs",
		Category: flags.VMCategory,
	}
	StatDumpFlag = &cli.BoolFlag{
		Name:     "statdump",
		Usage:    "displays stack and heap memory information",
		Category: flags.VMCategory,
	}
	CodeFlag = &cli.StringFlag{
		Name:     "code",
		Usage:    "EVM code",
		Category: flags.VMCategory,
	}
	CodeFileFlag = &cli.StringFlag{
		Name:     "codefile",
		Usage:    "File containing EVM code. If '-' is specified, code is read from stdin ",
		Category: flags.VMCategory,
	}
	GasFlag = &cli.Uint64Flag{
		Name:     "gas",
		Usage:    "gas limit for the evm",
		Value:    10000000000,
		Category: flags.VMCategory,
	}
	PriceFlag = &flags.BigFlag{
		Name:     "price",
		Usage:    "price set for the evm",
		Value:    new(big.Int),
		Category: flags.VMCategory,
	}
	ValueFlag = &flags.BigFlag{
		Name:     "value",
		Usage:    "value set for the evm",
		Value:    new(big.Int),
		Category: flags.VMCategory,
	}
	DumpFlag = &cli.BoolFlag{
		Name:     "dump",
		Usage:    "dumps the state after the run",
		Category: flags.VMCategory,
	}
	InputFlag = &cli.StringFlag{
		Name:     "input",
		Usage:    "input for the EVM",
		Category: flags.VMCategory,
	}
	InputFileFlag = &cli.StringFlag{
		Name:     "inputfile",
		Usage:    "file containing input for the EVM",
		Category: flags.VMCategory,
	}
	BenchFlag = &cli.BoolFlag{
		Name:     "bench",
		Usage:    "benchmark the execution",
		Category: flags.VMCategory,
	}
	CreateFlag = &cli.BoolFlag{
		Name:     "create",
		Usage:    "indicates the action should be create rather than call",
		Category: flags.VMCategory,
	}
	GenesisFlag = &cli.StringFlag{
		Name:     "prestate",
		Usage:    "JSON file with prestate (genesis) config",
		Category: flags.VMCategory,
	}
	MachineFlag = &cli.BoolFlag{
		Name:     "json",
		Usage:    "output trace logs in machine readable format (json)",
		Category: flags.VMCategory,
	}
	SenderFlag = &cli.StringFlag{
		Name:     "sender",
		Usage:    "The transaction origin",
		Category: flags.VMCategory,
	}
	ReceiverFlag = &cli.StringFlag{
		Name:     "receiver",
		Usage:    "The transaction receiver (execution context)",
		Category: flags.VMCategory,
	}
	DisableMemoryFlag = &cli.BoolFlag{
		Name:     "nomemory",
		Value:    true,
		Usage:    "disable memory output",
		Category: flags.VMCategory,
	}
	DisableStackFlag = &cli.BoolFlag{
		Name:     "nostack",
		Usage:    "disable stack output",
		Category: flags.VMCategory,
	}
	DisableStorageFlag = &cli.BoolFlag{
		Name:     "nostorage",
		Usage:    "disable storage output",
		Category: flags.VMCategory,
	}
	DisableReturnDataFlag = &cli.BoolFlag{
		Name:     "noreturndata",
		Value:    true,
		Usage:    "enable return data output",
		Category: flags.VMCategory,
	}
)

var stateTransitionCommand = &cli.Command{
	Name:    "transition",
	Aliases: []string{"t8n"},
	Usage:   "executes a full state transition",
	Action:  t8ntool.Transition,
	Flags: []cli.Flag{
		t8ntool.TraceFlag,
		t8ntool.TraceDisableMemoryFlag,
		t8ntool.TraceEnableMemoryFlag,
		t8ntool.TraceDisableStackFlag,
		t8ntool.TraceDisableReturnDataFlag,
		t8ntool.TraceEnableReturnDataFlag,
		t8ntool.OutputBasedir,
		t8ntool.OutputAllocFlag,
		t8ntool.OutputResultFlag,
		t8ntool.OutputBodyFlag,
		t8ntool.InputAllocFlag,
		t8ntool.InputEnvFlag,
		t8ntool.InputTxsFlag,
		t8ntool.ForknameFlag,
		t8ntool.ChainIDFlag,
		t8ntool.RewardFlag,
		t8ntool.VerbosityFlag,
	},
}

var transactionCommand = &cli.Command{
	Name:    "transaction",
	Aliases: []string{"t9n"},
	Usage:   "performs transaction validation",
	Action:  t8ntool.Transaction,
	Flags: []cli.Flag{
		t8ntool.InputTxsFlag,
		t8ntool.ChainIDFlag,
		t8ntool.ForknameFlag,
		t8ntool.VerbosityFlag,
	},
}

var blockBuilderCommand = &cli.Command{
	Name:    "block-builder",
	Aliases: []string{"b11r"},
	Usage:   "builds a block",
	Action:  t8ntool.BuildBlock,
	Flags: []cli.Flag{
		t8ntool.OutputBasedir,
		t8ntool.OutputBlockFlag,
		t8ntool.InputHeaderFlag,
		t8ntool.InputOmmersFlag,
		t8ntool.InputWithdrawalsFlag,
		t8ntool.InputTxsRlpFlag,
		t8ntool.SealCliqueFlag,
		t8ntool.VerbosityFlag,
	},
}

// vmFlags contains flags related to running the EVM.
var vmFlags = []cli.Flag{
	CodeFlag,
	CodeFileFlag,
	CreateFlag,
	GasFlag,
	PriceFlag,
	ValueFlag,
	InputFlag,
	InputFileFlag,
	GenesisFlag,
	SenderFlag,
	ReceiverFlag,
}

// traceFlags contains flags that configure tracing output.
var traceFlags = []cli.Flag{
	BenchFlag,
	DebugFlag,
	DumpFlag,
	MachineFlag,
	StatDumpFlag,
	DisableMemoryFlag,
	DisableStackFlag,
	DisableStorageFlag,
	DisableReturnDataFlag,
}

var app = flags.NewApp("the evm command line interface")

func init() {
	app.Flags = flags.Merge(vmFlags, traceFlags, debug.Flags)
	app.Commands = []*cli.Command{
		compileCommand,
		disasmCommand,
		runCommand,
		blockTestCommand,
		stateTestCommand,
		stateTransitionCommand,
		transactionCommand,
		blockBuilderCommand,
	}
	app.Before = func(ctx *cli.Context) error {
		flags.MigrateGlobalFlags(ctx)
		return debug.Setup(ctx)
	}
	app.After = func(ctx *cli.Context) error {
		debug.Exit()
		return nil
	}
}

func main() {
	if err := app.Run(os.Args); err != nil {
		code := 1
		if ec, ok := err.(*t8ntool.NumberedError); ok {
			code = ec.ExitCode()
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(code)
	}
}
