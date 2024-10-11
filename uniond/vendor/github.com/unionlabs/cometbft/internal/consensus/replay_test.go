package consensus

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/abci/types/mocks"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cfg "github.com/cometbft/cometbft/config"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	smmocks "github.com/cometbft/cometbft/state/mocks"
	"github.com/cometbft/cometbft/types"
)

func TestMain(m *testing.M) {
	config = ResetConfig("consensus_reactor_test")
	consensusReplayConfig = ResetConfig("consensus_replay_test")
	configStateTest := ResetConfig("consensus_state_test")
	configMempoolTest := ResetConfig("consensus_mempool_test")
	configByzantineTest := ResetConfig("consensus_byzantine_test")
	code := m.Run()
	os.RemoveAll(config.RootDir)
	os.RemoveAll(consensusReplayConfig.RootDir)
	os.RemoveAll(configStateTest.RootDir)
	os.RemoveAll(configMempoolTest.RootDir)
	os.RemoveAll(configByzantineTest.RootDir)
	os.Exit(code)
}

// These tests ensure we can always recover from failure at any part of the consensus process.
// There are two general failure scenarios: failure during consensus, and failure while applying the block.
// Only the latter interacts with the app and store,
// but the former has to deal with restrictions on reuse of priv_validator keys.
// The `WAL Tests` are for failures during the consensus;
// the `Handshake Tests` are for failures in applying the block.
// With the help of the WAL, we can recover from it all!

// ------------------------------------------------------------------------------------------
// WAL Tests

// TODO: It would be better to verify explicitly which states we can recover from without the wal
// and which ones we need the wal for - then we'd also be able to only flush the
// wal writer when we need to, instead of with every message.

func startNewStateAndWaitForBlock(
	t *testing.T,
	consensusReplayConfig *cfg.Config,
	blockDB dbm.DB,
	stateStore sm.Store,
) {
	t.Helper()
	logger := log.TestingLogger()
	state, _ := stateStore.LoadFromDBOrGenesisFile(consensusReplayConfig.GenesisFile())
	privValidator, err := loadPrivValidator(consensusReplayConfig)
	require.NoError(t, err)
	cs := newStateWithConfigAndBlockStore(
		consensusReplayConfig,
		state,
		privValidator,
		kvstore.NewInMemoryApplication(),
		blockDB,
	)
	cs.SetLogger(logger)

	bytes, _ := os.ReadFile(cs.config.WalFile())
	t.Logf("====== WAL: \n\r%X\n", bytes)

	err = cs.Start()
	require.NoError(t, err)
	defer func() {
		if err := cs.Stop(); err != nil {
			t.Error(err)
		}
	}()

	// This is just a signal that we haven't halted; its not something contained
	// in the WAL itself. Assuming the consensus state is running, replay of any
	// WAL, including the empty one, should eventually be followed by a new
	// block, or else something is wrong.
	newBlockSub, err := cs.eventBus.Subscribe(context.Background(), testSubscriber, types.EventQueryNewBlock)
	require.NoError(t, err)
	select {
	case <-newBlockSub.Out():
	case <-newBlockSub.Canceled():
		t.Fatal("newBlockSub was canceled")
	case <-time.After(120 * time.Second):
		t.Fatal("Timed out waiting for new block (see trace above)")
	}
}

func sendTxs(ctx context.Context, cs *State) {
	for i := 0; i < 256; i++ {
		select {
		case <-ctx.Done():
			return
		default:
			tx := kvstore.NewTxFromID(i)
			reqRes, err := assertMempool(cs.txNotifier).CheckTx(tx, "")
			if err != nil {
				panic(err)
			}
			resp := reqRes.Response.GetCheckTx()
			if resp.Code != 0 {
				panic(fmt.Sprintf("Unexpected code: %d, log: %s", resp.Code, resp.Log))
			}
			i++
		}
	}
}

// TestWALCrash uses crashing WAL to test we can recover from any WAL failure.
func TestWALCrash(t *testing.T) {
	testCases := []struct {
		name         string
		initFn       func(dbm.DB, *State, context.Context)
		heightToStop int64
	}{
		{
			"empty block",
			func(_ dbm.DB, _ *State, _ context.Context) {},
			1,
		},
		{
			"many non-empty blocks",
			func(_ dbm.DB, cs *State, ctx context.Context) {
				go sendTxs(ctx, cs)
			},
			3,
		},
	}

	for i, tc := range testCases {
		consensusReplayConfig := ResetConfig(fmt.Sprintf("%s_%d", t.Name(), i))
		t.Run(tc.name, func(t *testing.T) {
			crashWALandCheckLiveness(t, consensusReplayConfig, tc.initFn, tc.heightToStop)
		})
	}
}

func crashWALandCheckLiveness(t *testing.T, consensusReplayConfig *cfg.Config,
	initFn func(dbm.DB, *State, context.Context), heightToStop int64,
) {
	t.Helper()
	walPanicked := make(chan error)
	crashingWal := &crashingWAL{panicCh: walPanicked, heightToStop: heightToStop}

	i := 1
LOOP:
	for {
		t.Logf("====== LOOP %d\n", i)

		// create consensus state from a clean slate
		logger := log.NewNopLogger()
		blockDB := dbm.NewMemDB()
		stateDB := blockDB
		stateStore := sm.NewStore(stateDB, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		state, err := sm.MakeGenesisStateFromFile(consensusReplayConfig.GenesisFile())
		require.NoError(t, err)
		privValidator, err := loadPrivValidator(consensusReplayConfig)
		require.NoError(t, err)
		cs := newStateWithConfigAndBlockStore(
			consensusReplayConfig,
			state,
			privValidator,
			kvstore.NewInMemoryApplication(),
			blockDB,
		)
		cs.SetLogger(logger)

		// start sending transactions
		ctx, cancel := context.WithCancel(context.Background())
		initFn(stateDB, cs, ctx)

		// clean up WAL file from the previous iteration
		walFile := cs.config.WalFile()
		os.Remove(walFile)

		// set crashing WAL
		csWal, err := cs.OpenWAL(walFile)
		require.NoError(t, err)
		crashingWal.next = csWal

		// reset the message counter
		crashingWal.msgIndex = 1
		cs.wal = crashingWal

		// start consensus state
		err = cs.Start()
		require.NoError(t, err)

		i++

		select {
		case err := <-walPanicked:
			t.Logf("WAL panicked: %v", err)

			// make sure we can make blocks after a crash
			startNewStateAndWaitForBlock(t, consensusReplayConfig, blockDB, stateStore)

			// stop consensus state and transactions sender (initFn)
			cs.Stop() //nolint:errcheck // Logging this error causes failure
			cancel()

			// if we reached the required height, exit
			if _, ok := err.(ReachedHeightToStopError); ok {
				break LOOP
			}
		case <-time.After(10 * time.Second):
			t.Fatal("WAL did not panic for 10 seconds (check the log)")
		}
	}
}

// crashingWAL is a WAL which crashes or rather simulates a crash during Save
// (before and after). It remembers a message for which we last panicked
// (lastPanickedForMsgIndex), so we don't panic for it in subsequent iterations.
type crashingWAL struct {
	next         WAL
	panicCh      chan error
	heightToStop int64

	msgIndex                int // current message index
	lastPanickedForMsgIndex int // last message for which we panicked
}

var _ WAL = &crashingWAL{}

// WALWriteError indicates a WAL crash.
type WALWriteError struct {
	msg string
}

func (e WALWriteError) Error() string {
	return e.msg
}

// ReachedHeightToStopError indicates we've reached the required consensus
// height and may exit.
type ReachedHeightToStopError struct {
	height int64
}

func (e ReachedHeightToStopError) Error() string {
	return fmt.Sprintf("reached height to stop %d", e.height)
}

// Write simulate WAL's crashing by sending an error to the panicCh and then
// exiting the cs.receiveRoutine.
func (w *crashingWAL) Write(m WALMessage) error {
	if endMsg, ok := m.(EndHeightMessage); ok {
		if endMsg.Height == w.heightToStop {
			w.panicCh <- ReachedHeightToStopError{endMsg.Height}
			runtime.Goexit()
			return nil
		}

		return w.next.Write(m)
	}

	if w.msgIndex > w.lastPanickedForMsgIndex {
		w.lastPanickedForMsgIndex = w.msgIndex
		_, file, line, _ := runtime.Caller(1)
		w.panicCh <- WALWriteError{fmt.Sprintf("failed to write %T to WAL (fileline: %s:%d)", m, file, line)}
		runtime.Goexit()
		return nil
	}

	w.msgIndex++
	return w.next.Write(m)
}

func (w *crashingWAL) WriteSync(m WALMessage) error {
	return w.Write(m)
}

func (w *crashingWAL) FlushAndSync() error { return w.next.FlushAndSync() }

func (w *crashingWAL) SearchForEndHeight(
	height int64,
	options *WALSearchOptions,
) (rd io.ReadCloser, found bool, err error) {
	return w.next.SearchForEndHeight(height, options)
}

func (w *crashingWAL) Start() error { return w.next.Start() }
func (w *crashingWAL) Stop() error  { return w.next.Stop() }
func (w *crashingWAL) Wait()        { w.next.Wait() }

// ------------------------------------------------------------------------------------------

const numBlocks = 6

// ---------------------------------------
// Test handshake/replay

// 0 - all synced up
// 1 - saved block but app and state are behind by one height
// 2 - save block and committed (i.e. app got `Commit`) but state is behind
// 3 - same as 2 but with a truncated block store.
var modes = []uint{0, 1, 2, 3}

// This is actually not a test, it's for storing validator change tx data for testHandshakeReplay.
func setupChainWithChangingValidators(t *testing.T, name string, nBlocks int) (*cfg.Config, []*types.Block, []*types.ExtendedCommit, sm.State) {
	t.Helper()

	nPeers := 7
	nVals := 4
	css, genDoc, config, cleanup := randConsensusNetWithPeers(
		t,
		nVals,
		nPeers,
		name,
		newMockTickerFunc(true),
		func(_ string) abci.Application {
			return newKVStore()
		})
	chainID := genDoc.ChainID
	genesisState, err := sm.MakeGenesisState(genDoc)
	require.NoError(t, err)
	t.Cleanup(cleanup)

	newRoundCh := subscribe(css[0].eventBus, types.EventQueryNewRound)
	proposalCh := subscribe(css[0].eventBus, types.EventQueryCompleteProposal)

	vss := make([]*validatorStub, nPeers)
	for i := 0; i < nPeers; i++ {
		vss[i] = newValidatorStub(css[i].privValidator, int32(i))
	}
	height, round := css[0].Height, css[0].Round

	// start the machine
	startTestRound(css[0], height, round)
	incrementHeight(vss...)
	ensureNewRound(newRoundCh, height, 0)
	ensureNewProposal(proposalCh, height, round)
	rs := css[0].GetRoundState()
	signAddVotes(css[0], types.PrecommitType, chainID, types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, vss[1:nVals]...)
	ensureNewRound(newRoundCh, height+1, 0)

	// HEIGHT 2
	height++
	incrementHeight(vss...)
	newValidatorPubKey1, err := css[nVals].privValidator.GetPubKey()
	require.NoError(t, err)
	newValidatorTx1 := updateValTx(newValidatorPubKey1, testMinPower)
	_, err = assertMempool(css[0].txNotifier).CheckTx(newValidatorTx1, "")
	require.NoError(t, err)

	propBlock, propBlockParts, blockID := createProposalBlock(t, css[0]) // changeProposer(t, cs1, v2)
	proposal := types.NewProposal(vss[1].Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vss[1])

	// set the proposal block
	if err := css[0].SetProposalAndBlock(proposal, propBlockParts, "some peer"); err != nil {
		t.Fatal(err)
	}
	ensureNewProposal(proposalCh, height, round)
	rs = css[0].GetRoundState()
	signAddVotes(css[0], types.PrecommitType, chainID, types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, vss[1:nVals]...)
	ensureNewRound(newRoundCh, height+1, 0)

	// HEIGHT 3
	height++
	incrementHeight(vss...)
	updateValidatorPubKey1, err := css[nVals].privValidator.GetPubKey()
	require.NoError(t, err)
	updateValidatorTx1 := updateValTx(updateValidatorPubKey1, 25)
	_, err = assertMempool(css[0].txNotifier).CheckTx(updateValidatorTx1, "")
	require.NoError(t, err)

	propBlock, propBlockParts, blockID = createProposalBlock(t, css[0]) // changeProposer(t, cs1, v2)
	proposal = types.NewProposal(vss[2].Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vss[2])

	// set the proposal block
	if err := css[0].SetProposalAndBlock(proposal, propBlockParts, "some peer"); err != nil {
		t.Fatal(err)
	}
	ensureNewProposal(proposalCh, height, round)
	rs = css[0].GetRoundState()
	signAddVotes(css[0], types.PrecommitType, chainID, types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, vss[1:nVals]...)
	ensureNewRound(newRoundCh, height+1, 0)

	// HEIGHT 4
	height++
	incrementHeight(vss...)
	newValidatorPubKey2, err := css[nVals+1].privValidator.GetPubKey()
	require.NoError(t, err)
	newValidatorTx2 := updateValTx(newValidatorPubKey2, testMinPower)
	_, err = assertMempool(css[0].txNotifier).CheckTx(newValidatorTx2, "")
	require.NoError(t, err)
	newValidatorPubKey3, err := css[nVals+2].privValidator.GetPubKey()
	require.NoError(t, err)
	newValidatorTx3 := updateValTx(newValidatorPubKey3, testMinPower)
	_, err = assertMempool(css[0].txNotifier).CheckTx(newValidatorTx3, "")
	require.NoError(t, err)

	propBlock, propBlockParts, blockID = createProposalBlock(t, css[0]) // changeProposer(t, cs1, v2)

	newVss := make([]*validatorStub, nVals+1)
	copy(newVss, vss[:nVals+1])
	sort.Sort(ValidatorStubsByPower(newVss))

	valIndexFn := func(cssIdx int) int {
		for i, vs := range newVss {
			vsPubKey, err := vs.GetPubKey()
			require.NoError(t, err)

			cssPubKey, err := css[cssIdx].privValidator.GetPubKey()
			require.NoError(t, err)

			if vsPubKey.Type() == cssPubKey.Type() && bytes.Equal(vsPubKey.Bytes(), cssPubKey.Bytes()) {
				return i
			}
		}
		panic(fmt.Sprintf("validator css[%d] not found in newVss", cssIdx))
	}

	selfIndex := valIndexFn(0)

	proposal = types.NewProposal(vss[3].Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vss[3])

	// set the proposal block
	if err := css[0].SetProposalAndBlock(proposal, propBlockParts, "some peer"); err != nil {
		t.Fatal(err)
	}
	ensureNewProposal(proposalCh, height, round)

	removeValidatorTx2 := updateValTx(newValidatorPubKey2, 0)
	_, err = assertMempool(css[0].txNotifier).CheckTx(removeValidatorTx2, "")
	require.NoError(t, err)

	rs = css[0].GetRoundState()
	for i := 0; i < nVals+1; i++ {
		if i == selfIndex {
			continue
		}
		signAddVotes(css[0], types.PrecommitType, chainID,
			types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, newVss[i])
	}

	ensureNewRound(newRoundCh, height+1, 0)

	// HEIGHT 5
	height++
	incrementHeight(vss...)
	// Reflect the changes to vss[nVals] at height 3 and resort newVss.
	newVssIdx := valIndexFn(nVals)
	newVss[newVssIdx].VotingPower = 25
	sort.Sort(ValidatorStubsByPower(newVss))
	selfIndex = valIndexFn(0)
	ensureNewProposal(proposalCh, height, round)
	rs = css[0].GetRoundState()
	for i := 0; i < nVals+1; i++ {
		if i == selfIndex {
			continue
		}
		signAddVotes(css[0], types.PrecommitType, chainID, types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, newVss[i])
	}
	ensureNewRound(newRoundCh, height+1, 0)

	// HEIGHT 6
	height++
	incrementHeight(vss...)
	removeValidatorTx3 := updateValTx(newValidatorPubKey3, 0)
	_, err = assertMempool(css[0].txNotifier).CheckTx(removeValidatorTx3, "")
	require.NoError(t, err)

	propBlock, propBlockParts, blockID = createProposalBlock(t, css[0]) // changeProposer(t, cs1, v2)

	newVss = make([]*validatorStub, nVals+3)
	copy(newVss, vss[:nVals+3])
	sort.Sort(ValidatorStubsByPower(newVss))

	selfIndex = valIndexFn(0)
	proposal = types.NewProposal(vss[1].Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vss[1])

	// set the proposal block
	if err := css[0].SetProposalAndBlock(proposal, propBlockParts, "some peer"); err != nil {
		t.Fatal(err)
	}
	ensureNewProposal(proposalCh, height, round)
	rs = css[0].GetRoundState()
	for i := 0; i < nVals+3; i++ {
		if i == selfIndex {
			continue
		}
		signAddVotes(css[0], types.PrecommitType, chainID, types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}, true, newVss[i])
	}
	ensureNewRound(newRoundCh, height+1, 0)

	chain := []*types.Block{}
	extCommits := []*types.ExtendedCommit{}
	for i := 1; i <= nBlocks; i++ {
		block, _ := css[0].blockStore.LoadBlock(int64(i))
		chain = append(chain, block)
		extCommits = append(extCommits, css[0].blockStore.LoadBlockExtendedCommit(int64(i)))
	}
	return config, chain, extCommits, genesisState
}

// Sync from scratch.
func TestHandshakeReplayAll(t *testing.T) {
	for _, m := range modes {
		t.Run(fmt.Sprintf("mode_%d_single", m), func(t *testing.T) {
			testHandshakeReplay(t, config, 0, m, false)
		})
		t.Run(fmt.Sprintf("mode_%d_multi", m), func(t *testing.T) {
			testHandshakeReplay(t, config, 0, m, false)
		})
	}
}

// Sync many, not from scratch.
func TestHandshakeReplaySome(t *testing.T) {
	for _, m := range modes {
		t.Run(fmt.Sprintf("mode_%d_single", m), func(t *testing.T) {
			testHandshakeReplay(t, config, 2, m, false)
		})
		t.Run(fmt.Sprintf("mode_%d_multi", m), func(t *testing.T) {
			testHandshakeReplay(t, config, 2, m, true)
		})
	}
}

// Sync from lagging by one.
func TestHandshakeReplayOne(t *testing.T) {
	for _, m := range modes {
		t.Run(fmt.Sprintf("mode_%d_single", m), func(t *testing.T) {
			testHandshakeReplay(t, config, numBlocks-1, m, false)
		})
		t.Run(fmt.Sprintf("mode_%d_multi", m), func(t *testing.T) {
			testHandshakeReplay(t, config, numBlocks-1, m, true)
		})
	}
}

// Sync from caught up.
func TestHandshakeReplayNone(t *testing.T) {
	for _, m := range modes {
		t.Run(fmt.Sprintf("mode_%d_single", m), func(t *testing.T) {
			testHandshakeReplay(t, config, numBlocks, m, false)
		})
		t.Run(fmt.Sprintf("mode_%d_multi", m), func(t *testing.T) {
			testHandshakeReplay(t, config, numBlocks, m, true)
		})
	}
}

func tempWALWithData(data []byte) string {
	walFile, err := os.CreateTemp("", "wal")
	if err != nil {
		panic(fmt.Sprintf("failed to create temp WAL file: %v", err))
	}
	_, err = walFile.Write(data)
	if err != nil {
		panic(fmt.Sprintf("failed to write to temp WAL file: %v", err))
	}
	if err := walFile.Close(); err != nil {
		panic(fmt.Sprintf("failed to close temp WAL file: %v", err))
	}
	return walFile.Name()
}

// Make some blocks. Start a fresh app and apply nBlocks blocks.
// Then restart the app and sync it up with the remaining blocks.
func testHandshakeReplay(t *testing.T, config *cfg.Config, nBlocks int, mode uint, testValidatorsChange bool) {
	t.Helper()
	var (
		testConfig   *cfg.Config
		chain        []*types.Block
		extCommits   []*types.ExtendedCommit
		store        *mockBlockStore
		stateDB      dbm.DB
		genesisState sm.State
		mempool      = emptyMempool{}
		evpool       = sm.EmptyEvidencePool{}
	)

	if testValidatorsChange {
		testConfig, chain, extCommits, genesisState = setupChainWithChangingValidators(t, fmt.Sprintf("%d_%d_m", nBlocks, mode), numBlocks)
		stateDB = dbm.NewMemDB()
		store = newMockBlockStore(t, config, genesisState.ConsensusParams)
	} else {
		testConfig = ResetConfig(fmt.Sprintf("%d_%d_s", nBlocks, mode))
		t.Cleanup(func() {
			_ = os.RemoveAll(testConfig.RootDir)
		})
		walBody, err := WALWithNBlocks(t, numBlocks, testConfig)
		require.NoError(t, err)
		walFile := tempWALWithData(walBody)
		testConfig.Consensus.SetWalFile(walFile)

		wal, err := NewWAL(walFile)
		require.NoError(t, err)
		wal.SetLogger(log.TestingLogger())
		err = wal.Start()
		require.NoError(t, err)
		t.Cleanup(func() {
			if err := wal.Stop(); err != nil {
				t.Error(err)
			}
		})
		chain, extCommits, err = makeBlockchainFromWAL(wal)
		require.NoError(t, err)
		stateDB, genesisState, store = stateAndStore(t, testConfig, kvstore.AppVersion)
	}

	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	t.Cleanup(func() {
		_ = stateStore.Close()
	})
	store.chain = chain
	store.extCommits = extCommits

	state := genesisState.Copy()
	// run the chain through state.ApplyBlock to build up the CometBFT state
	state, latestAppHash := buildTMStateFromChain(t, testConfig, stateStore, mempool, evpool, state, chain, nBlocks, mode, store)

	// make a new client creator
	kvstoreApp := kvstore.NewPersistentApplication(
		filepath.Join(testConfig.DBDir(), fmt.Sprintf("replay_test_%d_%d_a", nBlocks, mode)))
	t.Cleanup(func() {
		_ = kvstoreApp.Close()
	})

	clientCreator2 := proxy.NewLocalClientCreator(kvstoreApp)
	if nBlocks > 0 {
		// run nBlocks against a new client to build up the app state.
		// use a throwaway CometBFT state
		proxyApp := proxy.NewAppConns(clientCreator2, proxy.NopMetrics())
		stateDB1 := dbm.NewMemDB()
		dummyStateStore := sm.NewStore(stateDB1, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		err := dummyStateStore.Save(genesisState)
		require.NoError(t, err)
		buildAppStateFromChain(t, proxyApp, dummyStateStore, mempool, evpool, genesisState, chain, nBlocks, mode, store)
	}

	// Prune block store if requested
	expectError := false
	if mode == 3 {
		pruned, _, err := store.PruneBlocks(2, state)
		require.NoError(t, err)
		require.EqualValues(t, 1, pruned)
		expectError = int64(nBlocks) < 2
	}

	// now start the app using the handshake - it should sync
	genDoc, err := sm.MakeGenesisDocFromFile(testConfig.GenesisFile())
	require.NoError(t, err)
	handshaker := NewHandshaker(stateStore, state, store, genDoc)
	proxyApp := proxy.NewAppConns(clientCreator2, proxy.NopMetrics())
	if err := proxyApp.Start(); err != nil {
		t.Fatalf("Error starting proxy app connections: %v", err)
	}

	t.Cleanup(func() {
		if err := proxyApp.Stop(); err != nil {
			t.Error(err)
		}
	})

	// perform the replay protocol to sync Tendermint and the application
	err = handshaker.Handshake(context.Background(), proxyApp)
	if expectError {
		require.Error(t, err)
		// finish the test early
		return
	}
	require.NoError(t, err)

	// get the latest app hash from the app
	res, err := proxyApp.Query().Info(context.Background(), proxy.InfoRequest)
	require.NoError(t, err)

	// block store and app height should be in sync
	require.Equal(t, store.Height(), res.LastBlockHeight)

	// tendermint state height and app height should be in sync
	state, err = stateStore.Load()
	require.NoError(t, err)
	require.Equal(t, state.LastBlockHeight, res.LastBlockHeight)
	require.Equal(t, int64(numBlocks), res.LastBlockHeight)

	// the app hash should be synced up
	if !bytes.Equal(latestAppHash, res.LastBlockAppHash) {
		t.Fatalf(
			"Expected app hashes to match after handshake/replay. got %X, expected %X",
			res.LastBlockAppHash,
			latestAppHash)
	}

	expectedBlocksToSync := numBlocks - nBlocks
	if nBlocks == numBlocks && mode > 0 {
		expectedBlocksToSync++
	} else if nBlocks > 0 && mode == 1 {
		expectedBlocksToSync++
	}

	if handshaker.NBlocks() != expectedBlocksToSync {
		t.Fatalf("Expected handshake to sync %d blocks, got %d", expectedBlocksToSync, handshaker.NBlocks())
	}
}

func applyBlock(t *testing.T, stateStore sm.Store, mempool mempl.Mempool, evpool sm.EvidencePool, st sm.State, blk *types.Block, proxyApp proxy.AppConns, bs sm.BlockStore) sm.State {
	t.Helper()
	testPartSize := types.BlockPartSizeBytes
	blockExec := sm.NewBlockExecutor(stateStore, log.TestingLogger(), proxyApp.Consensus(), mempool, evpool, bs)

	bps, err := blk.MakePartSet(testPartSize)
	require.NoError(t, err)
	blkID := types.BlockID{Hash: blk.Hash(), PartSetHeader: bps.Header()}
	newState, err := blockExec.ApplyBlock(st, blkID, blk, blk.Height)
	require.NoError(t, err)
	return newState
}

func buildAppStateFromChain(t *testing.T, proxyApp proxy.AppConns, stateStore sm.Store, mempool mempl.Mempool, evpool sm.EvidencePool,
	state sm.State, chain []*types.Block, nBlocks int, mode uint, bs sm.BlockStore,
) {
	t.Helper()
	// start a new app without handshake, play nBlocks blocks
	if err := proxyApp.Start(); err != nil {
		panic(err)
	}
	defer proxyApp.Stop() //nolint:errcheck // ignore

	state.Version.Consensus.App = kvstore.AppVersion // simulate handshake, receive app version
	validators := types.TM2PB.ValidatorUpdates(state.Validators)
	if _, err := proxyApp.Consensus().InitChain(context.Background(), &abci.InitChainRequest{
		Validators: validators,
	}); err != nil {
		panic(err)
	}
	if err := stateStore.Save(state); err != nil { // save height 1's validatorsInfo
		panic(err)
	}
	switch mode {
	case 0:
		for i := 0; i < nBlocks; i++ {
			block := chain[i]
			state = applyBlock(t, stateStore, mempool, evpool, state, block, proxyApp, bs)
		}
	case 1, 2, 3:
		for i := 0; i < nBlocks-1; i++ {
			block := chain[i]
			state = applyBlock(t, stateStore, mempool, evpool, state, block, proxyApp, bs)
		}

		// mode 1 only the block at the last height is saved
		// mode 2 and 3, the block is saved, commit is called, but the state is not saved
		if mode == 2 || mode == 3 {
			// update the kvstore height and apphash
			// as if we ran commit but not
			// here we expect a dummy state store to be used
			_ = applyBlock(t, stateStore, mempool, evpool, state, chain[nBlocks-1], proxyApp, bs)
		}
	default:
		panic(fmt.Sprintf("unknown mode %v", mode))
	}
}

func buildTMStateFromChain(
	t *testing.T,
	config *cfg.Config,
	stateStore sm.Store,
	mempool mempl.Mempool,
	evpool sm.EvidencePool,
	state sm.State,
	chain []*types.Block,
	nBlocks int,
	mode uint,
	bs sm.BlockStore,
) (sm.State, []byte) {
	t.Helper()
	// run the whole chain against this client to build up the CometBFT state
	clientCreator := proxy.NewLocalClientCreator(
		kvstore.NewPersistentApplication(
			filepath.Join(config.DBDir(), fmt.Sprintf("replay_test_%d_%d_t", nBlocks, mode))))
	proxyApp := proxy.NewAppConns(clientCreator, proxy.NopMetrics())
	if err := proxyApp.Start(); err != nil {
		panic(err)
	}
	defer proxyApp.Stop() //nolint:errcheck

	state.Version.Consensus.App = kvstore.AppVersion // simulate handshake, receive app version
	validators := types.TM2PB.ValidatorUpdates(state.Validators)
	if _, err := proxyApp.Consensus().InitChain(context.Background(), &abci.InitChainRequest{
		Validators: validators,
	}); err != nil {
		panic(err)
	}
	if err := stateStore.Save(state); err != nil { // save height 1's validatorsInfo
		panic(err)
	}
	switch mode {
	case 0:
		// sync right up
		for _, block := range chain {
			state = applyBlock(t, stateStore, mempool, evpool, state, block, proxyApp, bs)
		}
		return state, state.AppHash

	case 1, 2, 3:
		// sync up to the penultimate as if we stored the block.
		// whether we commit or not depends on the appHash
		for _, block := range chain[:len(chain)-1] {
			state = applyBlock(t, stateStore, mempool, evpool, state, block, proxyApp, bs)
		}

		dummyStateStore := &smmocks.Store{}
		lastHeight := int64(len(chain))
		penultimateHeight := int64(len(chain) - 1)
		vals, _ := stateStore.LoadValidators(penultimateHeight)
		dummyStateStore.On("LoadValidators", penultimateHeight).Return(vals, nil)
		dummyStateStore.On("Save", mock.Anything).Return(nil)
		dummyStateStore.On("SaveFinalizeBlockResponse", lastHeight, mock.MatchedBy(func(response *abci.FinalizeBlockResponse) bool {
			require.NoError(t, stateStore.SaveFinalizeBlockResponse(lastHeight, response))
			return true
		})).Return(nil)
		dummyStateStore.On("GetApplicationRetainHeight", mock.Anything).Return(int64(0), nil)
		dummyStateStore.On("GetCompanionBlockRetainHeight", mock.Anything).Return(int64(0), nil)
		dummyStateStore.On("GetABCIResRetainHeight", mock.Anything).Return(int64(0), nil)

		// apply the final block to a state copy so we can
		// get the right next appHash but keep the state back
		s := applyBlock(t, dummyStateStore, mempool, evpool, state, chain[len(chain)-1], proxyApp, bs)
		return state, s.AppHash
	default:
		panic(fmt.Sprintf("unknown mode %v", mode))
	}
}

func makeBlocks(n int, state sm.State, privVals []types.PrivValidator) ([]*types.Block, error) {
	blockID := test.MakeBlockID()
	blocks := make([]*types.Block, n)

	for i := 0; i < n; i++ {
		height := state.LastBlockHeight + 1 + int64(i)
		lastCommit, err := test.MakeCommit(blockID, height-1, 0, state.LastValidators, privVals, state.ChainID, state.LastBlockTime)
		if err != nil {
			return nil, err
		}
		block := state.MakeBlock(height, test.MakeNTxs(height, 10), lastCommit, nil, state.LastValidators.Proposer.Address)
		blocks[i] = block
		state.LastBlockID = blockID
		state.LastBlockHeight = height
		state.LastBlockTime = state.LastBlockTime.Add(1 * time.Second)
		state.LastValidators = state.Validators.Copy()
		state.Validators = state.NextValidators.Copy()
		state.NextValidators = state.NextValidators.CopyIncrementProposerPriority(1)
		state.AppHash = test.RandomHash()

		blockID = test.MakeBlockIDWithHash(block.Hash())
	}

	return blocks, nil
}

func TestHandshakePanicsIfAppReturnsWrongAppHash(t *testing.T) {
	// 1. Initialize CometBFT and commit 3 blocks with the following app hashes:
	//		- 0x01
	//		- 0x02
	//		- 0x03
	config := ResetConfig("handshake_test_")
	defer os.RemoveAll(config.RootDir)
	privVal := privval.LoadFilePV(config.PrivValidatorKeyFile(), config.PrivValidatorStateFile())
	const appVersion = 0x0
	stateDB, state, store := stateAndStore(t, config, appVersion)
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	genDoc, err := sm.MakeGenesisDocFromFile(config.GenesisFile())
	require.NoError(t, err)
	state.LastValidators = state.Validators.Copy()
	// mode = 0 for committing all the blocks
	blocks, err := makeBlocks(3, state, []types.PrivValidator{privVal})
	require.NoError(t, err)

	store.chain = blocks

	// 2. CometBFT must panic if app returns wrong hash for the first block
	//		- RANDOM HASH
	//		- 0x02
	//		- 0x03
	{
		app := &badApp{numBlocks: 3, allHashesAreWrong: true}
		clientCreator := proxy.NewLocalClientCreator(app)
		proxyApp := proxy.NewAppConns(clientCreator, proxy.NopMetrics())
		err := proxyApp.Start()
		require.NoError(t, err)
		t.Cleanup(func() {
			if err := proxyApp.Stop(); err != nil {
				t.Error(err)
			}
		})

		assert.Panics(t, func() {
			h := NewHandshaker(stateStore, state, store, genDoc)
			if err = h.Handshake(context.Background(), proxyApp); err != nil {
				t.Log(err)
			}
		})
	}

	// 3. CometBFT must panic if app returns wrong hash for the last block
	//		- 0x01
	//		- 0x02
	//		- RANDOM HASH
	{
		app := &badApp{numBlocks: 3, onlyLastHashIsWrong: true}
		clientCreator := proxy.NewLocalClientCreator(app)
		proxyApp := proxy.NewAppConns(clientCreator, proxy.NopMetrics())
		err := proxyApp.Start()
		require.NoError(t, err)
		t.Cleanup(func() {
			if err := proxyApp.Stop(); err != nil {
				t.Error(err)
			}
		})

		assert.Panics(t, func() {
			h := NewHandshaker(stateStore, state, store, genDoc)
			if err = h.Handshake(context.Background(), proxyApp); err != nil {
				t.Log(err)
			}
		})
	}
}

type badApp struct {
	abci.BaseApplication
	numBlocks           byte
	height              byte
	allHashesAreWrong   bool
	onlyLastHashIsWrong bool
}

func (app *badApp) FinalizeBlock(context.Context, *abci.FinalizeBlockRequest) (*abci.FinalizeBlockResponse, error) {
	app.height++
	if app.onlyLastHashIsWrong {
		if app.height == app.numBlocks {
			return &abci.FinalizeBlockResponse{AppHash: cmtrand.Bytes(8)}, nil
		}
		return &abci.FinalizeBlockResponse{AppHash: []byte{app.height}}, nil
	} else if app.allHashesAreWrong {
		return &abci.FinalizeBlockResponse{AppHash: cmtrand.Bytes(8)}, nil
	}

	panic("either allHashesAreWrong or onlyLastHashIsWrong must be set")
}

// --------------------------
// utils for making blocks

func makeBlockchainFromWAL(wal WAL) ([]*types.Block, []*types.ExtendedCommit, error) {
	var height int64

	// Search for height marker
	gr, found, err := wal.SearchForEndHeight(height, &WALSearchOptions{})
	if err != nil {
		return nil, nil, err
	}
	if !found {
		return nil, nil, fmt.Errorf("wal does not contain height %d", height)
	}
	defer gr.Close()

	// log.Notice("Build a blockchain by reading from the WAL")

	var (
		blocks             []*types.Block
		extCommits         []*types.ExtendedCommit
		thisBlockParts     *types.PartSet
		thisBlockExtCommit *types.ExtendedCommit
	)

	dec := NewWALDecoder(gr)
	for {
		msg, err := dec.Decode()
		if errors.Is(err, io.EOF) {
			break
		} else if err != nil {
			return nil, nil, err
		}

		piece := readPieceFromWAL(msg)
		if piece == nil {
			continue
		}

		switch p := piece.(type) {
		case EndHeightMessage:
			// if its not the first one, we have a full block
			if thisBlockParts != nil {
				pbb := new(cmtproto.Block)
				bz, err := io.ReadAll(thisBlockParts.GetReader())
				if err != nil {
					panic(err)
				}
				err = proto.Unmarshal(bz, pbb)
				if err != nil {
					panic(err)
				}
				block, err := types.BlockFromProto(pbb)
				if err != nil {
					panic(err)
				}

				if block.Height != height+1 {
					panic(fmt.Sprintf("read bad block from wal. got height %d, expected %d", block.Height, height+1))
				}
				commitHeight := thisBlockExtCommit.Height
				if commitHeight != height+1 {
					panic(fmt.Sprintf("commit doesn't match. got height %d, expected %d", commitHeight, height+1))
				}
				blocks = append(blocks, block)
				extCommits = append(extCommits, thisBlockExtCommit)
				height++
			}
		case *types.PartSetHeader:
			thisBlockParts = types.NewPartSetFromHeader(*p)
		case *types.Part:
			_, err := thisBlockParts.AddPart(p)
			if err != nil {
				return nil, nil, err
			}
		case *types.Vote:
			if p.Type == types.PrecommitType {
				thisBlockExtCommit = &types.ExtendedCommit{
					Height:             p.Height,
					Round:              p.Round,
					BlockID:            p.BlockID,
					ExtendedSignatures: []types.ExtendedCommitSig{p.ExtendedCommitSig()},
				}
			}
		}
	}
	// grab the last block too
	bz, err := io.ReadAll(thisBlockParts.GetReader())
	if err != nil {
		panic(err)
	}
	pbb := new(cmtproto.Block)
	err = proto.Unmarshal(bz, pbb)
	if err != nil {
		panic(err)
	}
	block, err := types.BlockFromProto(pbb)
	if err != nil {
		panic(err)
	}
	if block.Height != height+1 {
		panic(fmt.Sprintf("read bad block from wal. got height %d, expected %d", block.Height, height+1))
	}
	commitHeight := thisBlockExtCommit.Height
	if commitHeight != height+1 {
		panic(fmt.Sprintf("commit doesn't match. got height %d, expected %d", commitHeight, height+1))
	}
	blocks = append(blocks, block)
	extCommits = append(extCommits, thisBlockExtCommit)
	return blocks, extCommits, nil
}

func readPieceFromWAL(msg *TimedWALMessage) any {
	// for logging
	switch m := msg.Msg.(type) {
	case msgInfo:
		switch msg := m.Msg.(type) {
		case *ProposalMessage:
			return &msg.Proposal.BlockID.PartSetHeader
		case *BlockPartMessage:
			return msg.Part
		case *VoteMessage:
			return msg.Vote
		}
	case EndHeightMessage:
		return m
	}

	return nil
}

// fresh state and mock store.
func stateAndStore(
	t *testing.T,
	config *cfg.Config,
	appVersion uint64,
) (dbm.DB, sm.State, *mockBlockStore) {
	t.Helper()
	stateDB := dbm.NewMemDB()
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state, err := sm.MakeGenesisStateFromFile(config.GenesisFile())
	require.NoError(t, err)
	state.Version.Consensus.App = appVersion
	store := newMockBlockStore(t, config, state.ConsensusParams)
	require.NoError(t, stateStore.Save(state))

	return stateDB, state, store
}

// ----------------------------------
// mock block store

type mockBlockStore struct {
	config     *cfg.Config
	params     types.ConsensusParams
	chain      []*types.Block
	extCommits []*types.ExtendedCommit
	base       int64
	t          *testing.T
}

var _ sm.BlockStore = &mockBlockStore{}

// TODO: NewBlockStore(db.NewMemDB) ...
func newMockBlockStore(t *testing.T, config *cfg.Config, params types.ConsensusParams) *mockBlockStore {
	t.Helper()
	return &mockBlockStore{
		config: config,
		params: params,
		t:      t,
	}
}

func (bs *mockBlockStore) Height() int64                  { return int64(len(bs.chain)) }
func (bs *mockBlockStore) Base() int64                    { return bs.base }
func (bs *mockBlockStore) Size() int64                    { return bs.Height() - bs.Base() + 1 }
func (bs *mockBlockStore) LoadBaseMeta() *types.BlockMeta { return bs.LoadBlockMeta(bs.base) }
func (bs *mockBlockStore) LoadBlock(height int64) (*types.Block, *types.BlockMeta) {
	return bs.chain[height-1], bs.LoadBlockMeta(height)
}

func (bs *mockBlockStore) LoadBlockByHash([]byte) (*types.Block, *types.BlockMeta) {
	height := int64(len(bs.chain))
	return bs.chain[height-1], bs.LoadBlockMeta(height)
}
func (*mockBlockStore) LoadBlockMetaByHash([]byte) *types.BlockMeta { return nil }
func (bs *mockBlockStore) LoadBlockMeta(height int64) *types.BlockMeta {
	block := bs.chain[height-1]
	bps, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(bs.t, err)
	return &types.BlockMeta{
		BlockID: types.BlockID{Hash: block.Hash(), PartSetHeader: bps.Header()},
		Header:  block.Header,
	}
}
func (*mockBlockStore) LoadBlockPart(int64, int) *types.Part { return nil }
func (*mockBlockStore) SaveBlockWithExtendedCommit(*types.Block, *types.PartSet, *types.ExtendedCommit) {
}

func (*mockBlockStore) SaveBlock(*types.Block, *types.PartSet, *types.Commit) {
}

func (bs *mockBlockStore) LoadBlockCommit(height int64) *types.Commit {
	return bs.extCommits[height-1].ToCommit()
}

func (bs *mockBlockStore) LoadSeenCommit(height int64) *types.Commit {
	return bs.extCommits[height-1].ToCommit()
}

func (bs *mockBlockStore) LoadBlockExtendedCommit(height int64) *types.ExtendedCommit {
	return bs.extCommits[height-1]
}

func (bs *mockBlockStore) PruneBlocks(height int64, _ sm.State) (uint64, int64, error) {
	evidencePoint := height
	pruned := uint64(0)
	for i := int64(0); i < height-1; i++ {
		bs.chain[i] = nil
		bs.extCommits[i] = nil
		pruned++
	}
	bs.base = height
	return pruned, evidencePoint, nil
}

func (*mockBlockStore) DeleteLatestBlock() error { return nil }
func (*mockBlockStore) Close() error             { return nil }

// ---------------------------------------
// Test handshake/init chain

func TestHandshakeUpdatesValidators(t *testing.T) {
	val, _ := types.RandValidator(true, 10)
	vals := types.NewValidatorSet([]*types.Validator{val})
	app := &mocks.Application{}
	app.On("Info", mock.Anything, mock.Anything).Return(&abci.InfoResponse{
		LastBlockHeight: 0,
	}, nil)
	app.On("InitChain", mock.Anything, mock.Anything).Return(&abci.InitChainResponse{
		Validators: types.TM2PB.ValidatorUpdates(vals),
	}, nil)
	clientCreator := proxy.NewLocalClientCreator(app)

	config := ResetConfig("handshake_test_")
	defer os.RemoveAll(config.RootDir)
	stateDB, state, store := stateAndStore(t, config, 0x0)
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})

	oldValAddr := state.Validators.Validators[0].Address

	// now start the app using the handshake - it should sync
	genDoc, _ := sm.MakeGenesisDocFromFile(config.GenesisFile())
	handshaker := NewHandshaker(stateStore, state, store, genDoc)
	proxyApp := proxy.NewAppConns(clientCreator, proxy.NopMetrics())
	if err := proxyApp.Start(); err != nil {
		t.Fatalf("Error starting proxy app connections: %v", err)
	}
	t.Cleanup(func() {
		if err := proxyApp.Stop(); err != nil {
			t.Error(err)
		}
	})
	if err := handshaker.Handshake(context.Background(), proxyApp); err != nil {
		t.Fatalf("Error on abci handshake: %v", err)
	}
	var err error
	// reload the state, check the validator set was updated
	state, err = stateStore.Load()
	require.NoError(t, err)

	newValAddr := state.Validators.Validators[0].Address
	expectValAddr := val.Address
	assert.NotEqual(t, oldValAddr, newValAddr)
	assert.Equal(t, newValAddr, expectValAddr)
}
