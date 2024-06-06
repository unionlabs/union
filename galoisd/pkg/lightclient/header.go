package lightclient

import (
	"fmt"
	g2 "github.com/unionlabs/union/galoisd/pkg/emulated"
	"github.com/unionlabs/union/galoisd/pkg/merkle"
	"slices"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/sha2"
	"github.com/consensys/gnark/std/math/uints"

	"github.com/cometbft/cometbft/proto/tendermint/types"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/hash/mimc"
)

type UnconsHash struct {
	Head frontend.Variable
	Tail frontend.Variable
}

type BlockVote struct {
	// ======================================
	// Technically not part of this header, this is the expected part set
	// header for the current header that will become part of the
	// LastBlockPartsetHeader in the next block
	BlockPartSetHeaderTotal frontend.Variable
	BlockPartSetHeaderHash  UnconsHash
	// The commit round, part of the vote structure only
	Round frontend.Variable
}

type BlockHeader struct {
	VersionBlock frontend.Variable
	VersionApp   frontend.Variable

	ChainID frontend.Variable

	Height frontend.Variable

	TimeSecs  frontend.Variable
	TimeNanos frontend.Variable

	LastBlockHash               frontend.Variable
	LastBlockPartSetHeaderTotal frontend.Variable
	LastBlockPartSetHeaderHash  UnconsHash

	LastCommitHash     UnconsHash
	DataHash           UnconsHash
	ValidatorsHash     frontend.Variable
	NextValidatorsHash frontend.Variable
	ConsensusHash      UnconsHash
	AppHash            UnconsHash
	LastResultsHash    UnconsHash
	EvidenceHash       UnconsHash
	ProposerAddress    UnconsHash
}

type BlockHeaderAPI struct {
	vote        BlockVote
	header      BlockHeader
	api         frontend.API
	binaryField *uints.BinaryField[uints.U32]
}

func NewBlockHeaderAPI(api frontend.API, header BlockHeader, vote BlockVote) (*BlockHeaderAPI, error) {
	binaryField, err := uints.New[uints.U32](api)
	if err != nil {
		return nil, err
	}
	return &BlockHeaderAPI{
		api:         api,
		vote:        vote,
		header:      header,
		binaryField: binaryField,
	}, nil
}

func (b *BlockHeaderAPI) unpack(x frontend.Variable) []uints.U8 {
	split := Unpack(b.api, x, 256, 8)
	slices.Reverse(split)
	bytes := make([]uints.U8, 32)
	for i := 0; i < len(bytes); i++ {
		bytes[i] = uints.U8{
			Val: split[i],
		}
	}
	return bytes
}

func (b *BlockHeaderAPI) unpackHead(x frontend.Variable) uints.U8 {
	split := Unpack(b.api, x, 8, 8)
	return uints.U8{
		Val: split[0],
	}
}

func (b *BlockHeaderAPI) unpackTail(x frontend.Variable) []uints.U8 {
	split := Unpack(b.api, x, 248, 8)
	bytes := make([]uints.U8, 31)
	for i := 0; i < len(bytes); i++ {
		bytes[i] = uints.U8{
			Val: split[i],
		}
	}
	return bytes
}

func (b *BlockHeaderAPI) unpackHash(x *UnconsHash) []uints.U8 {
	hash := append(b.unpackTail(x.Tail), b.unpackHead(x.Head))
	slices.Reverse(hash)
	logs := make([]frontend.Variable, len(hash))
	for i := 0; i < len(logs); i++ {
		logs[i] = hash[i].Val
	}
	return hash
}

func (b *BlockHeaderAPI) VerifyInputs(expectedHash frontend.Variable, trustedValRoot frontend.Variable) error {
	expectedHashBytes := b.unpack(expectedHash)
	hash, err := b.InputsHash(trustedValRoot)
	if err != nil {
		return err
	}
	// Truncate most significant byte of the sha256 hash to fit in a single public input
	for i := 1; i < len(hash); i++ {
		b.binaryField.ByteAssertEq(expectedHashBytes[i], hash[i])
	}
	return nil
}

func (b *BlockHeaderAPI) InputsHash(trustedValRoot frontend.Variable) ([]uints.U8, error) {
	h, err := sha2.New(b.api)
	if err != nil {
		return nil, err
	}
	// Header
	h.Write(b.unpack(b.header.ChainID))
	h.Write(b.unpack(b.header.Height))
	h.Write(b.unpack(b.header.TimeSecs))
	h.Write(b.unpack(b.header.TimeNanos))
	h.Write(b.unpack(b.header.ValidatorsHash))
	h.Write(b.unpack(b.header.NextValidatorsHash))
	h.Write(b.unpackHash(&b.header.AppHash))
	// Private extra inputs
	h.Write(b.unpack(trustedValRoot))
	return h.Sum(), nil
}

func (b *BlockHeaderAPI) BlockHash() frontend.Variable {
	m := merkle.NewMerkleTreeAPI(b.api)
	uncons := func(x *UnconsHash) frontend.Variable {
		leaves := []frontend.Variable{
			x.Head,
			x.Tail,
		}
		for i := 0; i < len(leaves); i++ {
			leaves[i] = m.LeafHash([]frontend.Variable{leaves[i]})
		}
		return m.RootHash(leaves, len(leaves))
	}
	leaves := []frontend.Variable{
		b.header.VersionBlock,
		b.header.VersionApp,
		b.header.ChainID,
		b.header.Height,
		b.header.TimeSecs,
		b.header.TimeNanos,
		b.header.LastBlockHash,
		b.header.LastBlockPartSetHeaderTotal,
		uncons(&b.header.LastBlockPartSetHeaderHash),
		uncons(&b.header.LastCommitHash),
		uncons(&b.header.DataHash),
		b.header.ValidatorsHash,
		b.header.NextValidatorsHash,
		uncons(&b.header.ConsensusHash),
		uncons(&b.header.AppHash),
		uncons(&b.header.LastResultsHash),
		uncons(&b.header.EvidenceHash),
		uncons(&b.header.ProposerAddress),
	}
	for i := 0; i < len(leaves); i++ {
		leaves[i] = m.LeafHash([]frontend.Variable{leaves[i]})
	}
	return m.RootHash(leaves, len(leaves))
}

func (b *BlockHeaderAPI) VoteSignBytes() (frontend.Variable, error) {
	h, err := mimc.NewMiMC(b.api)
	if err != nil {
		return nil, fmt.Errorf("new mimc: %w", err)
	}
	// Vote structure
	h.Write(int64(types.PrecommitType))
	h.Write(b.header.Height)
	h.Write(b.vote.Round)
	h.Write(b.BlockHash())
	h.Write(b.vote.BlockPartSetHeaderTotal)
	h.Write(b.vote.BlockPartSetHeaderHash.Head)
	h.Write(b.vote.BlockPartSetHeaderHash.Tail)
	h.Write(b.header.ChainID)
	return h.Sum(), nil
}

func (b *BlockHeaderAPI) HashToCurve(domainSeparationTag frontend.Variable) (*gadget.G2Affine, error) {
	voteSignBytes, err := b.VoteSignBytes()
	if err != nil {
		return nil, err
	}
	emulatedAPI, err := g2.NewEmulatedAPI(b.api)
	if err != nil {
		return nil, err
	}
	return emulatedAPI.HashToG2(voteSignBytes, domainSeparationTag)
}
