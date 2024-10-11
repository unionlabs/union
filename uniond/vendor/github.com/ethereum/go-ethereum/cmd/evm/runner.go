// Copyright 2017 The go-ethereum Authors
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
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"os"
	goruntime "runtime"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/cmd/evm/internal/compiler"
	"github.com/ethereum/go-ethereum/cmd/utils"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/core/vm/runtime"
	"github.com/ethereum/go-ethereum/eth/tracers/logger"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/triedb/hashdb"
	"github.com/urfave/cli/v2"
)

var runCommand = &cli.Command{
	Action:      runCmd,
	Name:        "run",
	Usage:       "run arbitrary evm binary",
	ArgsUsage:   "<code>",
	Description: `The run command runs arbitrary EVM code.`,
	Flags:       flags.Merge(vmFlags, traceFlags),
}

// readGenesis will read the given JSON format genesis file and return
// the initialized Genesis structure
func readGenesis(genesisPath string) *core.Genesis {
	// Make sure we have a valid genesis JSON
	//genesisPath := ctx.Args().First()
	if len(genesisPath) == 0 {
		utils.Fatalf("Must supply path to genesis JSON file")
	}
	file, err := os.Open(genesisPath)
	if err != nil {
		utils.Fatalf("Failed to read genesis file: %v", err)
	}
	defer file.Close()

	genesis := new(core.Genesis)
	if err := json.NewDecoder(file).Decode(genesis); err != nil {
		utils.Fatalf("invalid genesis file: %v", err)
	}
	return genesis
}

type execStats struct {
	time           time.Duration // The execution time.
	allocs         int64         // The number of heap allocations during execution.
	bytesAllocated int64         // The cumulative number of bytes allocated during execution.
}

func timedExec(bench bool, execFunc func() ([]byte, uint64, error)) (output []byte, gasLeft uint64, stats execStats, err error) {
	if bench {
		result := testing.Benchmark(func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				output, gasLeft, err = execFunc()
			}
		})

		// Get the average execution time from the benchmarking result.
		// There are other useful stats here that could be reported.
		stats.time = time.Duration(result.NsPerOp())
		stats.allocs = result.AllocsPerOp()
		stats.bytesAllocated = result.AllocedBytesPerOp()
	} else {
		var memStatsBefore, memStatsAfter goruntime.MemStats
		goruntime.ReadMemStats(&memStatsBefore)
		startTime := time.Now()
		output, gasLeft, err = execFunc()
		stats.time = time.Since(startTime)
		goruntime.ReadMemStats(&memStatsAfter)
		stats.allocs = int64(memStatsAfter.Mallocs - memStatsBefore.Mallocs)
		stats.bytesAllocated = int64(memStatsAfter.TotalAlloc - memStatsBefore.TotalAlloc)
	}

	return output, gasLeft, stats, err
}

func runCmd(ctx *cli.Context) error {
	logconfig := &logger.Config{
		EnableMemory:     !ctx.Bool(DisableMemoryFlag.Name),
		DisableStack:     ctx.Bool(DisableStackFlag.Name),
		DisableStorage:   ctx.Bool(DisableStorageFlag.Name),
		EnableReturnData: !ctx.Bool(DisableReturnDataFlag.Name),
		Debug:            ctx.Bool(DebugFlag.Name),
	}

	var (
		tracer      vm.EVMLogger
		debugLogger *logger.StructLogger
		statedb     *state.StateDB
		chainConfig *params.ChainConfig
		sender      = common.BytesToAddress([]byte("sender"))
		receiver    = common.BytesToAddress([]byte("receiver"))
		preimages   = ctx.Bool(DumpFlag.Name)
		blobHashes  []common.Hash  // TODO (MariusVanDerWijden) implement blob hashes in state tests
		blobBaseFee = new(big.Int) // TODO (MariusVanDerWijden) implement blob fee in state tests
	)
	if ctx.Bool(MachineFlag.Name) {
		tracer = logger.NewJSONLogger(logconfig, os.Stdout)
	} else if ctx.Bool(DebugFlag.Name) {
		debugLogger = logger.NewStructLogger(logconfig)
		tracer = debugLogger
	} else {
		debugLogger = logger.NewStructLogger(logconfig)
	}

	initialGas := ctx.Uint64(GasFlag.Name)
	genesisConfig := new(core.Genesis)
	genesisConfig.GasLimit = initialGas
	if ctx.String(GenesisFlag.Name) != "" {
		genesisConfig = readGenesis(ctx.String(GenesisFlag.Name))
		if genesisConfig.GasLimit != 0 {
			initialGas = genesisConfig.GasLimit
		}
	} else {
		genesisConfig.Config = params.AllEthashProtocolChanges
	}

	db := rawdb.NewMemoryDatabase()
	triedb := trie.NewDatabase(db, &trie.Config{
		Preimages: preimages,
		HashDB:    hashdb.Defaults,
	})
	defer triedb.Close()
	genesis := genesisConfig.MustCommit(db, triedb)
	sdb := state.NewDatabaseWithNodeDB(db, triedb)
	statedb, _ = state.New(genesis.Root(), sdb, nil)
	chainConfig = genesisConfig.Config

	if ctx.String(SenderFlag.Name) != "" {
		sender = common.HexToAddress(ctx.String(SenderFlag.Name))
	}
	statedb.CreateAccount(sender)

	if ctx.String(ReceiverFlag.Name) != "" {
		receiver = common.HexToAddress(ctx.String(ReceiverFlag.Name))
	}

	var code []byte
	codeFileFlag := ctx.String(CodeFileFlag.Name)
	codeFlag := ctx.String(CodeFlag.Name)

	// The '--code' or '--codefile' flag overrides code in state
	if codeFileFlag != "" || codeFlag != "" {
		var hexcode []byte
		if codeFileFlag != "" {
			var err error
			// If - is specified, it means that code comes from stdin
			if codeFileFlag == "-" {
				//Try reading from stdin
				if hexcode, err = io.ReadAll(os.Stdin); err != nil {
					fmt.Printf("Could not load code from stdin: %v\n", err)
					os.Exit(1)
				}
			} else {
				// Codefile with hex assembly
				if hexcode, err = os.ReadFile(codeFileFlag); err != nil {
					fmt.Printf("Could not load code from file: %v\n", err)
					os.Exit(1)
				}
			}
		} else {
			hexcode = []byte(codeFlag)
		}
		hexcode = bytes.TrimSpace(hexcode)
		if len(hexcode)%2 != 0 {
			fmt.Printf("Invalid input length for hex data (%d)\n", len(hexcode))
			os.Exit(1)
		}
		code = common.FromHex(string(hexcode))
	} else if fn := ctx.Args().First(); len(fn) > 0 {
		// EASM-file to compile
		src, err := os.ReadFile(fn)
		if err != nil {
			return err
		}
		bin, err := compiler.Compile(fn, src, false)
		if err != nil {
			return err
		}
		code = common.Hex2Bytes(bin)
	}
	runtimeConfig := runtime.Config{
		Origin:      sender,
		State:       statedb,
		GasLimit:    initialGas,
		GasPrice:    flags.GlobalBig(ctx, PriceFlag.Name),
		Value:       flags.GlobalBig(ctx, ValueFlag.Name),
		Difficulty:  genesisConfig.Difficulty,
		Time:        genesisConfig.Timestamp,
		Coinbase:    genesisConfig.Coinbase,
		BlockNumber: new(big.Int).SetUint64(genesisConfig.Number),
		BlobHashes:  blobHashes,
		BlobBaseFee: blobBaseFee,
		EVMConfig: vm.Config{
			Tracer: tracer,
		},
	}

	if chainConfig != nil {
		runtimeConfig.ChainConfig = chainConfig
	} else {
		runtimeConfig.ChainConfig = params.AllEthashProtocolChanges
	}

	var hexInput []byte
	if inputFileFlag := ctx.String(InputFileFlag.Name); inputFileFlag != "" {
		var err error
		if hexInput, err = os.ReadFile(inputFileFlag); err != nil {
			fmt.Printf("could not load input from file: %v\n", err)
			os.Exit(1)
		}
	} else {
		hexInput = []byte(ctx.String(InputFlag.Name))
	}
	hexInput = bytes.TrimSpace(hexInput)
	if len(hexInput)%2 != 0 {
		fmt.Println("input length must be even")
		os.Exit(1)
	}
	input := common.FromHex(string(hexInput))

	var execFunc func() ([]byte, uint64, error)
	if ctx.Bool(CreateFlag.Name) {
		input = append(code, input...)
		execFunc = func() ([]byte, uint64, error) {
			output, _, gasLeft, err := runtime.Create(input, &runtimeConfig)
			return output, gasLeft, err
		}
	} else {
		if len(code) > 0 {
			statedb.SetCode(receiver, code)
		}
		execFunc = func() ([]byte, uint64, error) {
			return runtime.Call(receiver, input, &runtimeConfig)
		}
	}

	bench := ctx.Bool(BenchFlag.Name)
	output, leftOverGas, stats, err := timedExec(bench, execFunc)

	if ctx.Bool(DumpFlag.Name) {
		statedb.Commit(genesisConfig.Number, true)
		fmt.Println(string(statedb.Dump(nil)))
	}

	if ctx.Bool(DebugFlag.Name) {
		if debugLogger != nil {
			fmt.Fprintln(os.Stderr, "#### TRACE ####")
			logger.WriteTrace(os.Stderr, debugLogger.StructLogs())
		}
		fmt.Fprintln(os.Stderr, "#### LOGS ####")
		logger.WriteLogs(os.Stderr, statedb.Logs())
	}

	if bench || ctx.Bool(StatDumpFlag.Name) {
		fmt.Fprintf(os.Stderr, `EVM gas used:    %d
execution time:  %v
allocations:     %d
allocated bytes: %d
`, initialGas-leftOverGas, stats.time, stats.allocs, stats.bytesAllocated)
	}
	if tracer == nil {
		fmt.Printf("%#x\n", output)
		if err != nil {
			fmt.Printf(" error: %v\n", err)
		}
	}

	return nil
}
