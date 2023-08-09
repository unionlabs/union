package simulation

import (
	"fmt"
	"io"
	"math/rand"
	"os"
	"os/signal"
	"syscall"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

const AverageBlockTime = 6 * time.Second

// initialize the chain for the simulation
func initChain(
	r *rand.Rand,
	params Params,
	accounts []simulation.Account,
	app *baseapp.BaseApp,
	appStateFn simulation.AppStateFn,
	config simulation.Config,
	cdc codec.JSONCodec,
) (mockValidators, time.Time, []simulation.Account, string) {
	blockMaxGas := int64(-1)
	if config.BlockMaxGas > 0 {
		blockMaxGas = config.BlockMaxGas
	}
	appState, accounts, chainID, genesisTimestamp := appStateFn(r, accounts, config)
	consensusParams := randomConsensusParams(r, appState, cdc, blockMaxGas)
	req := abci.RequestInitChain{
		AppStateBytes:   appState,
		ChainId:         chainID,
		ConsensusParams: consensusParams,
		Time:            genesisTimestamp,
	}
	res := app.InitChain(req)
	validators := newMockValidators(r, res.Validators, params)

	return validators, genesisTimestamp, accounts, chainID
}

// SimulateFromSeed tests an application by running the provided
// operations, testing the provided invariants, but using the provided config.Seed.
// TODO: split this monster function up
func SimulateFromSeed(
	tb testing.TB,
	w io.Writer,
	app *baseapp.BaseApp,
	appStateFn simulation.AppStateFn,
	randAccFn simulation.RandomAccountFn,
	ops WeightedOperations,
	blockedAddrs map[string]bool,
	config simulation.Config,
	cdc codec.JSONCodec,
) (stopEarly bool, exportedParams Params, err error) {
	// in case we have to end early, don't os.Exit so that we can run cleanup code.
	testingMode, _, b := getTestingMode(tb)

	fmt.Fprintf(w, "Starting SimulateFromSeed with randomness created with seed %d\n", int(config.Seed))
	r := rand.New(rand.NewSource(config.Seed))
	params := RandomParams(r)
	fmt.Fprintf(w, "Randomized simulation params: \n%s\n", mustMarshalJSONIndent(params))

	timeDiff := maxTimePerBlock - minTimePerBlock
	accs := randAccFn(r, params.NumKeys())
	eventStats := NewEventStats()

	// Second variable to keep pending validator set (delayed one block since
	// TM 0.24) Initially this is the same as the initial validator set
	validators, genesisTimestamp, accs, chainID := initChain(r, params, accs, app, appStateFn, config, cdc)
	if len(accs) == 0 {
		return true, params, fmt.Errorf("must have greater than zero genesis accounts")
	}

	config.ChainID = chainID

	fmt.Printf(
		"Starting the simulation from time %v (unixtime %v)\n",
		genesisTimestamp.UTC().Format(time.UnixDate), genesisTimestamp.Unix(),
	)

	// remove module account address if they exist in accs
	var tmpAccs []simulation.Account

	for _, acc := range accs {
		if !blockedAddrs[acc.Address.String()] {
			tmpAccs = append(tmpAccs, acc)
		}
	}

	accs = tmpAccs
	nextValidators := validators

	header := tmproto.Header{
		ChainID:         config.ChainID,
		Height:          1,
		Time:            genesisTimestamp,
		ProposerAddress: validators.randomProposer(r),
	}
	opCount := 0

	// Setup code to catch SIGTERM's
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM, syscall.SIGINT)

	go func() {
		receivedSignal := <-c
		fmt.Fprintf(w, "\nExiting early due to %s, on block %d, operation %d\n", receivedSignal, header.Height, opCount)
		err = fmt.Errorf("exited due to %s", receivedSignal)
		stopEarly = true
	}()

	var (
		pastTimes     []time.Time
		pastVoteInfos [][]abci.VoteInfo
	)

	request := RandomRequestBeginBlock(r, params,
		validators, pastTimes, pastVoteInfos, eventStats.Tally, header)

	// These are operations which have been queued by previous operations
	operationQueue := NewOperationQueue()

	var timeOperationQueue []simulation.FutureOperation

	logWriter := NewLogWriter(testingMode)

	blockSimulator := createBlockSimulator(
		testingMode, tb, w, params, eventStats.Tally,
		ops, operationQueue, timeOperationQueue, logWriter, config)

	if !testingMode {
		b.ResetTimer()
	} else {
		// recover logs in case of panic
		defer func() {
			if r := recover(); r != nil {
				_, _ = fmt.Fprintf(w, "simulation halted due to panic on block %d\n", header.Height)
				logWriter.PrintLogs()
				panic(r)
			}
		}()
	}

	// set exported params to the initial state
	if config.ExportParamsPath != "" && config.ExportParamsHeight == 0 {
		exportedParams = params
	}

	// TODO: split up the contents of this for loop into new functions
	for height := config.InitialBlockHeight; height < config.NumBlocks+config.InitialBlockHeight && !stopEarly; height++ {

		// Log the header time for future lookup
		pastTimes = append(pastTimes, header.Time)
		pastVoteInfos = append(pastVoteInfos, request.LastCommitInfo.Votes)

		// Run the BeginBlock handler
		logWriter.AddEntry(BeginBlockEntry(int64(height)))
		app.BeginBlock(request)

		ctx := app.NewContext(false, header)

		// Run queued operations. Ignores blocksize if blocksize is too small
		numQueuedOpsRan, futureOps := runQueuedOperations(
			operationQueue, int(header.Height), tb, r, app, ctx, accs, logWriter,
			eventStats.Tally, config.Lean, config.ChainID,
		)

		numQueuedTimeOpsRan, timeFutureOps := runQueuedTimeOperations(
			timeOperationQueue, int(header.Height), header.Time,
			tb, r, app, ctx, accs, logWriter, eventStats.Tally,
			config.Lean, config.ChainID,
		)

		futureOps = append(futureOps, timeFutureOps...)
		queueOperations(operationQueue, timeOperationQueue, futureOps)

		// run standard operations
		operations := blockSimulator(r, app, ctx, accs, header)
		opCount += operations + numQueuedOpsRan + numQueuedTimeOpsRan

		res := app.EndBlock(abci.RequestEndBlock{})
		header.Height++
		header.Time = header.Time.Add(
			time.Duration(minTimePerBlock) * time.Second)
		header.Time = header.Time.Add(
			time.Duration(int64(r.Intn(int(timeDiff)))) * time.Second)
		header.ProposerAddress = validators.randomProposer(r)

		logWriter.AddEntry(EndBlockEntry(int64(height)))

		if config.Commit {
			app.Commit()
		}

		if header.ProposerAddress == nil {
			fmt.Fprintf(w, "\nSimulation stopped early as all validators have been unbonded; nobody left to propose a block!\n")
			stopEarly = true
			break
		}

		// Generate a random RequestBeginBlock with the current validator set
		// for the next block
		request = RandomRequestBeginBlock(r, params, validators, pastTimes, pastVoteInfos, eventStats.Tally, header)

		// Update the validator set, which will be reflected in the application
		// on the next block
		validators = nextValidators
		nextValidators = updateValidators(tb, r, params, validators, res.ValidatorUpdates, eventStats.Tally)

		// update the exported params
		if config.ExportParamsPath != "" && config.ExportParamsHeight == height {
			exportedParams = params
		}
	}

	if stopEarly {
		if config.ExportStatsPath != "" {
			fmt.Println("Exporting simulation statistics...")
			eventStats.ExportJSON(config.ExportStatsPath)
		} else {
			eventStats.Print(w)
		}

		return true, exportedParams, err
	}

	fmt.Fprintf(
		w,
		"\nSimulation complete; Final height (blocks): %d, final time (seconds): %v, operations ran: %d\n",
		header.Height, header.Time, opCount,
	)

	if config.ExportStatsPath != "" {
		fmt.Println("Exporting simulation statistics...")
		eventStats.ExportJSON(config.ExportStatsPath)
	} else {
		eventStats.Print(w)
	}

	return false, exportedParams, nil
}

type blockSimFn func(r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context,
	accounts []simulation.Account, header tmproto.Header) (opCount int)

// Returns a function to simulate blocks. Written like this to avoid constant
// parameters being passed everytime, to minimize memory overhead.
func createBlockSimulator(testingMode bool, tb testing.TB, w io.Writer, params Params,
	event func(route, op, evResult string), ops WeightedOperations,
	operationQueue OperationQueue, timeOperationQueue []simulation.FutureOperation,
	logWriter LogWriter, config simulation.Config,
) blockSimFn {
	lastBlockSizeState := 0 // state for [4 * uniform distribution]
	blocksize := 0
	selectOp := ops.getSelectOpFn()

	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accounts []simulation.Account, header tmproto.Header,
	) (opCount int) {
		_, _ = fmt.Fprintf(
			w, "\rSimulating... block %d/%d, operation %d/%d.",
			header.Height, config.NumBlocks, opCount, blocksize,
		)
		lastBlockSizeState, blocksize = getBlockSize(r, params, lastBlockSizeState, config.BlockSize)

		type opAndR struct {
			op   simulation.Operation
			rand *rand.Rand
		}

		opAndRz := make([]opAndR, 0, blocksize)

		// Predetermine the blocksize slice so that we can do things like block
		// out certain operations without changing the ops that follow.
		for i := 0; i < blocksize; i++ {
			opAndRz = append(opAndRz, opAndR{
				op:   selectOp(r),
				rand: simulation.DeriveRand(r),
			})
		}

		for i := 0; i < blocksize; i++ {
			// NOTE: the Rand 'r' should not be used here.
			opAndR := opAndRz[i]
			op, r2 := opAndR.op, opAndR.rand
			opMsg, futureOps, err := op(r2, app, ctx, accounts, config.ChainID)
			opMsg.LogEvent(event)

			if !config.Lean || opMsg.OK {
				logWriter.AddEntry(MsgEntry(header.Height, int64(i), opMsg))
			}

			if err != nil {
				logWriter.PrintLogs()
				tb.Fatalf(`error on block  %d/%d, operation (%d/%d) from x/%s:
%v
Comment: %s`,
					header.Height, config.NumBlocks, opCount, blocksize, opMsg.Route, err, opMsg.Comment)
			}

			queueOperations(operationQueue, timeOperationQueue, futureOps)

			if testingMode && opCount%50 == 0 {
				fmt.Fprintf(w, "\rSimulating... block %d/%d, operation %d/%d. ",
					header.Height, config.NumBlocks, opCount, blocksize)
			}

			opCount++
		}

		return opCount
	}
}

func runQueuedOperations(queueOps map[int][]simulation.Operation,
	height int, tb testing.TB, r *rand.Rand, app *baseapp.BaseApp,
	ctx sdk.Context, accounts []simulation.Account, logWriter LogWriter,
	event func(route, op, evResult string), lean bool, chainID string,
) (numOpsRan int, allFutureOps []simulation.FutureOperation) {
	queuedOp, ok := queueOps[height]
	if !ok {
		return 0, nil
	}

	// Keep all future operations
	allFutureOps = make([]simulation.FutureOperation, 0)

	numOpsRan = len(queuedOp)
	for i := 0; i < numOpsRan; i++ {
		opMsg, futureOps, err := queuedOp[i](r, app, ctx, accounts, chainID)
		if len(futureOps) > 0 {
			allFutureOps = append(allFutureOps, futureOps...)
		}

		opMsg.LogEvent(event)

		if !lean || opMsg.OK {
			logWriter.AddEntry((QueuedMsgEntry(int64(height), opMsg)))
		}

		if err != nil {
			logWriter.PrintLogs()
			tb.FailNow()
		}
	}
	delete(queueOps, height)

	return numOpsRan, allFutureOps
}

func runQueuedTimeOperations(queueOps []simulation.FutureOperation,
	height int, currentTime time.Time, tb testing.TB, r *rand.Rand,
	app *baseapp.BaseApp, ctx sdk.Context, accounts []simulation.Account,
	logWriter LogWriter, event func(route, op, evResult string),
	lean bool, chainID string,
) (numOpsRan int, allFutureOps []simulation.FutureOperation) {
	// Keep all future operations
	allFutureOps = make([]simulation.FutureOperation, 0)

	numOpsRan = 0
	for len(queueOps) > 0 && currentTime.After(queueOps[0].BlockTime) {
		opMsg, futureOps, err := queueOps[0].Op(r, app, ctx, accounts, chainID)

		opMsg.LogEvent(event)

		if !lean || opMsg.OK {
			logWriter.AddEntry(QueuedMsgEntry(int64(height), opMsg))
		}

		if err != nil {
			logWriter.PrintLogs()
			tb.FailNow()
		}

		if len(futureOps) > 0 {
			allFutureOps = append(allFutureOps, futureOps...)
		}

		queueOps = queueOps[1:]
		numOpsRan++
	}

	return numOpsRan, allFutureOps
}
