package evidence_test

import (
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/internal/evidence"
	"github.com/cometbft/cometbft/internal/evidence/mocks"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	sm "github.com/cometbft/cometbft/state"
	smmocks "github.com/cometbft/cometbft/state/mocks"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

func TestMain(m *testing.M) {
	code := m.Run()
	os.Exit(code)
}

const evidenceChainID = "test_chain"

var (
	defaultEvidenceTime           = time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)
	defaultEvidenceMaxBytes int64 = 1000
)

func TestEvidencePoolBasic(t *testing.T) {
	var (
		height     = int64(1)
		stateStore = &smmocks.Store{}
		evidenceDB = dbm.NewMemDB()
		blockStore = &mocks.BlockStore{}
	)

	valSet, privVals := types.RandValidatorSet(1, 10)

	blockStore.On("LoadBlockMeta", mock.AnythingOfType("int64")).Return(
		&types.BlockMeta{Header: types.Header{Time: defaultEvidenceTime}},
	)
	stateStore.On("LoadValidators", mock.AnythingOfType("int64")).Return(valSet, nil)
	stateStore.On("Load").Return(createState(height+1, valSet), nil)

	require.Panics(t, func() { _, _ = evidence.NewPool(evidenceDB, stateStore, blockStore, evidence.WithDBKeyLayout("2")) }, "failed to create tore")

	pool, err := evidence.NewPool(evidenceDB, stateStore, blockStore, evidence.WithDBKeyLayout("v2"))
	require.NoError(t, err)

	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())

	// evidence not seen yet:
	evs, size := pool.PendingEvidence(defaultEvidenceMaxBytes)
	require.Empty(t, evs)
	require.Zero(t, size)

	ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height, defaultEvidenceTime, privVals[0], evidenceChainID)
	require.NoError(t, err)

	// good evidence
	evAdded := make(chan struct{})
	go func() {
		<-pool.EvidenceWaitChan()
		close(evAdded)
	}()

	// evidence seen but not yet committed:
	require.NoError(t, pool.AddEvidence(ev))

	select {
	case <-evAdded:
	case <-time.After(5 * time.Second):
		t.Fatal("evidence was not added to list after 5s")
	}

	next := pool.EvidenceFront()
	assert.Equal(t, ev, next.Value.(types.Evidence))

	const evidenceBytes int64 = 372
	evs, size = pool.PendingEvidence(evidenceBytes)
	assert.Len(t, evs, 1)
	assert.Equal(t, evidenceBytes, size) // check that the size of the single evidence in bytes is correct

	// shouldn't be able to add evidence twice
	require.NoError(t, pool.AddEvidence(ev))
	evs, _ = pool.PendingEvidence(defaultEvidenceMaxBytes)
	assert.Len(t, evs, 1)
}

// Tests inbound evidence for the right time and height.
func TestAddExpiredEvidence(t *testing.T) {
	var (
		val                 = types.NewMockPV()
		height              = int64(30)
		stateStore          = initializeValidatorState(val, height)
		evidenceDB          = dbm.NewMemDB()
		blockStore          = &mocks.BlockStore{}
		expiredEvidenceTime = time.Date(2018, 1, 1, 0, 0, 0, 0, time.UTC)
		expiredHeight       = int64(2)
	)

	blockStore.On("LoadBlockMeta", mock.AnythingOfType("int64")).Return(func(h int64) *types.BlockMeta {
		if h == height || h == expiredHeight {
			return &types.BlockMeta{Header: types.Header{Time: defaultEvidenceTime}}
		}
		return &types.BlockMeta{Header: types.Header{Time: expiredEvidenceTime}}
	})

	pool, err := evidence.NewPool(evidenceDB, stateStore, blockStore)
	require.NoError(t, err)

	testCases := []struct {
		evHeight      int64
		evTime        time.Time
		expErr        bool
		evDescription string
	}{
		{height, defaultEvidenceTime, false, "valid evidence"},
		{expiredHeight, defaultEvidenceTime, false, "valid evidence (despite old height)"},
		{height - 1, expiredEvidenceTime, false, "valid evidence (despite old time)"},
		{
			expiredHeight - 1, expiredEvidenceTime, true,
			"evidence from height 1 (created at: 2019-01-01 00:00:00 +0000 UTC) is too old",
		},
		{height, defaultEvidenceTime.Add(1 * time.Minute), true, "evidence time and block time is different"},
	}

	for _, tc := range testCases {
		t.Run(tc.evDescription, func(t *testing.T) {
			ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(tc.evHeight, tc.evTime, val, evidenceChainID)
			require.NoError(t, err)
			err = pool.AddEvidence(ev)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestReportConflictingVotes(t *testing.T) {
	var height int64 = 10

	pool, pv := defaultTestPool(t, height)
	val := types.NewValidator(pv.PrivKey.PubKey(), 10)
	ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height+1, defaultEvidenceTime, pv, evidenceChainID)
	require.NoError(t, err)

	pool.ReportConflictingVotes(ev.VoteA, ev.VoteB)

	// shouldn't be able to submit the same evidence twice
	pool.ReportConflictingVotes(ev.VoteA, ev.VoteB)

	// evidence from consensus should not be added immediately but reside in the consensus buffer
	evList, evSize := pool.PendingEvidence(defaultEvidenceMaxBytes)
	require.Empty(t, evList)
	require.Zero(t, evSize)

	next := pool.EvidenceFront()
	require.Nil(t, next)

	// move to next height and update state and evidence pool
	state := pool.State()
	state.LastBlockHeight++
	state.LastBlockTime = ev.Time()
	state.LastValidators = types.NewValidatorSet([]*types.Validator{val})
	pool.Update(state, []types.Evidence{})

	// should be able to retrieve evidence from pool
	evList, _ = pool.PendingEvidence(defaultEvidenceMaxBytes)
	require.Equal(t, []types.Evidence{ev}, evList)

	next = pool.EvidenceFront()
	require.NotNil(t, next)
}

func TestEvidencePoolUpdate(t *testing.T) {
	height := int64(21)
	pool, val := defaultTestPool(t, height)
	state := pool.State()

	// create new block (no need to save it to blockStore)
	prunedEv, err := types.NewMockDuplicateVoteEvidenceWithValidator(1, defaultEvidenceTime.Add(1*time.Minute),
		val, evidenceChainID)
	require.NoError(t, err)
	err = pool.AddEvidence(prunedEv)
	require.NoError(t, err)
	ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height, defaultEvidenceTime.Add(21*time.Minute),
		val, evidenceChainID)
	require.NoError(t, err)
	lastExtCommit := makeExtCommit(height, val.PrivKey.PubKey().Address())
	block := types.MakeBlock(height+1, []types.Tx{}, lastExtCommit.ToCommit(), []types.Evidence{ev})
	// update state (partially)
	state.LastBlockHeight = height + 1
	state.LastBlockTime = defaultEvidenceTime.Add(22 * time.Minute)
	err = pool.CheckEvidence(types.EvidenceList{ev})
	require.NoError(t, err)

	pool.Update(state, block.Evidence.Evidence)
	// a) Update marks evidence as committed so pending evidence should be empty
	evList, evSize := pool.PendingEvidence(defaultEvidenceMaxBytes)
	assert.Empty(t, evList)
	assert.Zero(t, evSize)

	// b) If we try to check this evidence again it should fail because it has already been committed
	err = pool.CheckEvidence(types.EvidenceList{ev})
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, evidence.ErrEvidenceAlreadyCommitted.Error(), err.(*types.ErrInvalidEvidence).Reason.Error())
	}
}

func TestVerifyPendingEvidencePasses(t *testing.T) {
	var height int64 = 1
	pool, val := defaultTestPool(t, height)
	ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height, defaultEvidenceTime.Add(1*time.Minute),
		val, evidenceChainID)
	require.NoError(t, err)
	err = pool.AddEvidence(ev)
	require.NoError(t, err)

	err = pool.CheckEvidence(types.EvidenceList{ev})
	require.NoError(t, err)
}

func TestVerifyDuplicatedEvidenceFails(t *testing.T) {
	var height int64 = 1
	pool, val := defaultTestPool(t, height)
	ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(height, defaultEvidenceTime.Add(1*time.Minute),
		val, evidenceChainID)
	require.NoError(t, err)
	err = pool.CheckEvidence(types.EvidenceList{ev, ev})
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, evidence.ErrDuplicateEvidence.Error(), err.(*types.ErrInvalidEvidence).Reason.Error())
	}
}

// check that valid light client evidence is correctly validated and stored in
// evidence pool.
func TestLightClientAttackEvidenceLifecycle(t *testing.T) {
	var (
		height       int64 = 100
		commonHeight int64 = 90
	)

	ev, trusted, common := makeLunaticEvidence(t, height, commonHeight,
		5, 5, defaultEvidenceTime, defaultEvidenceTime.Add(1*time.Hour))

	state := sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(2 * time.Hour),
		LastBlockHeight: 110,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", height).Return(trusted.ValidatorSet, nil)
	stateStore.On("LoadValidators", commonHeight).Return(common.ValidatorSet, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", height).Return(&types.BlockMeta{Header: *trusted.Header})
	blockStore.On("LoadBlockMeta", commonHeight).Return(&types.BlockMeta{Header: *common.Header})
	blockStore.On("LoadBlockCommit", height).Return(trusted.Commit)
	blockStore.On("LoadBlockCommit", commonHeight).Return(common.Commit)

	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())

	err = pool.AddEvidence(ev)
	require.NoError(t, err)

	hash := ev.Hash()

	require.NoError(t, pool.AddEvidence(ev))
	require.NoError(t, pool.AddEvidence(ev))

	pendingEv, _ := pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	require.Len(t, pendingEv, 1)
	require.Equal(t, ev, pendingEv[0])

	require.NoError(t, pool.CheckEvidence(pendingEv))
	require.Equal(t, ev, pendingEv[0])

	state.LastBlockHeight++
	state.LastBlockTime = state.LastBlockTime.Add(1 * time.Minute)
	pool.Update(state, pendingEv)
	require.Equal(t, hash, pendingEv[0].Hash())

	remaindingEv, _ := pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	require.Empty(t, remaindingEv)

	// evidence is already committed so it shouldn't pass
	require.Error(t, pool.CheckEvidence(types.EvidenceList{ev}))
	require.NoError(t, pool.AddEvidence(ev))

	remaindingEv, _ = pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	require.Empty(t, remaindingEv)
}

// Tests that restarting the evidence pool after a potential failure will recover the
// pending evidence and continue to gossip it.
func TestRecoverPendingEvidence(t *testing.T) {
	height := int64(10)
	val := types.NewMockPV()
	valAddress := val.PrivKey.PubKey().Address()
	evidenceDB := dbm.NewMemDB()
	stateStore := initializeValidatorState(val, height)
	state, err := stateStore.Load()
	require.NoError(t, err)
	blockStore, err := initializeBlockStore(dbm.NewMemDB(), state, valAddress)
	require.NoError(t, err)
	// create previous pool and populate it
	pool, err := evidence.NewPool(evidenceDB, stateStore, blockStore)
	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())
	goodEvidence, err := types.NewMockDuplicateVoteEvidenceWithValidator(height,
		defaultEvidenceTime.Add(10*time.Minute), val, evidenceChainID)
	require.NoError(t, err)
	expiredEvidence, err := types.NewMockDuplicateVoteEvidenceWithValidator(int64(1),
		defaultEvidenceTime.Add(1*time.Minute), val, evidenceChainID)
	require.NoError(t, err)
	err = pool.AddEvidence(goodEvidence)
	require.NoError(t, err)
	err = pool.AddEvidence(expiredEvidence)
	require.NoError(t, err)

	// now recover from the previous pool at a different time
	newStateStore := &smmocks.Store{}
	newStateStore.On("Load").Return(sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(25 * time.Minute),
		LastBlockHeight: height + 15,
		ConsensusParams: types.ConsensusParams{
			Block: types.BlockParams{
				MaxBytes: 22020096,
				MaxGas:   -1,
			},
			Evidence: types.EvidenceParams{
				MaxAgeNumBlocks: 20,
				MaxAgeDuration:  20 * time.Minute,
				MaxBytes:        defaultEvidenceMaxBytes,
			},
		},
	}, nil)
	newPool, err := evidence.NewPool(evidenceDB, newStateStore, blockStore)
	require.NoError(t, err)
	evList, _ := newPool.PendingEvidence(defaultEvidenceMaxBytes)
	require.Len(t, evList, 1)
	next := newPool.EvidenceFront()
	require.Equal(t, goodEvidence, next.Value.(types.Evidence))
}

func initializeStateFromValidatorSet(valSet *types.ValidatorSet, height int64) sm.Store {
	stateDB := dbm.NewMemDB()
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state := sm.State{
		ChainID:                     evidenceChainID,
		InitialHeight:               1,
		LastBlockHeight:             height,
		LastBlockTime:               defaultEvidenceTime,
		Validators:                  valSet,
		NextValidators:              valSet.CopyIncrementProposerPriority(1),
		LastValidators:              valSet,
		LastHeightValidatorsChanged: 1,
		ConsensusParams: types.ConsensusParams{
			Block: types.BlockParams{
				MaxBytes: 22020096,
				MaxGas:   -1,
			},
			Evidence: types.EvidenceParams{
				MaxAgeNumBlocks: 20,
				MaxAgeDuration:  20 * time.Minute,
				MaxBytes:        1000,
			},
		},
	}

	// save all states up to height
	for i := int64(0); i <= height; i++ {
		state.LastBlockHeight = i
		if err := stateStore.Save(state); err != nil {
			panic(err)
		}
	}

	return stateStore
}

func initializeValidatorState(privVal types.PrivValidator, height int64) sm.Store {
	pubKey, _ := privVal.GetPubKey()
	validator := &types.Validator{Address: pubKey.Address(), VotingPower: 10, PubKey: pubKey}

	// create validator set and state
	valSet := &types.ValidatorSet{
		Validators: []*types.Validator{validator},
		Proposer:   validator,
	}

	return initializeStateFromValidatorSet(valSet, height)
}

// initializeBlockStore creates a block storage and populates it w/ a dummy
// block at +height+.
func initializeBlockStore(db dbm.DB, state sm.State, valAddr []byte) (*store.BlockStore, error) {
	blockStore := store.NewBlockStore(db)

	for i := int64(1); i <= state.LastBlockHeight; i++ {
		lastCommit := makeExtCommit(i-1, valAddr)
		block := state.MakeBlock(i, test.MakeNTxs(i, 1), lastCommit.ToCommit(), nil, state.Validators.Proposer.Address)
		block.Header.Time = defaultEvidenceTime.Add(time.Duration(i) * time.Minute)
		block.Header.Version = cmtversion.Consensus{Block: version.BlockProtocol, App: 1}
		partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
		if err != nil {
			return nil, err
		}

		seenCommit := makeExtCommit(i, valAddr)
		blockStore.SaveBlockWithExtendedCommit(block, partSet, seenCommit)
	}

	return blockStore, nil
}

func makeExtCommit(height int64, valAddr []byte) *types.ExtendedCommit {
	return &types.ExtendedCommit{
		Height: height,
		ExtendedSignatures: []types.ExtendedCommitSig{{
			CommitSig: types.CommitSig{
				BlockIDFlag:      types.BlockIDFlagCommit,
				ValidatorAddress: valAddr,
				Timestamp:        defaultEvidenceTime,
				Signature:        []byte("Signature"),
			},
			ExtensionSignature: []byte("Extended Signature"),
		}},
	}
}

func defaultTestPool(t *testing.T, height int64) (*evidence.Pool, types.MockPV) {
	t.Helper()
	val := types.NewMockPV()
	valAddress := val.PrivKey.PubKey().Address()
	evidenceDB := dbm.NewMemDB()
	stateStore := initializeValidatorState(val, height)
	state, _ := stateStore.Load()
	blockStore, err := initializeBlockStore(dbm.NewMemDB(), state, valAddress)
	require.NoError(t, err)
	pool, err := evidence.NewPool(evidenceDB, stateStore, blockStore)
	if err != nil {
		panic("test evidence pool could not be created")
	}
	pool.SetLogger(log.TestingLogger())
	return pool, val
}

func createState(height int64, valSet *types.ValidatorSet) sm.State {
	return sm.State{
		ChainID:         evidenceChainID,
		LastBlockHeight: height,
		LastBlockTime:   defaultEvidenceTime,
		Validators:      valSet,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
}
