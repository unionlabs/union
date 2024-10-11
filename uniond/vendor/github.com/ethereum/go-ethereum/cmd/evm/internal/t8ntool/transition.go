// Copyright 2020 The go-ethereum Authors
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

package t8ntool

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"os"
	"path"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers/logger"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/tests"
	"github.com/urfave/cli/v2"
)

const (
	ErrorEVM              = 2
	ErrorConfig           = 3
	ErrorMissingBlockhash = 4

	ErrorJson = 10
	ErrorIO   = 11
	ErrorRlp  = 12

	stdinSelector = "stdin"
)

type NumberedError struct {
	errorCode int
	err       error
}

func NewError(errorCode int, err error) *NumberedError {
	return &NumberedError{errorCode, err}
}

func (n *NumberedError) Error() string {
	return fmt.Sprintf("ERROR(%d): %v", n.errorCode, n.err.Error())
}

func (n *NumberedError) ExitCode() int {
	return n.errorCode
}

// compile-time conformance test
var (
	_ cli.ExitCoder = (*NumberedError)(nil)
)

type input struct {
	Alloc core.GenesisAlloc `json:"alloc,omitempty"`
	Env   *stEnv            `json:"env,omitempty"`
	Txs   []*txWithKey      `json:"txs,omitempty"`
	TxRlp string            `json:"txsRlp,omitempty"`
}

func Transition(ctx *cli.Context) error {
	// Configure the go-ethereum logger
	glogger := log.NewGlogHandler(log.StreamHandler(os.Stderr, log.TerminalFormat(false)))
	glogger.Verbosity(log.Lvl(ctx.Int(VerbosityFlag.Name)))
	log.Root().SetHandler(glogger)

	var (
		err    error
		tracer vm.EVMLogger
	)
	var getTracer func(txIndex int, txHash common.Hash) (vm.EVMLogger, error)

	baseDir, err := createBasedir(ctx)
	if err != nil {
		return NewError(ErrorIO, fmt.Errorf("failed creating output basedir: %v", err))
	}
	if ctx.Bool(TraceFlag.Name) {
		if ctx.IsSet(TraceDisableMemoryFlag.Name) && ctx.IsSet(TraceEnableMemoryFlag.Name) {
			return NewError(ErrorConfig, fmt.Errorf("can't use both flags --%s and --%s", TraceDisableMemoryFlag.Name, TraceEnableMemoryFlag.Name))
		}
		if ctx.IsSet(TraceDisableReturnDataFlag.Name) && ctx.IsSet(TraceEnableReturnDataFlag.Name) {
			return NewError(ErrorConfig, fmt.Errorf("can't use both flags --%s and --%s", TraceDisableReturnDataFlag.Name, TraceEnableReturnDataFlag.Name))
		}
		if ctx.IsSet(TraceDisableMemoryFlag.Name) {
			log.Warn(fmt.Sprintf("--%s has been deprecated in favour of --%s", TraceDisableMemoryFlag.Name, TraceEnableMemoryFlag.Name))
		}
		if ctx.IsSet(TraceDisableReturnDataFlag.Name) {
			log.Warn(fmt.Sprintf("--%s has been deprecated in favour of --%s", TraceDisableReturnDataFlag.Name, TraceEnableReturnDataFlag.Name))
		}
		// Configure the EVM logger
		logConfig := &logger.Config{
			DisableStack:     ctx.Bool(TraceDisableStackFlag.Name),
			EnableMemory:     !ctx.Bool(TraceDisableMemoryFlag.Name) || ctx.Bool(TraceEnableMemoryFlag.Name),
			EnableReturnData: !ctx.Bool(TraceDisableReturnDataFlag.Name) || ctx.Bool(TraceEnableReturnDataFlag.Name),
			Debug:            true,
		}
		var prevFile *os.File
		// This one closes the last file
		defer func() {
			if prevFile != nil {
				prevFile.Close()
			}
		}()
		getTracer = func(txIndex int, txHash common.Hash) (vm.EVMLogger, error) {
			if prevFile != nil {
				prevFile.Close()
			}
			traceFile, err := os.Create(path.Join(baseDir, fmt.Sprintf("trace-%d-%v.jsonl", txIndex, txHash.String())))
			if err != nil {
				return nil, NewError(ErrorIO, fmt.Errorf("failed creating trace-file: %v", err))
			}
			prevFile = traceFile
			return logger.NewJSONLogger(logConfig, traceFile), nil
		}
	} else {
		getTracer = func(txIndex int, txHash common.Hash) (tracer vm.EVMLogger, err error) {
			return nil, nil
		}
	}
	// We need to load three things: alloc, env and transactions. May be either in
	// stdin input or in files.
	// Check if anything needs to be read from stdin
	var (
		prestate Prestate
		txIt     txIterator // txs to apply
		allocStr = ctx.String(InputAllocFlag.Name)

		envStr    = ctx.String(InputEnvFlag.Name)
		txStr     = ctx.String(InputTxsFlag.Name)
		inputData = &input{}
	)
	// Figure out the prestate alloc
	if allocStr == stdinSelector || envStr == stdinSelector || txStr == stdinSelector {
		decoder := json.NewDecoder(os.Stdin)
		if err := decoder.Decode(inputData); err != nil {
			return NewError(ErrorJson, fmt.Errorf("failed unmarshaling stdin: %v", err))
		}
	}
	if allocStr != stdinSelector {
		if err := readFile(allocStr, "alloc", &inputData.Alloc); err != nil {
			return err
		}
	}
	prestate.Pre = inputData.Alloc

	// Set the block environment
	if envStr != stdinSelector {
		var env stEnv
		if err := readFile(envStr, "env", &env); err != nil {
			return err
		}
		inputData.Env = &env
	}
	prestate.Env = *inputData.Env

	vmConfig := vm.Config{
		Tracer: tracer,
	}
	// Construct the chainconfig
	var chainConfig *params.ChainConfig
	if cConf, extraEips, err := tests.GetChainConfig(ctx.String(ForknameFlag.Name)); err != nil {
		return NewError(ErrorConfig, fmt.Errorf("failed constructing chain configuration: %v", err))
	} else {
		chainConfig = cConf
		vmConfig.ExtraEips = extraEips
	}
	// Set the chain id
	chainConfig.ChainID = big.NewInt(ctx.Int64(ChainIDFlag.Name))

	if txIt, err = loadTransactions(txStr, inputData, prestate.Env, chainConfig); err != nil {
		return err
	}
	if err := applyLondonChecks(&prestate.Env, chainConfig); err != nil {
		return err
	}
	if err := applyShanghaiChecks(&prestate.Env, chainConfig); err != nil {
		return err
	}
	if err := applyMergeChecks(&prestate.Env, chainConfig); err != nil {
		return err
	}
	if err := applyCancunChecks(&prestate.Env, chainConfig); err != nil {
		return err
	}
	// Run the test and aggregate the result
	s, result, body, err := prestate.Apply(vmConfig, chainConfig, txIt, ctx.Int64(RewardFlag.Name), getTracer)
	if err != nil {
		return err
	}
	// Dump the excution result
	collector := make(Alloc)
	s.DumpToCollector(collector, nil)
	return dispatchOutput(ctx, baseDir, result, collector, body)
}

func applyLondonChecks(env *stEnv, chainConfig *params.ChainConfig) error {
	if !chainConfig.IsLondon(big.NewInt(int64(env.Number))) {
		return nil
	}
	// Sanity check, to not `panic` in state_transition
	if env.BaseFee != nil {
		// Already set, base fee has precedent over parent base fee.
		return nil
	}
	if env.ParentBaseFee == nil || env.Number == 0 {
		return NewError(ErrorConfig, errors.New("EIP-1559 config but missing 'currentBaseFee' in env section"))
	}
	env.BaseFee = eip1559.CalcBaseFee(chainConfig, &types.Header{
		Number:   new(big.Int).SetUint64(env.Number - 1),
		BaseFee:  env.ParentBaseFee,
		GasUsed:  env.ParentGasUsed,
		GasLimit: env.ParentGasLimit,
	})
	return nil
}

func applyShanghaiChecks(env *stEnv, chainConfig *params.ChainConfig) error {
	if !chainConfig.IsShanghai(big.NewInt(int64(env.Number)), env.Timestamp) {
		return nil
	}
	if env.Withdrawals == nil {
		return NewError(ErrorConfig, errors.New("Shanghai config but missing 'withdrawals' in env section"))
	}
	return nil
}

func applyMergeChecks(env *stEnv, chainConfig *params.ChainConfig) error {
	isMerged := chainConfig.TerminalTotalDifficulty != nil && chainConfig.TerminalTotalDifficulty.BitLen() == 0
	if !isMerged {
		// pre-merge: If difficulty was not provided by caller, we need to calculate it.
		if env.Difficulty != nil {
			// already set
			return nil
		}
		switch {
		case env.ParentDifficulty == nil:
			return NewError(ErrorConfig, errors.New("currentDifficulty was not provided, and cannot be calculated due to missing parentDifficulty"))
		case env.Number == 0:
			return NewError(ErrorConfig, errors.New("currentDifficulty needs to be provided for block number 0"))
		case env.Timestamp <= env.ParentTimestamp:
			return NewError(ErrorConfig, fmt.Errorf("currentDifficulty cannot be calculated -- currentTime (%d) needs to be after parent time (%d)",
				env.Timestamp, env.ParentTimestamp))
		}
		env.Difficulty = calcDifficulty(chainConfig, env.Number, env.Timestamp,
			env.ParentTimestamp, env.ParentDifficulty, env.ParentUncleHash)
		return nil
	}
	// post-merge:
	// - random must be supplied
	// - difficulty must be zero
	switch {
	case env.Random == nil:
		return NewError(ErrorConfig, errors.New("post-merge requires currentRandom to be defined in env"))
	case env.Difficulty != nil && env.Difficulty.BitLen() != 0:
		return NewError(ErrorConfig, errors.New("post-merge difficulty must be zero (or omitted) in env"))
	}
	env.Difficulty = nil
	return nil
}

func applyCancunChecks(env *stEnv, chainConfig *params.ChainConfig) error {
	if !chainConfig.IsCancun(big.NewInt(int64(env.Number)), env.Timestamp) {
		env.ParentBeaconBlockRoot = nil // un-set it if it has been set too early
		return nil
	}
	// Post-cancun
	// We require EIP-4788 beacon root to be set in the env
	if env.ParentBeaconBlockRoot == nil {
		return NewError(ErrorConfig, errors.New("post-cancun env requires parentBeaconBlockRoot to be set"))
	}
	return nil
}

type Alloc map[common.Address]core.GenesisAccount

func (g Alloc) OnRoot(common.Hash) {}

func (g Alloc) OnAccount(addr *common.Address, dumpAccount state.DumpAccount) {
	if addr == nil {
		return
	}
	balance, _ := new(big.Int).SetString(dumpAccount.Balance, 10)
	var storage map[common.Hash]common.Hash
	if dumpAccount.Storage != nil {
		storage = make(map[common.Hash]common.Hash)
		for k, v := range dumpAccount.Storage {
			storage[k] = common.HexToHash(v)
		}
	}
	genesisAccount := core.GenesisAccount{
		Code:    dumpAccount.Code,
		Storage: storage,
		Balance: balance,
		Nonce:   dumpAccount.Nonce,
	}
	g[*addr] = genesisAccount
}

// saveFile marshals the object to the given file
func saveFile(baseDir, filename string, data interface{}) error {
	b, err := json.MarshalIndent(data, "", " ")
	if err != nil {
		return NewError(ErrorJson, fmt.Errorf("failed marshalling output: %v", err))
	}
	location := path.Join(baseDir, filename)
	if err = os.WriteFile(location, b, 0644); err != nil {
		return NewError(ErrorIO, fmt.Errorf("failed writing output: %v", err))
	}
	log.Info("Wrote file", "file", location)
	return nil
}

// dispatchOutput writes the output data to either stderr or stdout, or to the specified
// files
func dispatchOutput(ctx *cli.Context, baseDir string, result *ExecutionResult, alloc Alloc, body hexutil.Bytes) error {
	stdOutObject := make(map[string]interface{})
	stdErrObject := make(map[string]interface{})
	dispatch := func(baseDir, fName, name string, obj interface{}) error {
		switch fName {
		case "stdout":
			stdOutObject[name] = obj
		case "stderr":
			stdErrObject[name] = obj
		case "":
			// don't save
		default: // save to file
			if err := saveFile(baseDir, fName, obj); err != nil {
				return err
			}
		}
		return nil
	}
	if err := dispatch(baseDir, ctx.String(OutputAllocFlag.Name), "alloc", alloc); err != nil {
		return err
	}
	if err := dispatch(baseDir, ctx.String(OutputResultFlag.Name), "result", result); err != nil {
		return err
	}
	if err := dispatch(baseDir, ctx.String(OutputBodyFlag.Name), "body", body); err != nil {
		return err
	}
	if len(stdOutObject) > 0 {
		b, err := json.MarshalIndent(stdOutObject, "", "  ")
		if err != nil {
			return NewError(ErrorJson, fmt.Errorf("failed marshalling output: %v", err))
		}
		os.Stdout.Write(b)
		os.Stdout.WriteString("\n")
	}
	if len(stdErrObject) > 0 {
		b, err := json.MarshalIndent(stdErrObject, "", "  ")
		if err != nil {
			return NewError(ErrorJson, fmt.Errorf("failed marshalling output: %v", err))
		}
		os.Stderr.Write(b)
		os.Stderr.WriteString("\n")
	}
	return nil
}
