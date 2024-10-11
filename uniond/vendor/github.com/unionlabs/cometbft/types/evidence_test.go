package types

import (
	"math"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

var defaultVoteTime = time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)

func TestEvidenceList(t *testing.T) {
	ev := randomDuplicateVoteEvidence(t)
	evl := EvidenceList([]Evidence{ev})

	assert.NotNil(t, evl.Hash())
	assert.True(t, evl.Has(ev))
	assert.False(t, evl.Has(&DuplicateVoteEvidence{}))
}

func randomDuplicateVoteEvidence(t *testing.T) *DuplicateVoteEvidence {
	t.Helper()
	val := NewMockPV()
	blockID := makeBlockID([]byte("blockhash"), 1000, []byte("partshash"))
	blockID2 := makeBlockID([]byte("blockhash2"), 1000, []byte("partshash"))
	const chainID = "mychain"
	return &DuplicateVoteEvidence{
		VoteA:            MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID, defaultVoteTime),
		VoteB:            MakeVoteNoError(t, val, chainID, 0, 10, 2, 1, blockID2, defaultVoteTime.Add(1*time.Minute)),
		TotalVotingPower: 30,
		ValidatorPower:   10,
		Timestamp:        defaultVoteTime,
	}
}

func TestDuplicateVoteEvidence(t *testing.T) {
	const height = int64(13)
	ev, err := NewMockDuplicateVoteEvidence(height, cmttime.Now(), "mock-chain-id")
	require.NoError(t, err)
	assert.Equal(t, ev.Hash(), tmhash.Sum(ev.Bytes()))
	assert.NotNil(t, ev.String())
	assert.Equal(t, height, ev.Height())
}

func TestDuplicateVoteEvidenceValidation(t *testing.T) {
	val := NewMockPV()
	blockID := makeBlockID(tmhash.Sum([]byte("blockhash")), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	blockID2 := makeBlockID(tmhash.Sum([]byte("blockhash2")), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	const chainID = "mychain"

	testCases := []struct {
		testName         string
		malleateEvidence func(*DuplicateVoteEvidence)
		expectErr        bool
	}{
		{"Good DuplicateVoteEvidence", func(_ *DuplicateVoteEvidence) {}, false},
		{"Nil vote A", func(ev *DuplicateVoteEvidence) { ev.VoteA = nil }, true},
		{"Nil vote B", func(ev *DuplicateVoteEvidence) { ev.VoteB = nil }, true},
		{"Nil votes", func(ev *DuplicateVoteEvidence) {
			ev.VoteA = nil
			ev.VoteB = nil
		}, true},
		{"Invalid vote type", func(ev *DuplicateVoteEvidence) {
			ev.VoteA = MakeVoteNoError(t, val, chainID, math.MaxInt32, math.MaxInt64, math.MaxInt32, 0, blockID2, defaultVoteTime)
		}, true},
		{"Invalid vote order", func(ev *DuplicateVoteEvidence) {
			swap := ev.VoteA.Copy()
			ev.VoteA = ev.VoteB.Copy()
			ev.VoteB = swap
		}, true},
	}
	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			vote1 := MakeVoteNoError(t, val, chainID, math.MaxInt32, math.MaxInt64, math.MaxInt32, 0x02, blockID, defaultVoteTime)
			vote2 := MakeVoteNoError(t, val, chainID, math.MaxInt32, math.MaxInt64, math.MaxInt32, 0x02, blockID2, defaultVoteTime)
			valSet := NewValidatorSet([]*Validator{val.ExtractIntoValidator(10)})
			ev, err := NewDuplicateVoteEvidence(vote1, vote2, defaultVoteTime, valSet)
			require.NoError(t, err)
			tc.malleateEvidence(ev)
			assert.Equal(t, tc.expectErr, ev.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestLightClientAttackEvidenceBasic(t *testing.T) {
	height := int64(5)
	commonHeight := height - 1
	nValidators := 10
	voteSet, valSet, privVals := randVoteSet(height, 1, PrecommitType, nValidators, 1, false)
	header := makeHeaderRandom()
	header.Height = height
	blockID := makeBlockID(tmhash.Sum([]byte("blockhash")), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	extCommit, err := MakeExtCommit(blockID, height, 1, voteSet, privVals, defaultVoteTime, false)
	require.NoError(t, err)
	commit := extCommit.ToCommit()

	lcae := &LightClientAttackEvidence{
		ConflictingBlock: &LightBlock{
			SignedHeader: &SignedHeader{
				Header: header,
				Commit: commit,
			},
			ValidatorSet: valSet,
		},
		CommonHeight:        commonHeight,
		TotalVotingPower:    valSet.TotalVotingPower(),
		Timestamp:           header.Time,
		ByzantineValidators: valSet.Validators[:nValidators/2],
	}
	assert.NotNil(t, lcae.String())
	assert.NotNil(t, lcae.Hash())
	assert.Equal(t, lcae.Height(), commonHeight) // Height should be the common Height
	assert.NotNil(t, lcae.Bytes())

	// maleate evidence to test hash uniqueness
	testCases := []struct {
		testName         string
		malleateEvidence func(*LightClientAttackEvidence)
	}{
		{"Different header", func(ev *LightClientAttackEvidence) { ev.ConflictingBlock.Header = makeHeaderRandom() }},
		{"Different common height", func(ev *LightClientAttackEvidence) {
			ev.CommonHeight = height + 1
		}},
	}

	for _, tc := range testCases {
		lcae := &LightClientAttackEvidence{
			ConflictingBlock: &LightBlock{
				SignedHeader: &SignedHeader{
					Header: header,
					Commit: commit,
				},
				ValidatorSet: valSet,
			},
			CommonHeight:        commonHeight,
			TotalVotingPower:    valSet.TotalVotingPower(),
			Timestamp:           header.Time,
			ByzantineValidators: valSet.Validators[:nValidators/2],
		}
		hash := lcae.Hash()
		tc.malleateEvidence(lcae)
		assert.NotEqual(t, hash, lcae.Hash(), tc.testName)
	}
}

func TestLightClientAttackEvidenceValidation(t *testing.T) {
	height := int64(5)
	commonHeight := height - 1
	nValidators := 10
	voteSet, valSet, privVals := randVoteSet(height, 1, PrecommitType, nValidators, 1, false)
	header := makeHeaderRandom()
	header.Height = height
	header.ValidatorsHash = valSet.Hash()
	blockID := makeBlockID(header.Hash(), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	extCommit, err := MakeExtCommit(blockID, height, 1, voteSet, privVals, cmttime.Now(), false)
	require.NoError(t, err)
	commit := extCommit.ToCommit()

	lcae := &LightClientAttackEvidence{
		ConflictingBlock: &LightBlock{
			SignedHeader: &SignedHeader{
				Header: header,
				Commit: commit,
			},
			ValidatorSet: valSet,
		},
		CommonHeight:        commonHeight,
		TotalVotingPower:    valSet.TotalVotingPower(),
		Timestamp:           header.Time,
		ByzantineValidators: valSet.Validators[:nValidators/2],
	}
	require.NoError(t, lcae.ValidateBasic())

	testCases := []struct {
		testName         string
		malleateEvidence func(*LightClientAttackEvidence)
		expectErr        bool
	}{
		{"Good LightClientAttackEvidence", func(_ *LightClientAttackEvidence) {}, false},
		{"Negative height", func(ev *LightClientAttackEvidence) { ev.CommonHeight = -10 }, true},
		{"Height is greater than divergent block", func(ev *LightClientAttackEvidence) {
			ev.CommonHeight = height + 1
		}, true},
		{"Height is equal to the divergent block", func(ev *LightClientAttackEvidence) {
			ev.CommonHeight = height
		}, false},
		{"Nil conflicting header", func(ev *LightClientAttackEvidence) { ev.ConflictingBlock.Header = nil }, true},
		{"Nil conflicting blocl", func(ev *LightClientAttackEvidence) { ev.ConflictingBlock = nil }, true},
		{"Nil validator set", func(ev *LightClientAttackEvidence) {
			ev.ConflictingBlock.ValidatorSet = &ValidatorSet{}
		}, true},
		{"Negative total voting power", func(ev *LightClientAttackEvidence) {
			ev.TotalVotingPower = -1
		}, true},
	}
	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			lcae := &LightClientAttackEvidence{
				ConflictingBlock: &LightBlock{
					SignedHeader: &SignedHeader{
						Header: header,
						Commit: commit,
					},
					ValidatorSet: valSet,
				},
				CommonHeight:        commonHeight,
				TotalVotingPower:    valSet.TotalVotingPower(),
				Timestamp:           header.Time,
				ByzantineValidators: valSet.Validators[:nValidators/2],
			}
			tc.malleateEvidence(lcae)
			if tc.expectErr {
				require.Error(t, lcae.ValidateBasic(), tc.testName)
			} else {
				require.NoError(t, lcae.ValidateBasic(), tc.testName)
			}
		})
	}
}

func TestMockEvidenceValidateBasic(t *testing.T) {
	goodEvidence, err := NewMockDuplicateVoteEvidence(int64(1), cmttime.Now(), "mock-chain-id")
	require.NoError(t, err)
	require.NoError(t, goodEvidence.ValidateBasic())
}

func makeHeaderRandom() *Header {
	return &Header{
		Version:            cmtversion.Consensus{Block: version.BlockProtocol, App: 1},
		ChainID:            cmtrand.Str(12),
		Height:             int64(cmtrand.Uint16()) + 1,
		Time:               cmttime.Now(),
		LastBlockID:        makeBlockIDRandom(),
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

func TestEvidenceProto(t *testing.T) {
	// -------- Votes --------
	val := NewMockPV()
	blockID := makeBlockID(tmhash.Sum([]byte("blockhash")), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	blockID2 := makeBlockID(tmhash.Sum([]byte("blockhash2")), math.MaxInt32, tmhash.Sum([]byte("partshash")))
	const chainID = "mychain"
	v := MakeVoteNoError(t, val, chainID, math.MaxInt32, math.MaxInt64, 1, 0x01, blockID, defaultVoteTime)
	v2 := MakeVoteNoError(t, val, chainID, math.MaxInt32, math.MaxInt64, 2, 0x01, blockID2, defaultVoteTime)

	// -------- SignedHeaders --------
	const height int64 = 37

	var (
		header1 = makeHeaderRandom()
		header2 = makeHeaderRandom()
	)

	header1.Height = height
	header1.LastBlockID = blockID
	header1.ChainID = chainID

	header2.Height = height
	header2.LastBlockID = blockID
	header2.ChainID = chainID

	tests := []struct {
		testName     string
		evidence     Evidence
		toProtoErr   bool
		fromProtoErr bool
	}{
		{"nil fail", nil, true, true},
		{"DuplicateVoteEvidence empty fail", &DuplicateVoteEvidence{}, false, true},
		{"DuplicateVoteEvidence nil voteB", &DuplicateVoteEvidence{VoteA: v, VoteB: nil}, false, true},
		{"DuplicateVoteEvidence nil voteA", &DuplicateVoteEvidence{VoteA: nil, VoteB: v}, false, true},
		{"DuplicateVoteEvidence success", &DuplicateVoteEvidence{VoteA: v2, VoteB: v}, false, false},
	}
	for _, tt := range tests {
		t.Run(tt.testName, func(t *testing.T) {
			pb, err := EvidenceToProto(tt.evidence)
			if tt.toProtoErr {
				require.Error(t, err, tt.testName)
				return
			}
			require.NoError(t, err, tt.testName)

			evi, err := EvidenceFromProto(pb)
			if tt.fromProtoErr {
				require.Error(t, err, tt.testName)
				return
			}
			require.Equal(t, tt.evidence, evi, tt.testName)
		})
	}
}

// Test that the new JSON tags are picked up correctly, see issue #3528.
func TestDuplicateVoteEvidenceJSON(t *testing.T) {
	var evidence DuplicateVoteEvidence
	js, err := cmtjson.Marshal(evidence)
	require.NoError(t, err)

	wantJSON := `{"type":"tendermint/DuplicateVoteEvidence","value":{"vote_a":null,"vote_b":null,"total_voting_power":"0","validator_power":"0","timestamp":"0001-01-01T00:00:00Z"}}`
	assert.Equal(t, wantJSON, string(js))
}

// Test that the new JSON tags are picked up correctly, see issue #3528.
func TestLightClientAttackEvidenceJSON(t *testing.T) {
	var evidence LightClientAttackEvidence
	js, err := cmtjson.Marshal(evidence)
	require.NoError(t, err)

	wantJSON := `{"type":"tendermint/LightClientAttackEvidence","value":{"conflicting_block":null,"common_height":"0","byzantine_validators":null,"total_voting_power":"0","timestamp":"0001-01-01T00:00:00Z"}}`
	assert.Equal(t, wantJSON, string(js))
}
