package evidence_test

import (
	"encoding/hex"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/fortytw2/leaktest"
	"github.com/go-kit/log/term"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/internal/evidence"
	"github.com/cometbft/cometbft/internal/evidence/mocks"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/p2p"
	p2pmocks "github.com/cometbft/cometbft/p2p/mocks"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
)

var (
	numEvidence = 10
	timeout     = 120 * time.Second // ridiculously high because CircleCI is slow
)

// We have N evidence reactors connected to one another. The first reactor
// receives a number of evidence at varying heights. We test that all
// other reactors receive the evidence and add it to their own respective
// evidence pools.
func TestReactorBroadcastEvidence(t *testing.T) {
	config := cfg.TestConfig()
	n := 7

	// create statedb for everyone
	stateDBs := make([]sm.Store, n)
	val := types.NewMockPV()
	// we need validators saved for heights at least as high as we have evidence for
	height := int64(numEvidence) + 10
	for i := 0; i < n; i++ {
		stateDBs[i] = initializeValidatorState(val, height)
	}

	// make reactors from statedb
	reactors, pools := makeAndConnectReactorsAndPools(config, stateDBs)

	// set the peer height on each reactor
	for _, r := range reactors {
		for _, peer := range r.Switch.Peers().Copy() {
			ps := peerState{height}
			peer.Set(types.PeerStateKey, ps)
		}
	}

	// send a bunch of valid evidence to the first reactor's evpool
	// and wait for them all to be received in the others
	evList := sendEvidence(t, pools[0], val, numEvidence)
	waitForEvidence(t, evList, pools)
}

// We have two evidence reactors connected to one another but are at different heights.
// Reactor 1 which is ahead receives a number of evidence. It should only send the evidence
// that is below the height of the peer to that peer.
func TestReactorSelectiveBroadcast(t *testing.T) {
	config := cfg.TestConfig()

	val := types.NewMockPV()
	height1 := int64(numEvidence) + 10
	height2 := int64(numEvidence) / 2

	// DB1 is ahead of DB2
	stateDB1 := initializeValidatorState(val, height1)
	stateDB2 := initializeValidatorState(val, height2)

	// make reactors from statedb
	reactors, pools := makeAndConnectReactorsAndPools(config, []sm.Store{stateDB1, stateDB2})

	// set the peer height on each reactor
	for _, r := range reactors {
		for _, peer := range r.Switch.Peers().Copy() {
			ps := peerState{height1}
			peer.Set(types.PeerStateKey, ps)
		}
	}

	// update the first reactor peer's height to be very small
	peer := reactors[0].Switch.Peers().Copy()[0]
	ps := peerState{height2}
	peer.Set(types.PeerStateKey, ps)

	// send a bunch of valid evidence to the first reactor's evpool
	evList := sendEvidence(t, pools[0], val, numEvidence)

	// only ones less than the peers height should make it through
	waitForEvidence(t, evList[:numEvidence/2-1], []*evidence.Pool{pools[1]})

	// peers should still be connected
	peers := reactors[1].Switch.Peers().Copy()
	assert.Len(t, peers, 1)
}

// This tests aims to ensure that reactors don't send evidence that they have committed or that ar
// not ready for the peer through three scenarios.
// First, committed evidence to a newly connected peer
// Second, evidence to a peer that is behind
// Third, evidence that was pending and became committed just before the peer caught up.
func TestReactorsGossipNoCommittedEvidence(t *testing.T) {
	config := cfg.TestConfig()

	val := types.NewMockPV()
	var height int64 = 10

	// DB1 is ahead of DB2
	stateDB1 := initializeValidatorState(val, height-1)
	stateDB2 := initializeValidatorState(val, height-2)
	state, err := stateDB1.Load()
	require.NoError(t, err)
	state.LastBlockHeight++

	// make reactors from statedb
	reactors, pools := makeAndConnectReactorsAndPools(config, []sm.Store{stateDB1, stateDB2})

	evList := sendEvidence(t, pools[0], val, 2)
	pools[0].Update(state, evList)
	require.EqualValues(t, uint32(0), pools[0].Size())

	time.Sleep(100 * time.Millisecond)

	peer := reactors[0].Switch.Peers().Copy()[0]
	ps := peerState{height - 2}
	peer.Set(types.PeerStateKey, ps)

	peer = reactors[1].Switch.Peers().Copy()[0]
	ps = peerState{height}
	peer.Set(types.PeerStateKey, ps)

	// wait to see that no evidence comes through
	time.Sleep(300 * time.Millisecond)

	// the second pool should not have received any evidence because it has already been committed
	assert.Equal(t, uint32(0), pools[1].Size(), "second reactor should not have received evidence")

	// the first reactor receives three more evidence
	evList = make([]types.Evidence, 3)
	for i := 0; i < 3; i++ {
		ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height-3+int64(i),
			time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC), val, state.ChainID)
		require.NoError(t, err)
		err = pools[0].AddEvidence(ev)
		require.NoError(t, err)
		evList[i] = ev
	}

	// wait to see that only one evidence is sent
	time.Sleep(300 * time.Millisecond)

	// the second pool should only have received the first evidence because it is behind
	peerEv, _ := pools[1].PendingEvidence(10000)
	assert.EqualValues(t, []types.Evidence{evList[0]}, peerEv)

	// the last evidence is committed and the second reactor catches up in state to the first
	// reactor. We therefore expect that the second reactor only receives one more evidence, the
	// one that is still pending and not the evidence that has already been committed.
	state.LastBlockHeight++
	pools[0].Update(state, []types.Evidence{evList[2]})
	// the first reactor should have the two remaining pending evidence
	require.EqualValues(t, uint32(2), pools[0].Size())

	// now update the state of the second reactor
	pools[1].Update(state, types.EvidenceList{})
	peer = reactors[0].Switch.Peers().Copy()[0]
	ps = peerState{height}
	peer.Set(types.PeerStateKey, ps)

	// wait to see that only two evidence is sent
	time.Sleep(300 * time.Millisecond)

	peerEv, _ = pools[1].PendingEvidence(1000)
	assert.EqualValues(t, []types.Evidence{evList[0], evList[1]}, peerEv)
}

func TestReactorBroadcastEvidenceMemoryLeak(t *testing.T) {
	evidenceTime := time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)
	evidenceDB := dbm.NewMemDB()
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", mock.AnythingOfType("int64")).Return(
		&types.BlockMeta{Header: types.Header{Time: evidenceTime}},
	)
	val := types.NewMockPV()
	stateStore := initializeValidatorState(val, 1)
	pool, err := evidence.NewPool(evidenceDB, stateStore, blockStore)
	require.NoError(t, err)

	p := &p2pmocks.Peer{}

	p.On("IsRunning").Once().Return(true)
	p.On("IsRunning").Return(false)
	// check that we are not leaking any go-routines
	// i.e. broadcastEvidenceRoutine finishes when peer is stopped
	defer leaktest.CheckTimeout(t, 10*time.Second)()

	p.On("Send", mock.MatchedBy(func(i any) bool {
		e, ok := i.(p2p.Envelope)
		return ok && e.ChannelID == evidence.EvidenceChannel
	})).Return(false)
	quitChan := make(<-chan struct{})
	p.On("Quit").Return(quitChan)
	ps := peerState{2}
	p.On("Get", types.PeerStateKey).Return(ps)
	p.On("ID").Return("ABC")
	p.On("String").Return("mock")

	r := evidence.NewReactor(pool)
	r.SetLogger(log.TestingLogger())
	r.AddPeer(p)

	_ = sendEvidence(t, pool, val, 2)
}

// evidenceLogger is a TestingLogger which uses a different
// color for each validator ("validator" key must exist).
func evidenceLogger() log.Logger {
	return log.TestingLoggerWithColorFn(func(keyvals ...any) term.FgBgColor {
		for i := 0; i < len(keyvals)-1; i += 2 {
			if keyvals[i] == "validator" {
				return term.FgBgColor{Fg: term.Color(uint8(keyvals[i+1].(int) + 1))}
			}
		}
		return term.FgBgColor{}
	})
}

// connect N evidence reactors through N switches.
func makeAndConnectReactorsAndPools(config *cfg.Config, stateStores []sm.Store) ([]*evidence.Reactor,
	[]*evidence.Pool,
) {
	n := len(stateStores)

	reactors := make([]*evidence.Reactor, n)
	pools := make([]*evidence.Pool, n)
	logger := evidenceLogger()
	evidenceTime := time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)

	for i := 0; i < n; i++ {
		evidenceDB := dbm.NewMemDB()
		blockStore := &mocks.BlockStore{}
		blockStore.On("LoadBlockMeta", mock.AnythingOfType("int64")).Return(
			&types.BlockMeta{Header: types.Header{Time: evidenceTime}},
		)
		pool, err := evidence.NewPool(evidenceDB, stateStores[i], blockStore)
		if err != nil {
			panic(err)
		}
		pools[i] = pool
		reactors[i] = evidence.NewReactor(pool)
		reactors[i].SetLogger(logger.With("validator", i))
	}

	p2p.MakeConnectedSwitches(config.P2P, n, func(i int, s *p2p.Switch) *p2p.Switch {
		s.AddReactor("EVIDENCE", reactors[i])
		return s
	}, p2p.Connect2Switches)

	return reactors, pools
}

// wait for all evidence on all reactors.
func waitForEvidence(t *testing.T, evs types.EvidenceList, pools []*evidence.Pool) {
	t.Helper()
	// wait for the evidence in all evpools
	wg := new(sync.WaitGroup)
	for i := 0; i < len(pools); i++ {
		wg.Add(1)
		go _waitForEvidence(t, wg, evs, i, pools)
	}

	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	timer := time.After(timeout)
	select {
	case <-timer:
		t.Fatal("Timed out waiting for evidence")
	case <-done:
	}
}

// wait for all evidence on a single evpool.
func _waitForEvidence(
	t *testing.T,
	wg *sync.WaitGroup,
	evs types.EvidenceList,
	poolIdx int,
	pools []*evidence.Pool,
) {
	t.Helper()
	evpool := pools[poolIdx]
	var evList []types.Evidence
	currentPoolSize := 0
	for currentPoolSize != len(evs) {
		evList, _ = evpool.PendingEvidence(int64(len(evs) * 500)) // each evidence should not be more than 500 bytes
		currentPoolSize = len(evList)
		time.Sleep(time.Millisecond * 100)
	}

	// put the reaped evidence in a map so we can quickly check we got everything
	evMap := make(map[string]types.Evidence)
	for _, e := range evList {
		evMap[string(e.Hash())] = e
	}
	for i, expectedEv := range evs {
		gotEv := evMap[string(expectedEv.Hash())]
		assert.Equal(t, expectedEv, gotEv,
			fmt.Sprintf("evidence at index %d on pool %d don't match: %v vs %v",
				i, poolIdx, expectedEv, gotEv))
	}

	wg.Done()
}

func sendEvidence(t *testing.T, evpool *evidence.Pool, val types.PrivValidator, n int) types.EvidenceList {
	t.Helper()
	evList := make([]types.Evidence, n)
	for i := 0; i < n; i++ {
		ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(int64(i+1),
			time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC), val, evidenceChainID)
		require.NoError(t, err)
		err = evpool.AddEvidence(ev)
		require.NoError(t, err)
		evList[i] = ev
	}
	return evList
}

type peerState struct {
	height int64
}

func (ps peerState) GetHeight() int64 {
	return ps.height
}

func exampleVote(t byte) *types.Vote {
	stamp, err := time.Parse(types.TimeFormat, "2017-12-25T03:00:01.234Z")
	if err != nil {
		panic(err)
	}

	return &types.Vote{
		Type:      types.SignedMsgType(t),
		Height:    3,
		Round:     2,
		Timestamp: stamp,
		BlockID: types.BlockID{
			Hash: tmhash.Sum([]byte("blockID_hash")),
			PartSetHeader: types.PartSetHeader{
				Total: 1000000,
				Hash:  tmhash.Sum([]byte("blockID_part_set_header_hash")),
			},
		},
		ValidatorAddress: crypto.AddressHash([]byte("validator_address")),
		ValidatorIndex:   56789,
	}
}

//nolint:lll //ignore line length for tests
func TestEvidenceVectors(t *testing.T) {
	val := &types.Validator{
		Address:     crypto.AddressHash([]byte("validator_address")),
		VotingPower: 10,
	}

	valSet := types.NewValidatorSet([]*types.Validator{val})

	dupl, err := types.NewDuplicateVoteEvidence(
		exampleVote(1),
		exampleVote(2),
		defaultEvidenceTime,
		valSet,
	)
	require.NoError(t, err)

	testCases := []struct {
		testName     string
		evidenceList []types.Evidence
		expBytes     string
	}{
		{"DuplicateVoteEvidence", []types.Evidence{dupl}, "0a85020a82020a79080210031802224a0a208b01023386c371778ecb6368573e539afc3cc860ec3a2f614e54fe5652f4fc80122608c0843d122072db3d959635dff1bb567bedaa70573392c5159666a3f8caf11e413aac52207a2a0b08b1d381d20510809dca6f32146af1f4111082efb388211bc72c55bcd61e9ac3d538d5bb031279080110031802224a0a208b01023386c371778ecb6368573e539afc3cc860ec3a2f614e54fe5652f4fc80122608c0843d122072db3d959635dff1bb567bedaa70573392c5159666a3f8caf11e413aac52207a2a0b08b1d381d20510809dca6f32146af1f4111082efb388211bc72c55bcd61e9ac3d538d5bb03180a200a2a060880dbaae105"},
	}

	for _, tc := range testCases {
		evi := make([]cmtproto.Evidence, len(tc.evidenceList))
		for i := 0; i < len(tc.evidenceList); i++ {
			ev, err := types.EvidenceToProto(tc.evidenceList[i])
			require.NoError(t, err, tc.testName)
			evi[i] = *ev
		}

		epl := cmtproto.EvidenceList{
			Evidence: evi,
		}

		bz, err := epl.Marshal()
		require.NoError(t, err, tc.testName)

		require.Equal(t, tc.expBytes, hex.EncodeToString(bz), tc.testName)
	}
}
