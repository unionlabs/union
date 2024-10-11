package monitor

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func setupService(t *testing.T) *Service {
	beaconDB := testDB.SetupDB(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)

	pubKeys := make([][]byte, 3)
	pubKeys[0] = state.Validators()[0].PublicKey
	pubKeys[1] = state.Validators()[1].PublicKey
	pubKeys[2] = state.Validators()[2].PublicKey

	currentSyncCommittee := util.ConvertToCommittee([][]byte{
		pubKeys[0], pubKeys[1], pubKeys[2], pubKeys[1], pubKeys[1],
	})
	require.NoError(t, state.SetCurrentSyncCommittee(currentSyncCommittee))

	chainService := &mock.ChainService{
		Genesis:        time.Now(),
		DB:             beaconDB,
		State:          state,
		Root:           []byte("hello-world"),
		ValidatorsRoot: [32]byte{},
	}

	trackedVals := map[primitives.ValidatorIndex]bool{
		1:  true,
		2:  true,
		12: true,
		15: true,
	}
	latestPerformance := map[primitives.ValidatorIndex]ValidatorLatestPerformance{
		1: {
			balance: 32000000000,
		},
		2: {
			balance: 32000000000,
		},
		12: {
			balance: 31900000000,
		},
		15: {
			balance: 31900000000,
		},
	}
	aggregatedPerformance := map[primitives.ValidatorIndex]ValidatorAggregatedPerformance{
		1: {
			startEpoch:                      0,
			startBalance:                    31700000000,
			totalAttestedCount:              12,
			totalRequestedCount:             15,
			totalDistance:                   14,
			totalCorrectHead:                8,
			totalCorrectSource:              11,
			totalCorrectTarget:              12,
			totalProposedCount:              1,
			totalSyncCommitteeContributions: 0,
			totalSyncCommitteeAggregations:  0,
		},
		2:  {},
		12: {},
		15: {},
	}
	trackedSyncCommitteeIndices := map[primitives.ValidatorIndex][]primitives.CommitteeIndex{
		1:  {0, 1, 2, 3},
		12: {4, 5},
	}
	return &Service{
		config: &ValidatorMonitorConfig{
			StateGen:            stategen.New(beaconDB, doublylinkedtree.New()),
			StateNotifier:       chainService.StateNotifier(),
			HeadFetcher:         chainService,
			AttestationNotifier: chainService.OperationNotifier(),
			InitialSyncComplete: make(chan struct{}),
		},

		ctx:                         context.Background(),
		TrackedValidators:           trackedVals,
		latestPerformance:           latestPerformance,
		aggregatedPerformance:       aggregatedPerformance,
		trackedSyncCommitteeIndices: trackedSyncCommitteeIndices,
		lastSyncedEpoch:             0,
	}
}

func TestTrackedIndex(t *testing.T) {
	s := &Service{
		TrackedValidators: map[primitives.ValidatorIndex]bool{
			1: true,
			2: true,
		},
	}
	require.Equal(t, s.trackedIndex(primitives.ValidatorIndex(1)), true)
	require.Equal(t, s.trackedIndex(primitives.ValidatorIndex(3)), false)
}

func TestUpdateSyncCommitteeTrackedVals(t *testing.T) {
	hook := logTest.NewGlobal()
	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 1024)

	s.updateSyncCommitteeTrackedVals(state)
	require.LogsDoNotContain(t, hook, "Sync committee assignments will not be reported")
	newTrackedSyncIndices := map[primitives.ValidatorIndex][]primitives.CommitteeIndex{
		1: {1, 3, 4},
		2: {2},
	}
	require.DeepEqual(t, s.trackedSyncCommitteeIndices, newTrackedSyncIndices)
}

func TestNewService(t *testing.T) {
	config := &ValidatorMonitorConfig{}
	var tracked []primitives.ValidatorIndex
	ctx := context.Background()
	_, err := NewService(ctx, config, tracked)
	require.NoError(t, err)
}

func TestStart(t *testing.T) {
	hook := logTest.NewGlobal()
	s := setupService(t)

	s.Start()
	close(s.config.InitialSyncComplete)

	// wait for Logrus
	time.Sleep(1000 * time.Millisecond)
	require.LogsContain(t, hook, "Synced to head epoch, starting reporting performance")
	require.LogsContain(t, hook, "\"Starting service\" ValidatorIndices=\"[1 2 12 15]\"")
	s.Lock()
	require.Equal(t, s.isLogging, true, "monitor is not running")
	s.Unlock()
}

func TestInitializePerformanceStructures(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	s := setupService(t)
	state, err := s.config.HeadFetcher.HeadState(ctx)
	require.NoError(t, err)
	epoch := slots.ToEpoch(state.Slot())
	s.initializePerformanceStructures(state, epoch)
	require.LogsDoNotContain(t, hook, "Could not fetch starting balance")
	latestPerformance := map[primitives.ValidatorIndex]ValidatorLatestPerformance{
		1: {
			balance: 32000000000,
		},
		2: {
			balance: 32000000000,
		},
		12: {
			balance: 32000000000,
		},
		15: {
			balance: 32000000000,
		},
	}
	aggregatedPerformance := map[primitives.ValidatorIndex]ValidatorAggregatedPerformance{
		1: {
			startBalance: 32000000000,
		},
		2: {
			startBalance: 32000000000,
		},
		12: {
			startBalance: 32000000000,
		},
		15: {
			startBalance: 32000000000,
		},
	}

	require.DeepEqual(t, s.latestPerformance, latestPerformance)
	require.DeepEqual(t, s.aggregatedPerformance, aggregatedPerformance)
}

func TestMonitorRoutine(t *testing.T) {
	ctx := context.Background()
	hook := logTest.NewGlobal()
	s := setupService(t)
	stateChannel := make(chan *feed.Event, 1)
	stateSub := s.config.StateNotifier.StateFeed().Subscribe(stateChannel)

	wg := &sync.WaitGroup{}
	wg.Add(1)

	go func() {
		s.monitorRoutine(stateChannel, stateSub)
		wg.Done()
	}()

	genesis, keys := util.DeterministicGenesisStateAltair(t, 64)
	c, err := altair.NextSyncCommittee(ctx, genesis)
	require.NoError(t, err)
	require.NoError(t, genesis.SetCurrentSyncCommittee(c))

	genConfig := util.DefaultBlockGenConfig()
	block, err := util.GenerateFullBlockAltair(genesis, keys, genConfig, 1)
	require.NoError(t, err)
	root, err := block.GetBlock().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, s.config.StateGen.SaveState(ctx, root, genesis))

	wrapped, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	stateChannel <- &feed.Event{
		Type: statefeed.BlockProcessed,
		Data: &statefeed.BlockProcessedData{
			Slot:        1,
			Verified:    true,
			SignedBlock: wrapped,
		},
	}

	// Wait for Logrus
	time.Sleep(1000 * time.Millisecond)
	wanted1 := fmt.Sprintf("\"Proposed beacon block was included\" BalanceChange=100000000 BlockRoot=%#x NewBalance=32000000000 ParentRoot=0xf732eaeb7fae ProposerIndex=15 Slot=1 Version=1 prefix=monitor", bytesutil.Trunc(root[:]))
	require.LogsContain(t, hook, wanted1)

}

func TestWaitForSync(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	s := &Service{ctx: ctx}
	syncChan := make(chan struct{})

	go func() {
		// Failsafe to make sure tests never get deadlocked; we should always go through the happy path before 500ms.
		// Otherwise, the NoError assertion below will fail.
		time.Sleep(500 * time.Millisecond)
		cancel()
	}()
	go func() {
		close(syncChan)
	}()
	require.NoError(t, s.waitForSync(syncChan))
}

func TestWaitForSyncCanceled(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	s := &Service{ctx: ctx}
	syncChan := make(chan struct{})

	cancel()
	require.ErrorIs(t, s.waitForSync(syncChan), errContextClosedWhileWaiting)
}

func TestRun(t *testing.T) {
	hook := logTest.NewGlobal()
	s := setupService(t)

	go func() {
		s.run()
	}()
	close(s.config.InitialSyncComplete)

	time.Sleep(100 * time.Millisecond)
	require.LogsContain(t, hook, "Synced to head epoch, starting reporting performance")
}
