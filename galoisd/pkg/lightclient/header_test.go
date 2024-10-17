package lightclient

import (
	"crypto/sha256"
	"fmt"
	g2 "galois/pkg/emulated"
	"math/big"
	"math/rand"
	"testing"
	"time"

	tmtypes "github.com/cometbft/cometbft/api/cometbft/types/v1"
	version "github.com/cometbft/cometbft/api/cometbft/version/v1"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cometbft/cometbft/types"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
)

func getBlockHeader(r *rand.Rand) (*BlockHeader, *BlockVote, *types.Header, *tmtypes.Vote) {
	trunc := func(b []byte) UnconsHash {
		return UnconsHash{
			Head: b[0],
			Tail: b[1:],
		}
	}

	readHash := func() []byte {
		var hash [32]byte
		r.Read(hash[:])
		return hash[:]
	}

	partSetHeaderTotal := r.Uint32()
	partSetHeaderHash := readHash()
	round := r.Int31()
	versionBlock := r.Uint64()
	versionApp := r.Uint64()
	chainId := fmt.Sprintf("union-devnet-%d", r.Uint64()%65535)
	height := r.Int63()
	time := time.Unix(r.Int63(), r.Int63())
	lastBlockHash := readHash()[:1]
	lastBlockPartSetHeaderTotal := r.Uint32()
	lastBlockPartSetHeaderHash := readHash()
	lastCommitHash := readHash()
	dataHash := readHash()
	validatorsHash := readHash()[1:]
	nextValidatorsHash := readHash()[1:]
	consensusHash := readHash()
	appHash := readHash()
	lastResultsHash := readHash()
	evidenceHash := readHash()
	proposerAddress := readHash()

	header := &BlockHeader{
		VersionBlock:                versionBlock,
		VersionApp:                  versionApp,
		ChainID:                     []byte(chainId),
		Height:                      height,
		TimeSecs:                    time.Unix(),
		TimeNanos:                   time.Nanosecond(),
		LastBlockHash:               lastBlockHash,
		LastBlockPartSetHeaderTotal: lastBlockPartSetHeaderTotal,
		LastBlockPartSetHeaderHash:  trunc(lastBlockPartSetHeaderHash),
		LastCommitHash:              trunc(lastCommitHash),
		DataHash:                    trunc(dataHash),
		ValidatorsHash:              validatorsHash,
		NextValidatorsHash:          nextValidatorsHash,
		ConsensusHash:               trunc(consensusHash),
		AppHash:                     trunc(appHash),
		LastResultsHash:             trunc(lastResultsHash),
		EvidenceHash:                trunc(evidenceHash),
		ProposerAddress:             trunc(proposerAddress),
	}

	vote := &BlockVote{
		BlockPartSetHeaderTotal: partSetHeaderTotal,
		BlockPartSetHeaderHash:  trunc(partSetHeaderHash),
		Round:                   round,
	}

	cometblsHeader := &types.Header{
		Version: version.Consensus{
			Block: versionBlock,
			App:   versionApp,
		},
		ChainID: chainId,
		Height:  height,
		Time:    time,
		LastBlockID: types.BlockID{
			Hash: lastBlockHash,
			PartSetHeader: types.PartSetHeader{
				Total: lastBlockPartSetHeaderTotal,
				Hash:  lastBlockPartSetHeaderHash,
			},
		},
		LastCommitHash:     lastCommitHash,
		DataHash:           dataHash,
		ValidatorsHash:     validatorsHash,
		NextValidatorsHash: nextValidatorsHash,
		ConsensusHash:      consensusHash,
		AppHash:            appHash,
		LastResultsHash:    lastResultsHash,
		EvidenceHash:       evidenceHash,
		ProposerAddress:    proposerAddress,
	}

	cometblsVote := &tmtypes.Vote{
		Type:   tmtypes.PrecommitType,
		Height: cometblsHeader.Height,
		Round:  round,
		BlockID: tmtypes.BlockID{
			Hash: cometblsHeader.Hash(),
			PartSetHeader: tmtypes.PartSetHeader{
				Total: partSetHeaderTotal,
				Hash:  partSetHeaderHash,
			},
		},
	}

	return header, vote, cometblsHeader, cometblsVote
}

type VerifyInputs struct {
	Vote       BlockVote
	Header     BlockHeader
	InputsHash frontend.Variable
}

func (c *VerifyInputs) Define(api frontend.API) error {
	bhapi, err := NewBlockHeaderAPI(api, c.Header, c.Vote)
	if err != nil {
		return err
	}
	bhapi.VerifyInputs(c.InputsHash, c.Header.ValidatorsHash)
	return nil
}

func inputsHash(h *types.Header) []byte {
	buff := []byte{}
	var padded [32]byte
	writeI64 := func(x int64) {
		big.NewInt(x).FillBytes(padded[:])
		buff = append(buff, padded[:]...)
	}
	writeMiMCHash := func(b []byte) {
		big.NewInt(0).SetBytes(b).FillBytes(padded[:])
		buff = append(buff, padded[:]...)
	}
	writeHash := func(b []byte) {
		buff = append(buff, b...)
	}
	writeMiMCHash([]byte(h.ChainID))
	writeI64(h.Height)
	writeI64(h.Time.Unix())
	writeI64(int64(h.Time.Nanosecond()))
	writeMiMCHash(h.ValidatorsHash)
	writeMiMCHash(h.NextValidatorsHash)
	writeHash(h.AppHash)
	writeMiMCHash(h.ValidatorsHash)
	hash := sha256.Sum256(buff)
	return hash[1:]
}

func FuzzVerifyInputs(f *testing.F) {
	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))
		header, vote, h, _ := getBlockHeader(r)

		err := test.IsSolved(
			&VerifyInputs{},
			&VerifyInputs{
				Vote:       *vote,
				Header:     *header,
				InputsHash: inputsHash(h),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type BlockHash struct {
	Vote         BlockVote
	Header       BlockHeader
	ExpectedHash frontend.Variable
}

func (c *BlockHash) Define(api frontend.API) error {
	bhapi, err := NewBlockHeaderAPI(api, c.Header, c.Vote)
	if err != nil {
		return err
	}
	api.AssertIsEqual(c.ExpectedHash, bhapi.BlockHash())
	return nil
}

func FuzzBlockHash(f *testing.F) {
	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))
		header, vote, cometblsHeader, _ := getBlockHeader(r)
		hash := cometblsHeader.Hash()
		err := test.IsSolved(
			&BlockHash{},
			&BlockHash{
				Vote:         *vote,
				Header:       *header,
				ExpectedHash: []byte(hash),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type BlockVoteSignBytes struct {
	Vote              BlockVote
	Header            BlockHeader
	ExpectedVoteBytes frontend.Variable
}

func (c *BlockVoteSignBytes) Define(api frontend.API) error {
	bhapi, err := NewBlockHeaderAPI(api, c.Header, c.Vote)
	if err != nil {
		return err
	}
	voteSignBytes, err := bhapi.VoteSignBytes()
	if err != nil {
		return err
	}
	api.AssertIsEqual(voteSignBytes, c.ExpectedVoteBytes)
	return nil
}

func FuzzBlockVote(f *testing.F) {
	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))
		header, vote, cometblsHeader, cometblsVote := getBlockHeader(r)
		signBytes := types.VoteSignBytes(cometblsHeader.ChainID, cometblsVote)
		err := test.IsSolved(
			&BlockVoteSignBytes{},
			&BlockVoteSignBytes{
				Vote:              *vote,
				Header:            *header,
				ExpectedVoteBytes: signBytes,
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type BlockToCurve struct {
	DST          frontend.Variable
	Vote         BlockVote
	Header       BlockHeader
	ExpectedHash gadget.G2Affine
}

func (c *BlockToCurve) Define(api frontend.API) error {
	bhapi, err := NewBlockHeaderAPI(api, c.Header, c.Vote)
	if err != nil {
		return err
	}
	hashed, err := bhapi.HashToCurve(c.DST)
	if err != nil {
		return err
	}
	emulatedAPI, err := g2.NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	emulatedAPI.AssertIsEqual(&c.ExpectedHash, hashed)
	return nil
}

func FuzzBlockToCurve(f *testing.F) {
	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))
		header, vote, cometblsHeader, cometblsVote := getBlockHeader(r)
		signBytes := types.VoteSignBytes(cometblsHeader.ChainID, cometblsVote)
		err := test.IsSolved(
			&BlockToCurve{},
			&BlockToCurve{
				Vote:         *vote,
				Header:       *header,
				ExpectedHash: gadget.NewG2Affine(cometbn254.HashToG2(signBytes)),
				DST:          []byte(cometbn254.CometblsSigDST),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}
