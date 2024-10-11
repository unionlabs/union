package types

import (
	"crypto/rand"
	"encoding/hex"
	"math"
	"os"
	"reflect"
	"testing"
	"time"

	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/internal/bits"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/bytes"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

func TestMain(m *testing.M) {
	code := m.Run()
	os.Exit(code)
}

func TestBlockAddEvidence(t *testing.T) {
	txs := []Tx{Tx("foo"), Tx("bar")}
	lastID := makeBlockIDRandom()
	h := int64(3)

	voteSet, _, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, false)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), false)
	require.NoError(t, err)

	ev, err := NewMockDuplicateVoteEvidenceWithValidator(h, cmttime.Now(), vals[0], "block-test-chain")
	require.NoError(t, err)
	evList := []Evidence{ev}

	block := MakeBlock(h, txs, extCommit.ToCommit(), evList)
	require.NotNil(t, block)
	require.Len(t, block.Evidence.Evidence, 1)
	require.NotNil(t, block.EvidenceHash)
}

func TestBlockValidateBasic(t *testing.T) {
	require.Error(t, (*Block)(nil).ValidateBasic())

	txs := []Tx{Tx("foo"), Tx("bar")}
	lastID := makeBlockIDRandom()
	h := int64(3)

	voteSet, valSet, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, false)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), false)
	require.NoError(t, err)
	commit := extCommit.ToCommit()

	ev, err := NewMockDuplicateVoteEvidenceWithValidator(h, cmttime.Now(), vals[0], "block-test-chain")
	require.NoError(t, err)
	evList := []Evidence{ev}

	testCases := []struct {
		testName      string
		malleateBlock func(*Block)
		expErr        bool
	}{
		{"Make Block", func(_ *Block) {}, false},
		{"Make Block w/ proposer Addr", func(blk *Block) { blk.ProposerAddress = valSet.GetProposer().Address }, false},
		{"Negative Height", func(blk *Block) { blk.Height = -1 }, true},
		{"Remove 1/2 the commits", func(blk *Block) {
			blk.LastCommit.Signatures = commit.Signatures[:commit.Size()/2]
			blk.LastCommit.hash = nil // clear hash or change won't be noticed
		}, true},
		{"Remove LastCommitHash", func(blk *Block) { blk.LastCommitHash = []byte("something else") }, true},
		{"Tampered Data", func(blk *Block) {
			blk.Data.Txs[0] = Tx("something else")
			blk.Data.hash = nil // clear hash or change won't be noticed
		}, true},
		{"Tampered DataHash", func(blk *Block) {
			blk.DataHash = cmtrand.Bytes(len(blk.DataHash))
		}, true},
		{"Tampered EvidenceHash", func(blk *Block) {
			blk.EvidenceHash = []byte("something else")
		}, true},
		{"Incorrect block protocol version", func(blk *Block) {
			blk.Version.Block = 1
		}, true},
	}
	for i, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			block := MakeBlock(h, txs, commit, evList)
			block.ProposerAddress = valSet.GetProposer().Address
			tc.malleateBlock(block)
			err = block.ValidateBasic()
			assert.Equal(t, tc.expErr, err != nil, "#%d: %v", i, err)
		})
	}
}

func TestBlockHash(t *testing.T) {
	assert.Nil(t, (*Block)(nil).Hash())
	assert.Nil(t, MakeBlock(int64(3), []Tx{Tx("Hello World")}, nil, nil).Hash())
}

func TestBlockMakePartSet(t *testing.T) {
	bps, err := (*Block)(nil).MakePartSet(2)
	require.Error(t, err)
	assert.Nil(t, bps)

	partSet, err := MakeBlock(int64(3), []Tx{Tx("Hello World")}, nil, nil).MakePartSet(1024)
	require.NoError(t, err)
	assert.NotNil(t, partSet)
	assert.EqualValues(t, 1, partSet.Total())
}

func TestBlockMakePartSetWithEvidence(t *testing.T) {
	bps, err := (*Block)(nil).MakePartSet(2)
	require.Error(t, err)
	assert.Nil(t, bps)

	lastID := makeBlockIDRandom()
	h := int64(3)

	voteSet, _, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, false)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), false)
	require.NoError(t, err)

	ev, err := NewMockDuplicateVoteEvidenceWithValidator(h, cmttime.Now(), vals[0], "block-test-chain")
	require.NoError(t, err)
	evList := []Evidence{ev}

	partSet, err := MakeBlock(h, []Tx{Tx("Hello World")}, extCommit.ToCommit(), evList).MakePartSet(512)
	require.NoError(t, err)

	assert.NotNil(t, partSet)
	assert.EqualValues(t, 4, partSet.Total())
}

func TestBlockHashesTo(t *testing.T) {
	assert.False(t, (*Block)(nil).HashesTo(nil))

	lastID := makeBlockIDRandom()
	h := int64(3)
	voteSet, valSet, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, false)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), false)
	require.NoError(t, err)

	ev, err := NewMockDuplicateVoteEvidenceWithValidator(h, cmttime.Now(), vals[0], "block-test-chain")
	require.NoError(t, err)
	evList := []Evidence{ev}

	block := MakeBlock(h, []Tx{Tx("Hello World")}, extCommit.ToCommit(), evList)
	block.ValidatorsHash = valSet.Hash()
	assert.False(t, block.HashesTo([]byte{}))
	assert.False(t, block.HashesTo([]byte("something else")))
	assert.True(t, block.HashesTo(block.Hash()))
}

func TestBlockSize(t *testing.T) {
	size := MakeBlock(int64(3), []Tx{Tx("Hello World")}, nil, nil).Size()
	if size <= 0 {
		t.Fatal("Size of the block is zero or negative")
	}
}

func TestBlockString(t *testing.T) {
	assert.Equal(t, "nil-Block", (*Block)(nil).String())
	assert.Equal(t, "nil-Block", (*Block)(nil).StringIndented(""))
	assert.Equal(t, "nil-Block", (*Block)(nil).StringShort())

	block := MakeBlock(int64(3), []Tx{Tx("Hello World")}, nil, nil)
	assert.NotEqual(t, "nil-Block", block.String())
	assert.NotEqual(t, "nil-Block", block.StringIndented(""))
	assert.NotEqual(t, "nil-Block", block.StringShort())
}

func makeBlockIDRandom() BlockID {
	var (
		blockHash   = make([]byte, tmhash.Size)
		partSetHash = make([]byte, tmhash.Size)
	)
	rand.Read(blockHash)   //nolint: errcheck // ignore errcheck for read
	rand.Read(partSetHash) //nolint: errcheck // ignore errcheck for read
	return BlockID{blockHash, PartSetHeader{123, partSetHash}}
}

func makeBlockID(hash []byte, partSetSize uint32, partSetHash []byte) BlockID {
	var (
		h   = make([]byte, tmhash.Size)
		psH = make([]byte, tmhash.Size)
	)
	copy(h, hash)
	copy(psH, partSetHash)
	return BlockID{
		Hash: h,
		PartSetHeader: PartSetHeader{
			Total: partSetSize,
			Hash:  psH,
		},
	}
}

var nilBytes []byte

// This follows RFC-6962, i.e. `echo -n ” | sha256sum`.
var emptyBytes = []byte{
	0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8,
	0x99, 0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
	0x78, 0x52, 0xb8, 0x55,
}

func TestNilHeaderHashDoesntCrash(t *testing.T) {
	assert.Equal(t, nilBytes, []byte((*Header)(nil).Hash()))
	assert.Equal(t, nilBytes, []byte((new(Header)).Hash()))
}

func TestNilDataHashDoesntCrash(t *testing.T) {
	assert.Equal(t, emptyBytes, []byte((*Data)(nil).Hash()))
	assert.Equal(t, emptyBytes, []byte(new(Data).Hash()))
}

func TestCommit(t *testing.T) {
	lastID := makeBlockIDRandom()
	h := int64(3)
	voteSet, _, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, true)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), true)
	require.NoError(t, err)

	assert.Equal(t, h-1, extCommit.Height)
	assert.EqualValues(t, 1, extCommit.Round)
	assert.Equal(t, PrecommitType, SignedMsgType(extCommit.Type()))
	if extCommit.Size() <= 0 {
		t.Fatalf("commit %v has a zero or negative size: %d", extCommit, extCommit.Size())
	}

	require.NotNil(t, extCommit.BitArray())
	assert.Equal(t, bits.NewBitArray(10).Size(), extCommit.BitArray().Size())

	assert.Equal(t, voteSet.GetByIndex(0), extCommit.GetByIndex(0))
	assert.True(t, extCommit.IsCommit())
}

func TestCommitValidateBasic(t *testing.T) {
	testCases := []struct {
		testName       string
		malleateCommit func(*Commit)
		expectErr      bool
	}{
		{"Random Commit", func(_ *Commit) {}, false},
		{"Incorrect signature", func(com *Commit) { com.Signatures[0].Signature = []byte{0} }, false},
		{"Incorrect height", func(com *Commit) { com.Height = int64(-100) }, true},
		{"Incorrect round", func(com *Commit) { com.Round = -100 }, true},
	}
	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			com := randCommit(cmttime.Now())
			tc.malleateCommit(com)
			assert.Equal(t, tc.expectErr, com.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestMaxCommitBytes(t *testing.T) {
	// time is varint encoded so need to pick the max.
	// year int, month Month, day, hour, min, sec, nsec int, loc *Location
	timestamp := time.Date(math.MaxInt64, 0, 0, 0, 0, 0, math.MaxInt64, time.UTC)

	cs := CommitSig{
		BlockIDFlag:      BlockIDFlagNil,
		ValidatorAddress: crypto.AddressHash([]byte("validator_address")),
		Timestamp:        timestamp,
		Signature:        crypto.CRandBytes(MaxSignatureSize),
	}

	pbSig := cs.ToProto()
	// test that a single commit sig doesn't exceed max commit sig bytes
	assert.EqualValues(t, MaxCommitSigBytes, pbSig.Size())

	// check size with a single commit
	commit := &Commit{
		Height: math.MaxInt64,
		Round:  math.MaxInt32,
		BlockID: BlockID{
			Hash: tmhash.Sum([]byte("blockID_hash")),
			PartSetHeader: PartSetHeader{
				Total: math.MaxInt32,
				Hash:  tmhash.Sum([]byte("blockID_part_set_header_hash")),
			},
		},
		Signatures: []CommitSig{cs},
	}

	pb := commit.ToProto()

	assert.EqualValues(t, MaxCommitBytes(1), int64(pb.Size()))

	// check the upper bound of the commit size
	for i := 1; i < MaxVotesCount; i++ {
		commit.Signatures = append(commit.Signatures, cs)
	}

	pb = commit.ToProto()

	assert.EqualValues(t, MaxCommitBytes(MaxVotesCount), int64(pb.Size()))
}

func TestHeaderHash(t *testing.T) {
	testCases := []struct {
		desc       string
		header     *Header
		expectHash bytes.HexBytes
	}{
		{"Generates expected hash", &Header{
			Version:            cmtversion.Consensus{Block: 1, App: 2},
			ChainID:            "chainId",
			Height:             3,
			Time:               time.Date(2019, 10, 13, 16, 14, 44, 0, time.UTC),
			LastBlockID:        makeBlockID(make([]byte, tmhash.Size), 6, make([]byte, tmhash.Size)),
			LastCommitHash:     tmhash.Sum([]byte("last_commit_hash")),
			DataHash:           tmhash.Sum([]byte("data_hash")),
			ValidatorsHash:     tmhash.Sum([]byte("validators_hash")),
			NextValidatorsHash: tmhash.Sum([]byte("next_validators_hash")),
			ConsensusHash:      tmhash.Sum([]byte("consensus_hash")),
			AppHash:            tmhash.Sum([]byte("app_hash")),
			LastResultsHash:    tmhash.Sum([]byte("last_results_hash")),
			EvidenceHash:       tmhash.Sum([]byte("evidence_hash")),
			ProposerAddress:    crypto.AddressHash([]byte("proposer_address")),
		}, hexBytesFromString("F740121F553B5418C3EFBD343C2DBFE9E007BB67B0D020A0741374BAB65242A4")},
		{"nil header yields nil", nil, nil},
		{"nil ValidatorsHash yields nil", &Header{
			Version:            cmtversion.Consensus{Block: 1, App: 2},
			ChainID:            "chainId",
			Height:             3,
			Time:               time.Date(2019, 10, 13, 16, 14, 44, 0, time.UTC),
			LastBlockID:        makeBlockID(make([]byte, tmhash.Size), 6, make([]byte, tmhash.Size)),
			LastCommitHash:     tmhash.Sum([]byte("last_commit_hash")),
			DataHash:           tmhash.Sum([]byte("data_hash")),
			ValidatorsHash:     nil,
			NextValidatorsHash: tmhash.Sum([]byte("next_validators_hash")),
			ConsensusHash:      tmhash.Sum([]byte("consensus_hash")),
			AppHash:            tmhash.Sum([]byte("app_hash")),
			LastResultsHash:    tmhash.Sum([]byte("last_results_hash")),
			EvidenceHash:       tmhash.Sum([]byte("evidence_hash")),
			ProposerAddress:    crypto.AddressHash([]byte("proposer_address")),
		}, nil},
	}
	for _, tc := range testCases {
		t.Run(tc.desc, func(t *testing.T) {
			assert.Equal(t, tc.expectHash, tc.header.Hash())

			// We also make sure that all fields are hashed in struct order, and that all
			// fields in the test struct are non-zero.
			if tc.header != nil && tc.expectHash != nil {
				byteSlices := [][]byte{}

				s := reflect.ValueOf(*tc.header)
				for i := 0; i < s.NumField(); i++ {
					f := s.Field(i)

					assert.False(t, f.IsZero(), "Found zero-valued field %v",
						s.Type().Field(i).Name)

					switch f := f.Interface().(type) {
					case int64, bytes.HexBytes, string:
						byteSlices = append(byteSlices, cdcEncode(f))
					case time.Time:
						bz, err := gogotypes.StdTimeMarshal(f)
						require.NoError(t, err)
						byteSlices = append(byteSlices, bz)
					case cmtversion.Consensus:
						bz, err := f.Marshal()
						require.NoError(t, err)
						byteSlices = append(byteSlices, bz)
					case BlockID:
						pbbi := f.ToProto()
						bz, err := pbbi.Marshal()
						require.NoError(t, err)
						byteSlices = append(byteSlices, bz)
					default:
						t.Errorf("unknown type %T", f)
					}
				}
				assert.Equal(t,
					bytes.HexBytes(merkle.HashFromByteSlices(byteSlices)), tc.header.Hash())
			}
		})
	}
}

func TestMaxHeaderBytes(t *testing.T) {
	// Construct a UTF-8 string of MaxChainIDLen length using the supplementary
	// characters.
	// Each supplementary character takes 4 bytes.
	// http://www.i18nguy.com/unicode/supplementary-test.html
	maxChainID := ""
	for i := 0; i < MaxChainIDLen; i++ {
		maxChainID += "𠜎"
	}

	// time is varint encoded so need to pick the max.
	// year int, month Month, day, hour, min, sec, nsec int, loc *Location
	timestamp := time.Date(math.MaxInt64, 0, 0, 0, 0, 0, math.MaxInt64, time.UTC)

	h := Header{
		Version:            cmtversion.Consensus{Block: math.MaxInt64, App: math.MaxInt64},
		ChainID:            maxChainID,
		Height:             math.MaxInt64,
		Time:               timestamp,
		LastBlockID:        makeBlockID(make([]byte, tmhash.Size), math.MaxInt32, make([]byte, tmhash.Size)),
		LastCommitHash:     tmhash.Sum([]byte("last_commit_hash")),
		DataHash:           tmhash.Sum([]byte("data_hash")),
		ValidatorsHash:     tmhash.Sum([]byte("validators_hash")),
		NextValidatorsHash: tmhash.Sum([]byte("next_validators_hash")),
		ConsensusHash:      tmhash.Sum([]byte("consensus_hash")),
		AppHash:            tmhash.Sum([]byte("app_hash")),
		LastResultsHash:    tmhash.Sum([]byte("last_results_hash")),
		EvidenceHash:       tmhash.Sum([]byte("evidence_hash")),
		ProposerAddress:    crypto.AddressHash([]byte("proposer_address")),
	}

	bz, err := h.ToProto().Marshal()
	require.NoError(t, err)

	assert.EqualValues(t, MaxHeaderBytes, int64(len(bz)))
}

func randCommit(now time.Time) *Commit {
	lastID := makeBlockIDRandom()
	h := int64(3)
	voteSet, _, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, false)
	extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, now, false)
	if err != nil {
		panic(err)
	}
	return extCommit.ToCommit()
}

func hexBytesFromString(s string) bytes.HexBytes {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return bytes.HexBytes(b)
}

func TestBlockMaxDataBytes(t *testing.T) {
	testCases := []struct {
		maxBytes      int64
		valsCount     int
		evidenceBytes int64
		panics        bool
		result        int64
	}{
		0: {-10, 1, 0, true, 0},
		1: {10, 1, 0, true, 0},
		2: {841, 1, 0, true, 0},
		3: {875, 1, 0, false, 0},
		4: {876, 1, 0, false, 1},
		5: {1019, 2, 0, false, 0},
	}

	for i, tc := range testCases {
		if tc.panics {
			assert.Panics(t, func() {
				MaxDataBytes(tc.maxBytes, tc.evidenceBytes, tc.valsCount)
			}, "#%v", i)
		} else {
			assert.Equal(t,
				tc.result,
				MaxDataBytes(tc.maxBytes, tc.evidenceBytes, tc.valsCount),
				"#%v", i)
		}
	}
}

func TestBlockMaxDataBytesNoEvidence(t *testing.T) {
	testCases := []struct {
		maxBytes  int64
		valsCount int
		panics    bool
		result    int64
	}{
		0: {-10, 1, true, 0},
		1: {10, 1, true, 0},
		2: {841, 1, true, 0},
		3: {875, 1, false, 0},
		4: {876, 1, false, 1},
	}

	for i, tc := range testCases {
		if tc.panics {
			assert.Panics(t, func() {
				MaxDataBytesNoEvidence(tc.maxBytes, tc.valsCount)
			}, "#%v", i)
		} else {
			assert.Equal(t,
				tc.result,
				MaxDataBytesNoEvidence(tc.maxBytes, tc.valsCount),
				"#%v", i)
		}
	}
}

// TestVoteSetToExtendedCommit tests that the extended commit produced from a
// vote set contains the same vote information as the vote set. The test ensures
// that the MakeExtendedCommit method behaves as expected, whether vote extensions
// are present in the original votes or not.
func TestVoteSetToExtendedCommit(t *testing.T) {
	for _, testCase := range []struct {
		name             string
		includeExtension bool
	}{
		{
			name:             "no extensions",
			includeExtension: false,
		},
		{
			name:             "with extensions",
			includeExtension: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			blockID := makeBlockIDRandom()

			valSet, vals := RandValidatorSet(10, 1)
			var voteSet *VoteSet
			if testCase.includeExtension {
				voteSet = NewExtendedVoteSet("test_chain_id", 3, 1, PrecommitType, valSet)
			} else {
				voteSet = NewVoteSet("test_chain_id", 3, 1, PrecommitType, valSet)
			}
			for i := 0; i < len(vals); i++ {
				pubKey, err := vals[i].GetPubKey()
				require.NoError(t, err)
				vote := &Vote{
					ValidatorAddress: pubKey.Address(),
					ValidatorIndex:   int32(i),
					Height:           3,
					Round:            1,
					Type:             PrecommitType,
					BlockID:          blockID,
					Timestamp:        cmttime.Now(),
				}
				v := vote.ToProto()
				err = vals[i].SignVote(voteSet.ChainID(), v, true)
				require.NoError(t, err)
				vote.Signature = v.Signature
				if testCase.includeExtension {
					vote.ExtensionSignature = v.ExtensionSignature
				}
				added, err := voteSet.AddVote(vote)
				require.NoError(t, err)
				require.True(t, added)
			}
			p := DefaultFeatureParams()
			if testCase.includeExtension {
				p.VoteExtensionsEnableHeight = 1
			}
			ec := voteSet.MakeExtendedCommit(p)

			for i := int32(0); int(i) < len(vals); i++ {
				vote1 := voteSet.GetByIndex(i)
				vote2 := ec.GetExtendedVote(i)

				vote1bz, err := vote1.ToProto().Marshal()
				require.NoError(t, err)
				vote2bz, err := vote2.ToProto().Marshal()
				require.NoError(t, err)
				assert.Equal(t, vote1bz, vote2bz)
			}
		})
	}
}

// toVoteSet constructs a VoteSet from the Commit and validator set.
// Panics if signatures from the ExtendedCommit can't be added to the voteset.
// Inverse of VoteSet.MakeExtendedCommit().
func toVoteSet(ec *ExtendedCommit, chainID string, vals *ValidatorSet) *VoteSet {
	voteSet := NewVoteSet(chainID, ec.Height, ec.Round, PrecommitType, vals)
	ec.addSigsToVoteSet(voteSet)
	return voteSet
}

// TestExtendedCommitToVoteSet tests that the vote set produced from an extended commit
// contains the same vote information as the extended commit. The test ensures
// that the ToVoteSet method behaves as expected, whether vote extensions
// are present in the original votes or not.
func TestExtendedCommitToVoteSet(t *testing.T) {
	for _, testCase := range []struct {
		name             string
		includeExtension bool
	}{
		{
			name:             "no extensions",
			includeExtension: false,
		},
		{
			name:             "with extensions",
			includeExtension: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			lastID := makeBlockIDRandom()
			h := int64(3)

			voteSet, valSet, vals := randVoteSet(h-1, 1, PrecommitType, 10, 1, true)
			extCommit, err := MakeExtCommit(lastID, h-1, 1, voteSet, vals, cmttime.Now(), true)
			require.NoError(t, err)

			if !testCase.includeExtension {
				for i := 0; i < len(vals); i++ {
					v := voteSet.GetByIndex(int32(i))
					v.Extension = nil
					v.ExtensionSignature = nil
					extCommit.ExtendedSignatures[i].Extension = nil
					extCommit.ExtendedSignatures[i].ExtensionSignature = nil
				}
			}

			chainID := voteSet.ChainID()
			var voteSet2 *VoteSet
			if testCase.includeExtension {
				voteSet2 = extCommit.ToExtendedVoteSet(chainID, valSet)
			} else {
				voteSet2 = toVoteSet(extCommit, chainID, valSet)
			}

			for i := int32(0); int(i) < len(vals); i++ {
				vote1 := voteSet.GetByIndex(i)
				vote2 := voteSet2.GetByIndex(i)
				vote3 := extCommit.GetExtendedVote(i)

				vote1bz, err := vote1.ToProto().Marshal()
				require.NoError(t, err)
				vote2bz, err := vote2.ToProto().Marshal()
				require.NoError(t, err)
				vote3bz, err := vote3.ToProto().Marshal()
				require.NoError(t, err)
				assert.Equal(t, vote1bz, vote2bz)
				assert.Equal(t, vote1bz, vote3bz)
			}
		})
	}
}

func TestCommitToVoteSetWithVotesForNilBlock(t *testing.T) {
	blockID := makeBlockID([]byte("blockhash"), 1000, []byte("partshash"))

	const (
		height = int64(3)
		round  = 0
	)

	type commitVoteTest struct {
		blockIDs      []BlockID
		numVotes      []int // must sum to numValidators
		numValidators int
		valid         bool
	}

	testCases := []commitVoteTest{
		{[]BlockID{blockID, {}}, []int{67, 33}, 100, true},
	}

	for _, tc := range testCases {
		voteSet, valSet, vals := randVoteSet(height-1, round, PrecommitType, tc.numValidators, 1, false)

		vi := int32(0)
		for n := range tc.blockIDs {
			for i := 0; i < tc.numVotes[n]; i++ {
				pubKey, err := vals[vi].GetPubKey()
				require.NoError(t, err)
				vote := &Vote{
					ValidatorAddress: pubKey.Address(),
					ValidatorIndex:   vi,
					Height:           height - 1,
					Round:            round,
					Type:             PrecommitType,
					BlockID:          tc.blockIDs[n],
					Timestamp:        cmttime.Now(),
				}

				added, err := signAddVote(vals[vi], vote, voteSet)
				require.NoError(t, err)
				assert.True(t, added)

				vi++
			}
		}

		veHeightParam := DefaultFeatureParams()
		veHeightParam.VoteExtensionsEnableHeight = 0
		if tc.valid {
			extCommit := voteSet.MakeExtendedCommit(veHeightParam) // panics without > 2/3 valid votes
			assert.NotNil(t, extCommit)
			err := valSet.VerifyCommit(voteSet.ChainID(), blockID, height-1, extCommit.ToCommit())
			require.NoError(t, err)
		} else {
			assert.Panics(t, func() { voteSet.MakeExtendedCommit(veHeightParam) })
		}
	}
}

func TestBlockIDValidateBasic(t *testing.T) {
	validBlockID := BlockID{
		Hash: bytes.HexBytes{},
		PartSetHeader: PartSetHeader{
			Total: 1,
			Hash:  bytes.HexBytes{},
		},
	}

	invalidBlockID := BlockID{
		Hash: []byte{0},
		PartSetHeader: PartSetHeader{
			Total: 1,
			Hash:  []byte{0},
		},
	}

	testCases := []struct {
		testName             string
		blockIDHash          bytes.HexBytes
		blockIDPartSetHeader PartSetHeader
		expectErr            bool
	}{
		{"Valid BlockID", validBlockID.Hash, validBlockID.PartSetHeader, false},
		{"Invalid BlockID", invalidBlockID.Hash, validBlockID.PartSetHeader, true},
		{"Invalid BlockID", validBlockID.Hash, invalidBlockID.PartSetHeader, true},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			blockID := BlockID{
				Hash:          tc.blockIDHash,
				PartSetHeader: tc.blockIDPartSetHeader,
			}
			assert.Equal(t, tc.expectErr, blockID.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestBlockProtoBuf(t *testing.T) {
	h := cmtrand.Int63()
	c1 := randCommit(cmttime.Now())
	b1 := MakeBlock(h, []Tx{Tx([]byte{1})}, &Commit{Signatures: []CommitSig{}}, []Evidence{})
	b1.ProposerAddress = cmtrand.Bytes(crypto.AddressSize)

	b2 := MakeBlock(h, []Tx{Tx([]byte{1})}, c1, []Evidence{})
	b2.ProposerAddress = cmtrand.Bytes(crypto.AddressSize)
	evidenceTime := time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)
	evi, err := NewMockDuplicateVoteEvidence(h, evidenceTime, "block-test-chain")
	require.NoError(t, err)
	b2.Evidence = EvidenceData{Evidence: EvidenceList{evi}}
	b2.EvidenceHash = b2.Evidence.Hash()

	b3 := MakeBlock(h, []Tx{}, c1, []Evidence{})
	b3.ProposerAddress = cmtrand.Bytes(crypto.AddressSize)
	testCases := []struct {
		msg      string
		b1       *Block
		expPass  bool
		expPass2 bool
	}{
		{"nil block", nil, false, false},
		{"b1", b1, true, true},
		{"b2", b2, true, true},
		{"b3", b3, true, true},
	}
	for _, tc := range testCases {
		pb, err := tc.b1.ToProto()
		if tc.expPass {
			require.NoError(t, err, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}

		block, err := BlockFromProto(pb)
		if tc.expPass2 {
			require.NoError(t, err, tc.msg)
			require.EqualValues(t, tc.b1.Header, block.Header, tc.msg)
			require.EqualValues(t, tc.b1.Data, block.Data, tc.msg)
			require.EqualValues(t, tc.b1.Evidence.Evidence, block.Evidence.Evidence, tc.msg)
			require.EqualValues(t, *tc.b1.LastCommit, *block.LastCommit, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func TestDataProtoBuf(t *testing.T) {
	data := &Data{Txs: Txs{Tx([]byte{1}), Tx([]byte{2}), Tx([]byte{3})}}
	data2 := &Data{Txs: Txs{}}
	testCases := []struct {
		msg     string
		data1   *Data
		expPass bool
	}{
		{"success", data, true},
		{"success data2", data2, true},
	}
	for _, tc := range testCases {
		protoData := tc.data1.ToProto()
		d, err := DataFromProto(&protoData)
		if tc.expPass {
			require.NoError(t, err, tc.msg)
			require.EqualValues(t, tc.data1, &d, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

// TestEvidenceDataProtoBuf ensures parity in converting to and from proto.
func TestEvidenceDataProtoBuf(t *testing.T) {
	const chainID = "mychain"
	ev, err := NewMockDuplicateVoteEvidence(math.MaxInt64, cmttime.Now(), chainID)
	require.NoError(t, err)
	data := &EvidenceData{Evidence: EvidenceList{ev}}
	_ = data.ByteSize()
	testCases := []struct {
		msg      string
		data1    *EvidenceData
		expPass1 bool
		expPass2 bool
	}{
		{"success", data, true, true},
		{"empty evidenceData", &EvidenceData{Evidence: EvidenceList{}}, true, true},
		{"fail nil Data", nil, false, false},
	}

	for _, tc := range testCases {
		protoData, err := tc.data1.ToProto()
		if tc.expPass1 {
			require.NoError(t, err, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}

		eviD := new(EvidenceData)
		err = eviD.FromProto(protoData)
		if tc.expPass2 {
			require.NoError(t, err, tc.msg)
			require.Equal(t, tc.data1, eviD, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func makeRandHeader() Header {
	chainID := "test"
	t := cmttime.Now()
	height := cmtrand.Int63()
	randBytes := cmtrand.Bytes(tmhash.Size)
	randAddress := cmtrand.Bytes(crypto.AddressSize)
	h := Header{
		Version:            cmtversion.Consensus{Block: version.BlockProtocol, App: 1},
		ChainID:            chainID,
		Height:             height,
		Time:               t,
		LastBlockID:        BlockID{},
		LastCommitHash:     randBytes,
		DataHash:           randBytes,
		ValidatorsHash:     randBytes,
		NextValidatorsHash: randBytes,
		ConsensusHash:      randBytes,
		AppHash:            randBytes,

		LastResultsHash: randBytes,

		EvidenceHash:    randBytes,
		ProposerAddress: randAddress,
	}

	return h
}

func TestHeaderProto(t *testing.T) {
	h1 := makeRandHeader()
	tc := []struct {
		msg     string
		h1      *Header
		expPass bool
	}{
		{"success", &h1, true},
		{"failure empty Header", &Header{}, false},
	}

	for _, tt := range tc {
		t.Run(tt.msg, func(t *testing.T) {
			pb := tt.h1.ToProto()
			h, err := HeaderFromProto(pb)
			if tt.expPass {
				require.NoError(t, err, tt.msg)
				require.Equal(t, tt.h1, &h, tt.msg)
			} else {
				require.Error(t, err, tt.msg)
			}
		})
	}
}

func TestBlockIDProtoBuf(t *testing.T) {
	blockID := makeBlockID([]byte("hash"), 2, []byte("part_set_hash"))
	testCases := []struct {
		msg     string
		bid1    *BlockID
		expPass bool
	}{
		{"success", &blockID, true},
		{"success empty", &BlockID{}, true},
		{"failure BlockID nil", nil, false},
	}
	for _, tc := range testCases {
		protoBlockID := tc.bid1.ToProto()

		bi, err := BlockIDFromProto(&protoBlockID)
		if tc.expPass {
			require.NoError(t, err)
			require.Equal(t, tc.bid1, bi, tc.msg)
		} else {
			require.NotEqual(t, tc.bid1, bi, tc.msg)
		}
	}
}

func TestSignedHeaderProtoBuf(t *testing.T) {
	commit := randCommit(cmttime.Now())
	h := makeRandHeader()

	sh := SignedHeader{Header: &h, Commit: commit}

	testCases := []struct {
		msg     string
		sh1     *SignedHeader
		expPass bool
	}{
		{"empty SignedHeader 2", &SignedHeader{}, true},
		{"success", &sh, true},
		{"failure nil", nil, false},
	}
	for _, tc := range testCases {
		protoSignedHeader := tc.sh1.ToProto()

		sh, err := SignedHeaderFromProto(protoSignedHeader)

		if tc.expPass {
			require.NoError(t, err, tc.msg)
			require.Equal(t, tc.sh1, sh, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func TestBlockIDEquals(t *testing.T) {
	var (
		blockID          = makeBlockID([]byte("hash"), 2, []byte("part_set_hash"))
		blockIDDuplicate = makeBlockID([]byte("hash"), 2, []byte("part_set_hash"))
		blockIDDifferent = makeBlockID([]byte("different_hash"), 2, []byte("part_set_hash"))
		blockIDEmpty     = BlockID{}
	)

	assert.True(t, blockID.Equals(blockIDDuplicate))
	assert.False(t, blockID.Equals(blockIDDifferent))
	assert.False(t, blockID.Equals(blockIDEmpty))
	assert.True(t, blockIDEmpty.Equals(blockIDEmpty))
	assert.False(t, blockIDEmpty.Equals(blockIDDifferent))
}
