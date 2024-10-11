package evidence_test

import (
	"bytes"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/internal/evidence"
	"github.com/cometbft/cometbft/internal/evidence/mocks"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	sm "github.com/cometbft/cometbft/state"
	smmocks "github.com/cometbft/cometbft/state/mocks"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

const (
	defaultVotingPower = 10
)

func TestVerifyLightClientAttack_Lunatic(t *testing.T) {
	const (
		height       int64 = 10
		commonHeight int64 = 4
		totalVals          = 10
		byzVals            = 4
	)
	attackTime := defaultEvidenceTime.Add(1 * time.Hour)
	// create valid lunatic evidence
	ev, trusted, common := makeLunaticEvidence(
		t, height, commonHeight, byzVals, totalVals-byzVals, defaultEvidenceTime, attackTime)
	require.NoError(t, ev.ValidateBasic())

	// good pass -> no error
	err := evidence.VerifyLightClientAttack(ev, common.SignedHeader, trusted.SignedHeader, common.ValidatorSet)
	require.NoError(t, err)

	// trusted and conflicting hashes are the same -> an error should be returned
	err = evidence.VerifyLightClientAttack(ev, common.SignedHeader, ev.ConflictingBlock.SignedHeader, common.ValidatorSet)
	require.Error(t, err)

	// evidence with different total validator power should fail
	ev.TotalVotingPower = 1 * defaultVotingPower
	err = evidence.VerifyLightClientAttack(ev, common.SignedHeader, trusted.SignedHeader, common.ValidatorSet)
	require.Error(t, err)

	// evidence without enough malicious votes should fail
	ev, trusted, common = makeLunaticEvidence(
		t, height, commonHeight, byzVals-1, totalVals-byzVals, defaultEvidenceTime, attackTime)
	err = evidence.VerifyLightClientAttack(ev, common.SignedHeader, trusted.SignedHeader, common.ValidatorSet)
	require.Error(t, err)
}

func TestVerify_LunaticAttackAgainstState(t *testing.T) {
	const (
		height       int64 = 10
		commonHeight int64 = 4
		totalVals          = 10
		byzVals            = 4
	)
	attackTime := defaultEvidenceTime.Add(1 * time.Hour)
	// create valid lunatic evidence
	ev, trusted, common := makeLunaticEvidence(
		t, height, commonHeight, byzVals, totalVals-byzVals, defaultEvidenceTime, attackTime)

	// now we try to test verification against state
	state := sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(2 * time.Hour),
		LastBlockHeight: height + 1,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", commonHeight).Return(common.ValidatorSet, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", commonHeight).Return(&types.BlockMeta{Header: *common.Header})
	blockStore.On("LoadBlockMeta", height).Return(&types.BlockMeta{Header: *trusted.Header})
	blockStore.On("LoadBlockCommit", commonHeight).Return(common.Commit)
	blockStore.On("LoadBlockCommit", height).Return(trusted.Commit)
	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())

	evList := types.EvidenceList{ev}
	// check that the evidence pool correctly verifies the evidence
	require.NoError(t, pool.CheckEvidence(evList))

	// as it was not originally in the pending bucket, it should now have been added
	pendingEvs, _ := pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	assert.Len(t, pendingEvs, 1)
	assert.Equal(t, ev, pendingEvs[0])

	// if we submit evidence only against a single byzantine validator when we see there are more validators then this
	// should return an error
	ev.ByzantineValidators = ev.ByzantineValidators[:1]
	t.Log(evList)
	require.Error(t, pool.CheckEvidence(evList))
	// restore original byz vals
	ev.ByzantineValidators = ev.GetByzantineValidators(common.ValidatorSet, trusted.SignedHeader)

	// duplicate evidence should be rejected
	evList = types.EvidenceList{ev, ev}
	pool, err = evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	require.Error(t, pool.CheckEvidence(evList))

	// If evidence is submitted with an altered timestamp it should return an error
	ev.Timestamp = defaultEvidenceTime.Add(1 * time.Minute)
	pool, err = evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	require.Error(t, pool.AddEvidence(ev))
	ev.Timestamp = defaultEvidenceTime

	// Evidence submitted with a different validator power should fail
	ev.TotalVotingPower = 1
	pool, err = evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	require.Error(t, pool.AddEvidence(ev))
	ev.TotalVotingPower = common.ValidatorSet.TotalVotingPower()
}

func TestVerify_ForwardLunaticAttack(t *testing.T) {
	const (
		nodeHeight   int64 = 8
		attackHeight int64 = 10
		commonHeight int64 = 4
		totalVals          = 10
		byzVals            = 5
	)
	attackTime := defaultEvidenceTime.Add(1 * time.Hour)

	// create a forward lunatic attack
	ev, trusted, common := makeLunaticEvidence(
		t, attackHeight, commonHeight, byzVals, totalVals-byzVals, defaultEvidenceTime, attackTime)

	// now we try to test verification against state
	state := sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(2 * time.Hour),
		LastBlockHeight: nodeHeight,
		ConsensusParams: *types.DefaultConsensusParams(),
	}

	// modify trusted light block so that it is of a height less than the conflicting one
	trusted.Header.Height = state.LastBlockHeight
	trusted.Header.Time = state.LastBlockTime

	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", commonHeight).Return(common.ValidatorSet, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", commonHeight).Return(&types.BlockMeta{Header: *common.Header})
	blockStore.On("LoadBlockMeta", nodeHeight).Return(&types.BlockMeta{Header: *trusted.Header})
	blockStore.On("LoadBlockMeta", attackHeight).Return(nil)
	blockStore.On("LoadBlockCommit", commonHeight).Return(common.Commit)
	blockStore.On("LoadBlockCommit", nodeHeight).Return(trusted.Commit)
	blockStore.On("Height").Return(nodeHeight)
	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)

	// check that the evidence pool correctly verifies the evidence
	require.NoError(t, pool.CheckEvidence(types.EvidenceList{ev}))

	// now we use a time which isn't able to contradict the FLA - thus we can't verify the evidence
	oldBlockStore := &mocks.BlockStore{}
	oldHeader := trusted.Header
	oldHeader.Time = defaultEvidenceTime
	oldBlockStore.On("LoadBlockMeta", commonHeight).Return(&types.BlockMeta{Header: *common.Header})
	oldBlockStore.On("LoadBlockMeta", nodeHeight).Return(&types.BlockMeta{Header: *oldHeader})
	oldBlockStore.On("LoadBlockMeta", attackHeight).Return(nil)
	oldBlockStore.On("LoadBlockCommit", commonHeight).Return(common.Commit)
	oldBlockStore.On("LoadBlockCommit", nodeHeight).Return(trusted.Commit)
	oldBlockStore.On("Height").Return(nodeHeight)
	require.Equal(t, defaultEvidenceTime, oldBlockStore.LoadBlockMeta(nodeHeight).Header.Time)

	pool, err = evidence.NewPool(dbm.NewMemDB(), stateStore, oldBlockStore)
	require.NoError(t, err)
	require.Error(t, pool.CheckEvidence(types.EvidenceList{ev}))
}

func TestVerifyLightClientAttack_Equivocation(t *testing.T) {
	conflictingVals, conflictingPrivVals := types.RandValidatorSet(5, 10)
	trustedHeader := makeHeaderRandom(10)

	conflictingHeader := makeHeaderRandom(10)
	conflictingHeader.ValidatorsHash = conflictingVals.Hash()

	trustedHeader.ValidatorsHash = conflictingHeader.ValidatorsHash
	trustedHeader.NextValidatorsHash = conflictingHeader.NextValidatorsHash
	trustedHeader.ConsensusHash = conflictingHeader.ConsensusHash
	trustedHeader.AppHash = conflictingHeader.AppHash
	trustedHeader.LastResultsHash = conflictingHeader.LastResultsHash

	// we are simulating a duplicate vote attack where all the validators in the conflictingVals set
	// except the last validator vote twice
	blockID := makeBlockID(conflictingHeader.Hash(), 1000, []byte("partshash"))
	voteSet := types.NewVoteSet(evidenceChainID, 10, 1, types.SignedMsgType(2), conflictingVals)
	commit, err := test.MakeCommitFromVoteSet(blockID, voteSet, conflictingPrivVals[:4], defaultEvidenceTime)
	require.NoError(t, err)
	ev := &types.LightClientAttackEvidence{
		ConflictingBlock: &types.LightBlock{
			SignedHeader: &types.SignedHeader{
				Header: conflictingHeader,
				Commit: commit,
			},
			ValidatorSet: conflictingVals,
		},
		CommonHeight:        10,
		ByzantineValidators: conflictingVals.Validators[:4],
		TotalVotingPower:    50,
		Timestamp:           defaultEvidenceTime,
	}

	trustedBlockID := makeBlockID(trustedHeader.Hash(), 1000, []byte("partshash"))
	trustedVoteSet := types.NewVoteSet(evidenceChainID, 10, 1, types.SignedMsgType(2), conflictingVals)
	trustedCommit, err := test.MakeCommitFromVoteSet(trustedBlockID, trustedVoteSet, conflictingPrivVals, defaultEvidenceTime)
	require.NoError(t, err)
	trustedSignedHeader := &types.SignedHeader{
		Header: trustedHeader,
		Commit: trustedCommit,
	}

	// good pass -> no error
	err = evidence.VerifyLightClientAttack(ev, trustedSignedHeader, trustedSignedHeader, conflictingVals)
	require.NoError(t, err)

	// trusted and conflicting hashes are the same -> an error should be returned
	err = evidence.VerifyLightClientAttack(ev, trustedSignedHeader, ev.ConflictingBlock.SignedHeader, conflictingVals)
	require.Error(t, err)

	// conflicting header has different next validators hash which should have been correctly derived from
	// the previous round
	ev.ConflictingBlock.Header.NextValidatorsHash = crypto.CRandBytes(tmhash.Size)
	err = evidence.VerifyLightClientAttack(ev, trustedSignedHeader, trustedSignedHeader, nil)
	require.Error(t, err)
	// revert next validators hash
	ev.ConflictingBlock.Header.NextValidatorsHash = trustedHeader.NextValidatorsHash

	state := sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(1 * time.Minute),
		LastBlockHeight: 11,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", int64(10)).Return(conflictingVals, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", int64(10)).Return(&types.BlockMeta{Header: *trustedHeader})
	blockStore.On("LoadBlockCommit", int64(10)).Return(trustedCommit)

	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())

	evList := types.EvidenceList{ev}
	err = pool.CheckEvidence(evList)
	require.NoError(t, err)

	pendingEvs, _ := pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	assert.Len(t, pendingEvs, 1)
}

func TestVerifyLightClientAttack_Amnesia(t *testing.T) {
	conflictingVals, conflictingPrivVals := types.RandValidatorSet(5, 10)

	conflictingHeader := makeHeaderRandom(10)
	conflictingHeader.ValidatorsHash = conflictingVals.Hash()
	trustedHeader := makeHeaderRandom(10)
	trustedHeader.ValidatorsHash = conflictingHeader.ValidatorsHash
	trustedHeader.NextValidatorsHash = conflictingHeader.NextValidatorsHash
	trustedHeader.AppHash = conflictingHeader.AppHash
	trustedHeader.ConsensusHash = conflictingHeader.ConsensusHash
	trustedHeader.LastResultsHash = conflictingHeader.LastResultsHash

	// we are simulating an amnesia attack where all the validators in the conflictingVals set
	// except the last validator vote twice. However this time the commits are of different rounds.
	blockID := makeBlockID(conflictingHeader.Hash(), 1000, []byte("partshash"))
	voteSet := types.NewVoteSet(evidenceChainID, 10, 0, types.SignedMsgType(2), conflictingVals)
	commit, err := test.MakeCommitFromVoteSet(blockID, voteSet, conflictingPrivVals, defaultEvidenceTime)
	require.NoError(t, err)
	ev := &types.LightClientAttackEvidence{
		ConflictingBlock: &types.LightBlock{
			SignedHeader: &types.SignedHeader{
				Header: conflictingHeader,
				Commit: commit,
			},
			ValidatorSet: conflictingVals,
		},
		CommonHeight:        10,
		ByzantineValidators: nil, // with amnesia evidence no validators are submitted as abci evidence
		TotalVotingPower:    50,
		Timestamp:           defaultEvidenceTime,
	}

	trustedBlockID := makeBlockID(trustedHeader.Hash(), 1000, []byte("partshash"))
	trustedVoteSet := types.NewVoteSet(evidenceChainID, 10, 1, types.SignedMsgType(2), conflictingVals)
	trustedCommit, err := test.MakeCommitFromVoteSet(trustedBlockID, trustedVoteSet, conflictingPrivVals, defaultEvidenceTime)
	require.NoError(t, err)
	trustedSignedHeader := &types.SignedHeader{
		Header: trustedHeader,
		Commit: trustedCommit,
	}

	// good pass -> no error
	err = evidence.VerifyLightClientAttack(ev, trustedSignedHeader, trustedSignedHeader, conflictingVals)
	require.NoError(t, err)

	// trusted and conflicting hashes are the same -> an error should be returned
	err = evidence.VerifyLightClientAttack(ev, trustedSignedHeader, ev.ConflictingBlock.SignedHeader, conflictingVals)
	require.Error(t, err)

	state := sm.State{
		LastBlockTime:   defaultEvidenceTime.Add(1 * time.Minute),
		LastBlockHeight: 11,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", int64(10)).Return(conflictingVals, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", int64(10)).Return(&types.BlockMeta{Header: *trustedHeader})
	blockStore.On("LoadBlockCommit", int64(10)).Return(trustedCommit)

	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)
	pool.SetLogger(log.TestingLogger())

	evList := types.EvidenceList{ev}
	err = pool.CheckEvidence(evList)
	require.NoError(t, err)

	pendingEvs, _ := pool.PendingEvidence(state.ConsensusParams.Evidence.MaxBytes)
	assert.Len(t, pendingEvs, 1)
}

type voteData struct {
	vote1 *types.Vote
	vote2 *types.Vote
	valid bool
}

func TestVerifyDuplicateVoteEvidence(t *testing.T) {
	val := types.NewMockPV()
	val2 := types.NewMockPV()
	valSet := types.NewValidatorSet([]*types.Validator{val.ExtractIntoValidator(1)})

	blockID := makeBlockID([]byte("blockhash"), 1000, []byte("partshash"))
	blockID2 := makeBlockID([]byte("blockhash2"), 1000, []byte("partshash"))
	blockID3 := makeBlockID([]byte("blockhash"), 10000, []byte("partshash"))
	blockID4 := makeBlockID([]byte("blockhash"), 10000, []byte("partshash2"))

	const chainID = "mychain"

	vote1 := types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID, defaultEvidenceTime)

	v1 := vote1.ToProto()
	err := val.SignVote(chainID, v1, false)
	require.NoError(t, err)
	badVote := types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID, defaultEvidenceTime)
	bv := badVote.ToProto()
	err = val2.SignVote(chainID, bv, false)
	require.NoError(t, err)

	vote1.Signature = v1.Signature
	badVote.Signature = bv.Signature

	cases := []voteData{
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID2, defaultEvidenceTime), true}, // different block ids
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID3, defaultEvidenceTime), true},
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID4, defaultEvidenceTime), true},
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID, defaultEvidenceTime), false},     // wrong block id
		{vote1, types.MakeVoteNoError(t, val, "mychain2", 0, 10, 2, 1, blockID2, defaultEvidenceTime), false}, // wrong chain id
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 11, 2, 1, blockID2, defaultEvidenceTime), false},    // wrong height
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 3, 1, blockID2, defaultEvidenceTime), false},    // wrong round
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 2, blockID2, defaultEvidenceTime), false},    // wrong step
		{vote1, types.MakeVoteNoError(t, val2, chainID, 0, 10, 2, 1, blockID2, defaultEvidenceTime), false},   // wrong validator
		// a different vote time doesn't matter
		{vote1, types.MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID2, time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)), true},
		{vote1, badVote, false}, // signed by wrong key
	}

	require.NoError(t, err)
	for _, c := range cases {
		ev := &types.DuplicateVoteEvidence{
			VoteA:            c.vote1,
			VoteB:            c.vote2,
			ValidatorPower:   1,
			TotalVotingPower: 1,
			Timestamp:        defaultEvidenceTime,
		}
		if c.valid {
			require.NoError(t, evidence.VerifyDuplicateVote(ev, chainID, valSet), "evidence should be valid")
		} else {
			require.Error(t, evidence.VerifyDuplicateVote(ev, chainID, valSet), "evidence should be invalid")
		}
	}

	// create good evidence and correct validator power
	goodEv, err := types.NewMockDuplicateVoteEvidenceWithValidator(10, defaultEvidenceTime, val, chainID)
	require.NoError(t, err)
	goodEv.ValidatorPower = 1
	goodEv.TotalVotingPower = 1
	badEv, err := types.NewMockDuplicateVoteEvidenceWithValidator(10, defaultEvidenceTime, val, chainID)
	require.NoError(t, err)
	badTimeEv, err := types.NewMockDuplicateVoteEvidenceWithValidator(10, defaultEvidenceTime.Add(1*time.Minute), val, chainID)
	require.NoError(t, err)
	badTimeEv.ValidatorPower = 1
	badTimeEv.TotalVotingPower = 1
	state := sm.State{
		ChainID:         chainID,
		LastBlockTime:   defaultEvidenceTime.Add(1 * time.Minute),
		LastBlockHeight: 11,
		ConsensusParams: *types.DefaultConsensusParams(),
	}
	stateStore := &smmocks.Store{}
	stateStore.On("LoadValidators", int64(10)).Return(valSet, nil)
	stateStore.On("Load").Return(state, nil)
	blockStore := &mocks.BlockStore{}
	blockStore.On("LoadBlockMeta", int64(10)).Return(&types.BlockMeta{Header: types.Header{Time: defaultEvidenceTime}})

	pool, err := evidence.NewPool(dbm.NewMemDB(), stateStore, blockStore)
	require.NoError(t, err)

	evList := types.EvidenceList{goodEv}
	err = pool.CheckEvidence(evList)
	require.NoError(t, err)

	// evidence with a different validator power should fail
	evList = types.EvidenceList{badEv}
	err = pool.CheckEvidence(evList)
	require.Error(t, err)

	// evidence with a different timestamp should fail
	evList = types.EvidenceList{badTimeEv}
	err = pool.CheckEvidence(evList)
	require.Error(t, err)
}

func makeLunaticEvidence(
	t *testing.T,
	height, commonHeight int64,
	byzVals, phantomVals int,
	commonTime, attackTime time.Time,
) (ev *types.LightClientAttackEvidence, trusted *types.LightBlock, common *types.LightBlock) {
	t.Helper()
	totalVals := 10
	commonValSet, commonPrivVals := types.RandValidatorSet(totalVals, defaultVotingPower)

	require.Greater(t, totalVals, byzVals)

	// extract out the subset of byzantine validators in the common validator set
	byzValSet, byzPrivVals := commonValSet.Validators[:byzVals], commonPrivVals[:byzVals]

	phantomValSet, phantomPrivVals := types.RandValidatorSet(phantomVals, defaultVotingPower)

	conflictingVals := phantomValSet.Copy()
	require.NoError(t, conflictingVals.UpdateWithChangeSet(byzValSet))
	conflictingPrivVals := append(phantomPrivVals, byzPrivVals...)

	conflictingPrivVals = orderPrivValsByValSet(t, conflictingVals, conflictingPrivVals)

	commonHeader := makeHeaderRandom(commonHeight)
	commonHeader.Time = commonTime
	trustedHeader := makeHeaderRandom(height)

	conflictingHeader := makeHeaderRandom(height)
	conflictingHeader.Time = attackTime
	conflictingHeader.ValidatorsHash = conflictingVals.Hash()

	blockID := makeBlockID(conflictingHeader.Hash(), 1000, []byte("partshash"))
	voteSet := types.NewVoteSet(evidenceChainID, height, 1, types.SignedMsgType(2), conflictingVals)
	commit, err := test.MakeCommitFromVoteSet(blockID, voteSet, conflictingPrivVals, defaultEvidenceTime)
	require.NoError(t, err)
	ev = &types.LightClientAttackEvidence{
		ConflictingBlock: &types.LightBlock{
			SignedHeader: &types.SignedHeader{
				Header: conflictingHeader,
				Commit: commit,
			},
			ValidatorSet: conflictingVals,
		},
		CommonHeight:        commonHeight,
		TotalVotingPower:    commonValSet.TotalVotingPower(),
		ByzantineValidators: byzValSet,
		Timestamp:           commonTime,
	}

	common = &types.LightBlock{
		SignedHeader: &types.SignedHeader{
			Header: commonHeader,
			// we can leave this empty because we shouldn't be checking this
			Commit: &types.Commit{},
		},
		ValidatorSet: commonValSet,
	}
	trustedBlockID := makeBlockID(trustedHeader.Hash(), 1000, []byte("partshash"))
	trustedVals, privVals := types.RandValidatorSet(totalVals, defaultVotingPower)
	trustedVoteSet := types.NewVoteSet(evidenceChainID, height, 1, types.SignedMsgType(2), trustedVals)
	trustedCommit, err := test.MakeCommitFromVoteSet(trustedBlockID, trustedVoteSet, privVals, defaultEvidenceTime)
	require.NoError(t, err)
	trusted = &types.LightBlock{
		SignedHeader: &types.SignedHeader{
			Header: trustedHeader,
			Commit: trustedCommit,
		},
		ValidatorSet: trustedVals,
	}
	return ev, trusted, common
}

// func makeEquivocationEvidence() *types.LightClientAttackEvidence {

// }

// func makeAmnesiaEvidence() *types.LightClientAttackEvidence {

// }

func makeHeaderRandom(height int64) *types.Header {
	return &types.Header{
		Version:            cmtversion.Consensus{Block: version.BlockProtocol, App: 1},
		ChainID:            evidenceChainID,
		Height:             height,
		Time:               defaultEvidenceTime,
		LastBlockID:        makeBlockID([]byte("headerhash"), 1000, []byte("partshash")),
		LastCommitHash:     crypto.CRandBytes(tmhash.Size),
		DataHash:           crypto.CRandBytes(tmhash.Size),
		ValidatorsHash:     crypto.CRandBytes(tmhash.Size),
		NextValidatorsHash: crypto.CRandBytes(tmhash.Size),
		ConsensusHash:      crypto.CRandBytes(tmhash.Size),
		AppHash:            crypto.CRandBytes(tmhash.Size),
		LastResultsHash:    crypto.CRandBytes(tmhash.Size),
		EvidenceHash:       crypto.CRandBytes(tmhash.Size),
		ProposerAddress:    crypto.CRandBytes(crypto.AddressSize),
	}
}

func makeBlockID(hash []byte, partSetSize uint32, partSetHash []byte) types.BlockID {
	var (
		h   = make([]byte, tmhash.Size)
		psH = make([]byte, tmhash.Size)
	)
	copy(h, hash)
	copy(psH, partSetHash)
	return types.BlockID{
		Hash: h,
		PartSetHeader: types.PartSetHeader{
			Total: partSetSize,
			Hash:  psH,
		},
	}
}

func orderPrivValsByValSet(
	t *testing.T, vals *types.ValidatorSet, privVals []types.PrivValidator,
) []types.PrivValidator {
	t.Helper()
	output := make([]types.PrivValidator, len(privVals))
	for idx, v := range vals.Validators {
		for _, p := range privVals {
			pubKey, err := p.GetPubKey()
			require.NoError(t, err)
			if bytes.Equal(v.Address, pubKey.Address()) {
				output[idx] = p
				break
			}
		}
		require.NotEmpty(t, output[idx])
	}
	return output
}
