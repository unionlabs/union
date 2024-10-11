package simulation

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"slices"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/header"
	corelog "cosmossdk.io/core/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

const AverageBlockTime = 6 * time.Second

// initialize the chain for the simulation
func initChain(
	r *rand.Rand,
	params Params,
	accounts []simtypes.Account,
	app *baseapp.BaseApp,
	appStateFn simtypes.AppStateFn,
	config simtypes.Config,
	cdc codec.JSONCodec,
) (mockValidators, time.Time, []simtypes.Account, string) {
	blockMaxGas := int64(-1)
	if config.BlockMaxGas > 0 {
		blockMaxGas = config.BlockMaxGas
	}
	appState, accounts, chainID, genesisTimestamp := appStateFn(r, accounts, config)
	consensusParams := RandomConsensusParams(r, appState, cdc, blockMaxGas)
	req := abci.InitChainRequest{
		AppStateBytes:   appState,
		ChainId:         chainID,
		ConsensusParams: consensusParams,
		Time:            genesisTimestamp,
		InitialHeight:   int64(config.InitialBlockHeight),
	}
	res, err := app.InitChain(&req)
	if err != nil {
		panic(err)
	}
	validators := newMockValidators(r, res.Validators, params)

	return validators, genesisTimestamp, accounts, chainID
}

// SimulateFromSeed tests an application by running the provided
// operations, testing the provided invariants, but using the provided config.Seed.
func SimulateFromSeed(
	tb testing.TB,
	logger corelog.Logger,
	w io.Writer,
	app *baseapp.BaseApp,
	appStateFn simtypes.AppStateFn,
	randAccFn simtypes.RandomAccountFn,
	ops WeightedOperations,
	blockedAddrs map[string]bool,
	config simtypes.Config,
	cdc codec.JSONCodec,
	addressCodec address.Codec,
) (exportedParams Params, accs []simtypes.Account, err error) {
	tb.Helper()
	mode, _, _ := getTestingMode(tb)
	return SimulateFromSeedX(tb, logger, w, app, appStateFn, randAccFn, ops, blockedAddrs, config, cdc, NewLogWriter(mode))
}

// SimulateFromSeedX tests an application by running the provided
// operations, testing the provided invariants, but using the provided config.Seed.
func SimulateFromSeedX(
	tb testing.TB,
	logger corelog.Logger,
	w io.Writer,
	app *baseapp.BaseApp,
	appStateFn simtypes.AppStateFn,
	randAccFn simtypes.RandomAccountFn,
	ops WeightedOperations,
	blockedAddrs map[string]bool,
	config simtypes.Config,
	cdc codec.JSONCodec,
	logWriter LogWriter,
) (exportedParams Params, accs []simtypes.Account, err error) {
	tb.Helper()
	defer func() {
		if err != nil {
			logWriter.PrintLogs()
		}
	}()
	// in case we have to end early, don't os.Exit so that we can run cleanup code.
	testingMode, _, b := getTestingMode(tb)

	r := rand.New(NewByteSource(config.FuzzSeed, config.Seed))
	params := RandomParams(r)

	startTime := time.Now()
	logger.Info("Starting SimulateFromSeed with randomness", "time", startTime)
	logger.Debug("Randomized simulation setup", "params", mustMarshalJSONIndent(params))

	timeDiff := maxTimePerBlock - minTimePerBlock
	accs = randAccFn(r, params.NumKeys())
	eventStats := NewEventStats()

	// Second variable to keep pending validator set (delayed one block since
	// TM 0.24) Initially this is the same as the initial validator set
	validators, blockTime, accs, chainID := initChain(r, params, accs, app, appStateFn, config, cdc)
	// At least 2 accounts must be added here, otherwise when executing SimulateMsgSend
	// two accounts will be selected to meet the conditions from != to and it will fall into an infinite loop.
	if len(accs) <= 1 {
		return params, accs, errors.New("at least two genesis accounts are required")
	}

	config.ChainID = chainID

	// remove module account address if they exist in accs
	accs = slices.DeleteFunc(accs, func(acc simtypes.Account) bool {
		return blockedAddrs[acc.AddressBech32]
	})
	nextValidators := validators
	if len(nextValidators) == 0 {
		tb.Skip("skipping: empty validator set in genesis")
		return params, accs, nil
	}

	var (
		pastTimes          []time.Time
		pastVoteInfos      [][]abci.VoteInfo
		timeOperationQueue []simtypes.FutureOperation

		blockHeight     = int64(config.InitialBlockHeight)
		proposerAddress = validators.randomProposer(r)
		opCount         = 0
	)

	finalizeBlockReq := RandomRequestFinalizeBlock(
		r,
		params,
		validators,
		pastTimes,
		pastVoteInfos,
		eventStats.Tally,
		blockHeight,
		blockTime,
		validators.randomProposer(r),
	)

	// These are operations which have been queued by previous operations
	operationQueue := NewOperationQueue()

	blockSimulator := createBlockSimulator(
		tb,
		testingMode,
		w,
		params,
		eventStats.Tally,
		ops,
		operationQueue,
		&timeOperationQueue,
		logWriter,
		config,
	)

	if !testingMode {
		b.ResetTimer()
	} else {
		// recover logs in case of panic
		defer func() {
			if r := recover(); r != nil {
				logger.Error("simulation halted due to panic", "height", blockHeight)
				logWriter.PrintLogs()
				panic(r)
			}
		}()
	}

	// set exported params to the initial state
	if config.ExportParamsPath != "" && config.ExportParamsHeight == 0 {
		exportedParams = params
	}

	if _, err := app.FinalizeBlock(finalizeBlockReq); err != nil {
		return params, accs, fmt.Errorf("block finalization failed at height %d: %+w", blockHeight, err)
	}

	for blockHeight < int64(config.NumBlocks+config.InitialBlockHeight) {
		pastTimes = append(pastTimes, blockTime)
		pastVoteInfos = append(pastVoteInfos, finalizeBlockReq.DecidedLastCommit.Votes)

		// Run the BeginBlock handler
		logWriter.AddEntry(BeginBlockEntry(blockTime, blockHeight))

		res, err := app.FinalizeBlock(finalizeBlockReq)
		if err != nil {
			return params, accs, fmt.Errorf("block finalization failed at height %d: %w", blockHeight, err)
		}

		ctx := app.NewContextLegacy(false, cmtproto.Header{
			Height:          blockHeight,
			Time:            blockTime,
			ProposerAddress: proposerAddress,
			ChainID:         config.ChainID,
		}).WithHeaderInfo(header.Info{
			Height:  blockHeight,
			Time:    blockTime,
			ChainID: config.ChainID,
		})

		// run queued operations; ignores block size if block size is too small
		numQueuedOpsRan, futureOps := runQueuedOperations(
			tb, operationQueue, blockTime, int(blockHeight), r, app, ctx, accs, logWriter,
			eventStats.Tally, config.Lean, config.ChainID,
		)
		numQueuedTimeOpsRan, timeFutureOps := runQueuedTimeOperations(tb,
			&timeOperationQueue, int(blockHeight), blockTime,
			r, app, ctx, accs, logWriter, eventStats.Tally,
			config.Lean, config.ChainID,
		)

		futureOps = append(futureOps, timeFutureOps...)
		queueOperations(operationQueue, &timeOperationQueue, futureOps)

		// run standard operations
		operations := blockSimulator(r, app, ctx, accs, cmtproto.Header{
			Height:          blockHeight,
			Time:            blockTime,
			ProposerAddress: proposerAddress,
			ChainID:         config.ChainID,
		})
		opCount += operations + numQueuedOpsRan + numQueuedTimeOpsRan
		blockHeight++

		logWriter.AddEntry(EndBlockEntry(blockTime, blockHeight))

		blockTime = blockTime.Add(time.Duration(minTimePerBlock) * time.Second)
		blockTime = blockTime.Add(time.Duration(int64(r.Intn(int(timeDiff)))) * time.Second)
		proposerAddress = validators.randomProposer(r)

		if config.Commit {
			app.SimWriteState()
			if _, err := app.Commit(); err != nil {
				return params, accs, fmt.Errorf("commit failed at height %d: %w", blockHeight, err)
			}
		}

		if proposerAddress == nil {
			logger.Info("Simulation stopped early as all validators have been unbonded; nobody left to propose a block", "height", blockHeight)
			break
		}

		// Generate a random RequestBeginBlock with the current validator set
		// for the next block
		finalizeBlockReq = RandomRequestFinalizeBlock(r, params, validators, pastTimes, pastVoteInfos, eventStats.Tally, blockHeight, blockTime, proposerAddress)

		// Update the validator set, which will be reflected in the application
		// on the next block
		validators = nextValidators
		nextValidators = updateValidators(tb, r, params, validators, res.ValidatorUpdates, eventStats.Tally)
		if len(nextValidators) == 0 {
			tb.Skip("skipping: empty validator set")
			return exportedParams, accs, err
		}

		// update the exported params
		if config.ExportParamsPath != "" && int64(config.ExportParamsHeight) == blockHeight {
			exportedParams = params
		}
	}
	logger.Info("Simulation complete", "height", blockHeight, "block-time", blockTime, "opsCount", opCount,
		"run-time", time.Since(startTime), "app-hash", hex.EncodeToString(app.LastCommitID().Hash))

	if config.ExportStatsPath != "" {
		fmt.Println("Exporting simulation statistics...")
		eventStats.ExportJSON(config.ExportStatsPath)
	} else {
		eventStats.Print(w)
	}
	return exportedParams, accs, err
}

type blockSimFn func(
	r *rand.Rand,
	app simtypes.AppEntrypoint,
	ctx sdk.Context,
	accounts []simtypes.Account,
	header cmtproto.Header,
) (opCount int)

// Returns a function to simulate blocks. Written like this to avoid constant
// parameters being passed every time, to minimize memory overhead.
func createBlockSimulator(tb testing.TB, printProgress bool, w io.Writer, params Params,
	event func(route, op, evResult string), ops WeightedOperations,
	operationQueue OperationQueue, timeOperationQueue *[]simtypes.FutureOperation,
	logWriter LogWriter, config simtypes.Config,
) blockSimFn {
	tb.Helper()
	lastBlockSizeState := 0 // state for [4 * uniform distribution]
	blocksize := 0
	selectOp := ops.getSelectOpFn()

	return func(
		r *rand.Rand, app simtypes.AppEntrypoint, ctx sdk.Context, accounts []simtypes.Account, header cmtproto.Header,
	) (opCount int) {
		_, _ = fmt.Fprintf(
			w, "\rSimulating... block %d/%d, operation %d/%d.",
			header.Height, config.NumBlocks, opCount, blocksize,
		)
		lastBlockSizeState, blocksize = getBlockSize(r, params, lastBlockSizeState, config.BlockSize)

		type opAndR struct {
			op   simtypes.Operation
			rand *rand.Rand
		}

		opAndRz := make([]opAndR, 0, blocksize)

		// Predetermine the blocksize slice so that we can do things like block
		// out certain operations without changing the ops that follow.
		for i := 0; i < blocksize; i++ {
			opAndRz = append(opAndRz, opAndR{
				op:   selectOp(r),
				rand: r,
			})
		}

		for i := 0; i < blocksize; i++ {
			// NOTE: the Rand 'r' should not be used here.
			opAndR := opAndRz[i]
			op, r2 := opAndR.op, opAndR.rand
			opMsg, futureOps, err := op(r2, app, ctx, accounts, config.ChainID)
			opMsg.LogEvent(event)

			if !config.Lean || opMsg.OK {
				logWriter.AddEntry(MsgEntry(header.Time, header.Height, int64(i), opMsg))
			}

			if err != nil {
				logWriter.PrintLogs()
				tb.Fatalf(`error on block  %d/%d, operation (%d/%d) from x/%s:
%v
Comment: %s`,
					header.Height, config.NumBlocks, opCount, blocksize, opMsg.Route, err, opMsg.Comment)
			}

			queueOperations(operationQueue, timeOperationQueue, futureOps)

			if printProgress && opCount%50 == 0 {
				_, _ = fmt.Fprintf(w, "\rSimulating... block %d/%d, operation %d/%d. ",
					header.Height, config.NumBlocks, opCount, blocksize)
			}

			opCount++
		}

		return opCount
	}
}

func runQueuedOperations(tb testing.TB, queueOps map[int][]simtypes.Operation,
	blockTime time.Time, height int, r *rand.Rand, app *baseapp.BaseApp,
	ctx sdk.Context, accounts []simtypes.Account, logWriter LogWriter,
	event func(route, op, evResult string), lean bool, chainID string,
) (numOpsRan int, allFutureOps []simtypes.FutureOperation) {
	tb.Helper()
	queuedOp, ok := queueOps[height]
	if !ok {
		return 0, nil
	}

	// Keep all future operations
	allFutureOps = make([]simtypes.FutureOperation, 0)

	numOpsRan = len(queuedOp)
	for i := 0; i < numOpsRan; i++ {
		opMsg, futureOps, err := queuedOp[i](r, app, ctx, accounts, chainID)
		if err != nil {
			logWriter.PrintLogs()
			tb.FailNow()
		}
		if len(futureOps) > 0 {
			allFutureOps = append(allFutureOps, futureOps...)
		}

		opMsg.LogEvent(event)

		if !lean || opMsg.OK {
			logWriter.AddEntry(QueuedMsgEntry(blockTime, int64(height), opMsg))
		}

	}
	delete(queueOps, height)

	return numOpsRan, allFutureOps
}

func runQueuedTimeOperations(tb testing.TB, queueOps *[]simtypes.FutureOperation,
	height int, currentTime time.Time, r *rand.Rand,
	app *baseapp.BaseApp, ctx sdk.Context, accounts []simtypes.Account,
	logWriter LogWriter, event func(route, op, evResult string),
	lean bool, chainID string,
) (numOpsRan int, allFutureOps []simtypes.FutureOperation) {
	tb.Helper()
	// Keep all future operations
	numOpsRan = 0
	for len(*queueOps) > 0 && currentTime.After((*queueOps)[0].BlockTime) {
		if qOp := (*queueOps)[0]; qOp.Op != nil {
			opMsg, futureOps, err := qOp.Op(r, app, ctx, accounts, chainID)

			opMsg.LogEvent(event)

			if !lean || opMsg.OK {
				logWriter.AddEntry(QueuedMsgEntry(currentTime, int64(height), opMsg))
			}

			if err != nil {
				logWriter.PrintLogs()
				tb.Fatal(err)
			}

			if len(futureOps) > 0 {
				allFutureOps = append(allFutureOps, futureOps...)
			}
		}
		*queueOps = slices.Delete(*queueOps, 0, 1)
		numOpsRan++
	}

	return numOpsRan, allFutureOps
}

const (
	rngMax  = 1 << 63
	rngMask = rngMax - 1
)

// ByteSource offers deterministic pseudo-random numbers for math.Rand with fuzzer support.
// The 'seed' data is read in big endian to uint64. When exhausted,
// it falls back to a standard random number generator initialized with a specific 'seed' value.
type ByteSource struct {
	seed     *bytes.Reader
	fallback *rand.Rand
}

// NewByteSource creates a new ByteSource with a specified byte slice and seed. This gives a fixed sequence of pseudo-random numbers.
// Initially, it utilizes the byte slice. Once that's exhausted, it continues generating numbers using the provided seed.
func NewByteSource(fuzzSeed []byte, seed int64) *ByteSource {
	return &ByteSource{
		seed:     bytes.NewReader(fuzzSeed),
		fallback: rand.New(rand.NewSource(seed)),
	}
}

func (s *ByteSource) Uint64() uint64 {
	if s.seed.Len() < 8 {
		return s.fallback.Uint64()
	}
	var b [8]byte
	if _, err := s.seed.Read(b[:]); err != nil && err != io.EOF {
		panic(err) // Should not happen.
	}
	return binary.BigEndian.Uint64(b[:])
}

func (s *ByteSource) Int63() int64 {
	return int64(s.Uint64() & rngMask)
}
func (s *ByteSource) Seed(seed int64) {}
