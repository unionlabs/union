package nonadjacent

import (
	"cosmossdk.io/math"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"galois/pkg/lightclient"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	tmtypes "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/proto/tendermint/version"
	"github.com/cometbft/cometbft/types"
	comettypes "github.com/cometbft/cometbft/types"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend/witness"
	bn254 "github.com/consensys/gnark/constraint/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/test"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/assert"
	"math/big"
	"math/rand"
	"testing"
	"time"
)

type Pairing struct {
	PK  gadget.G1Affine
	Sig gadget.G2Affine
	Msg gadget.G2Affine
}

func (c *Pairing) Define(api frontend.API) error {
	pairing, err := gadget.NewPairing(api)
	if err != nil {
		return fmt.Errorf("new pairing: %w", err)
	}
	_, _, g1Gen, _ := curve.Generators()
	g1GenNeg := gadget.NewG1Affine(*g1Gen.Neg(&g1Gen))

	err = pairing.PairingCheck(
		[]*gadget.G1Affine{&g1GenNeg, &c.PK},
		[]*gadget.G2Affine{&c.Sig, &c.Msg},
	)
	if err != nil {
		return fmt.Errorf("pairing check: %w", err)
	}
	return nil
}

func TestPairingVirtual(t *testing.T) {
	t.Parallel()

	hex := func(h string) []byte {
		b, err := hex.DecodeString(h)
		assert.NoError(t, err)
		return b
	}

	_, _, g1Gen, _ := curve.Generators()
	var g1GenNeg curve.G1Affine
	g1GenNeg.Neg(&g1Gen)

	var pk curve.G1Affine
	_, err := pk.SetBytes(hex("83D016646DF946E887CD36AE6C10BED9C4A49D675CCC072E5AAF496AA4B2D50D"))
	assert.NoError(t, err)
	var sig curve.G2Affine
	_, err = sig.SetBytes(hex("C4B626B703FACBFA5A5071B8254E4A4B78BB45C6A534FF5822BA806730BCE9522707DBF7D689759DFB5CE8DA3A99E04219DDC1CBB8F94481876B1F24FFA5A73E"))
	assert.NoError(t, err)
	var msg curve.G2Affine
	_, err = msg.SetBytes(hex("8591C93118F5A406886C558BEA365D249D881BBC7FCD68673307CC8350BA9C49000ABE5E44150F1E98196B088D4A3A60AFE27E49BA4D4411E528320022D324CF"))
	assert.NoError(t, err)

	err = test.IsSolved(
		&Pairing{},
		&Pairing{
			PK:  gadget.NewG1Affine(pk),
			Sig: gadget.NewG2Affine(sig),
			Msg: gadget.NewG2Affine(msg),
		},
		ecc.BN254.ScalarField(),
	)
	assert.NoError(t, err)
}

func TestPairingNative(t *testing.T) {
	t.Parallel()

	hex := func(h string) []byte {
		b, err := hex.DecodeString(h)
		assert.NoError(t, err)
		return b
	}

	_, _, g1Gen, _ := curve.Generators()
	var g1GenNeg curve.G1Affine
	g1GenNeg.Neg(&g1Gen)

	var pk curve.G1Affine
	_, err := pk.SetBytes(hex("83D016646DF946E887CD36AE6C10BED9C4A49D675CCC072E5AAF496AA4B2D50D"))
	assert.NoError(t, err)
	var sig curve.G2Affine
	_, err = sig.SetBytes(hex("C4B626B703FACBFA5A5071B8254E4A4B78BB45C6A534FF5822BA806730BCE9522707DBF7D689759DFB5CE8DA3A99E04219DDC1CBB8F94481876B1F24FFA5A73E"))
	assert.NoError(t, err)
	var msg curve.G2Affine
	_, err = msg.SetBytes(hex("8591C93118F5A406886C558BEA365D249D881BBC7FCD68673307CC8350BA9C49000ABE5E44150F1E98196B088D4A3A60AFE27E49BA4D4411E528320022D324CF"))
	assert.NoError(t, err)

	ok, err := curve.PairingCheck(
		[]curve.G1Affine{
			g1GenNeg,
			pk,
		},
		[]curve.G2Affine{
			sig,
			msg,
		})
	assert.NoError(t, err)
	assert.True(t, ok)
}

func inputsHash(h *comettypes.Header) []byte {
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

func marshalValidators(validators []*tmtypes.SimpleValidator) ([lightclient.MaxVal]lightclient.Validator, []byte, error) {
	lcValidators := [lightclient.MaxVal]lightclient.Validator{}
	// Make sure we zero initialize
	for i := 0; i < lightclient.MaxVal; i++ {
		lcValidators[i].HashableX = 0
		lcValidators[i].HashableXMSB = 0
		lcValidators[i].HashableY = 0
		lcValidators[i].HashableYMSB = 0
		lcValidators[i].Power = 0
	}
	merkleTree := make([][]byte, len(validators))
	for i, val := range validators {
		tmPK, err := ce.PubKeyFromProto(*val.PubKey)
		if err != nil {
			return lcValidators, nil, fmt.Errorf("Could not deserialize proto to tendermint public key %s", err)
		}
		var public curve.G1Affine
		_, err = public.SetBytes(tmPK.Bytes())
		if err != nil {
			return lcValidators, nil, fmt.Errorf("Could not deserialize bn254 public key %s", err)
		}
		leaf, err := cometbn254.NewMerkleLeaf(public, val.VotingPower)
		if err != nil {
			return lcValidators, nil, fmt.Errorf("Could not create merkle leaf %s", err)
		}
		lcValidators[i].HashableX = leaf.ShiftedX
		lcValidators[i].HashableY = leaf.ShiftedY
		lcValidators[i].HashableXMSB = leaf.MsbX
		lcValidators[i].HashableYMSB = leaf.MsbY
		lcValidators[i].Power = leaf.VotingPower
		merkleTree[i], err = leaf.Hash()
		if err != nil {
			return lcValidators, nil, fmt.Errorf("Could not create merkle hash %s", err)
		}
	}
	return lcValidators, merkle.MimcHashFromByteSlices(merkleTree), nil
}

func aggregateSignatures(signatures [][]byte) (curve.G2Affine, error) {
	var aggregatedSignature curve.G2Affine
	var decompressedSignature curve.G2Affine
	for _, signature := range signatures {
		_, err := decompressedSignature.SetBytes(signature)
		if err != nil {
			return curve.G2Affine{}, fmt.Errorf("Could not decompress signature %s", err)
		}
		aggregatedSignature.Add(&aggregatedSignature, &decompressedSignature)
	}
	return aggregatedSignature, nil
}

// Nb of tokens for each val in devnet
func toValidator(pubKey []byte, power int64) (*tmtypes.SimpleValidator, error) {
	protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
	if err != nil {
		return &tmtypes.SimpleValidator{}, err
	}
	return &tmtypes.SimpleValidator{
		PubKey:      &protoPK,
		VotingPower: sdk.TokensToConsensusPower(math.NewInt(power), sdk.DefaultPowerReduction),
	}, nil
}

func getBlockHeader(r *rand.Rand, validatorsHash []byte, nextValidatorsHash []byte) (*lightclient.BlockHeader, *lightclient.BlockVote, *types.Header, *tmtypes.Vote) {
	trunc := func(b []byte) lightclient.UnconsHash {
		return lightclient.UnconsHash{
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
	consensusHash := readHash()
	appHash := readHash()
	lastResultsHash := readHash()
	evidenceHash := readHash()
	proposerAddress := readHash()

	header := &lightclient.BlockHeader{
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

	vote := &lightclient.BlockVote{
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

func FuzzNonadjacent(f *testing.F) {
	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))

		nbOfValidators := 1 + r.Uint32()%lightclient.MaxVal

		privKeys := make([]cometbn254.PrivKey, nbOfValidators)
		validators := make([]*tmtypes.SimpleValidator, nbOfValidators)
		totalPower := int64(0)
		for i := 0; i < len(validators); i++ {
			privKeys[i] = cometbn254.GenPrivKey()
			val, err := toValidator(privKeys[i].PubKey().Bytes(), 100000000+r.Int63n(100000000))
			if err != nil {
				t.Fatal(err)
			}
			totalPower += val.VotingPower
			validators[i] = val
		}

		trustedValidators := validators
		untrustedValidators := validators

		trustedValidatorsInput, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
		if err != nil {
			t.Fatal(err)
		}

		untrustedValidatorsInput, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
		if err != nil {
			t.Fatal(err)
		}

		header, vote, cometblsHeader, cometblsVote := getBlockHeader(r, trustedValidatorsRoot, untrustedValidatorsRoot)

		signedBytes := comettypes.VoteSignBytes(cometblsHeader.ChainID, cometblsVote)

		var signatures [][]byte
		var bitmap big.Int
		votingPower := 0

		for true {
			if votingPower > int(totalPower)/3*2+1 {
				break
			}
			index := uint32(rand.Int31n(int32(nbOfValidators) - 1))
			i := index
			for bitmap.Bit(int(i)) == 1 {
				i = (i + 1) % nbOfValidators
			}
			votingPower += int(validators[i].VotingPower)
			bitmap.SetBit(&bitmap, int(i), 1)
			sig, err := privKeys[i].Sign(signedBytes)
			if err != nil {
				t.Fatal(err)
			}
			signatures = append(signatures, sig)
		}

		trustedSignatures := signatures
		untrustedSignatures := signatures

		trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
		if err != nil {
			t.Fatal(err)
		}

		untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
		if err != nil {
			t.Fatal(err)
		}

		trustedBitmap := bitmap
		untrustedBitmap := bitmap

		trustedInput := TendermintNonAdjacentLightClientInput{
			Sig:           gadget.NewG2Affine(trustedAggregatedSignature),
			Validators:    trustedValidatorsInput,
			NbOfVal:       nbOfValidators,
			NbOfSignature: len(trustedSignatures),
			Bitmap:        trustedBitmap,
		}

		untrustedInput := TendermintNonAdjacentLightClientInput{
			Sig:           gadget.NewG2Affine(untrustedAggregatedSignature),
			Validators:    untrustedValidatorsInput,
			NbOfVal:       nbOfValidators,
			NbOfSignature: len(untrustedSignatures),
			Bitmap:        untrustedBitmap,
		}

		circuit := Circuit{
			DomainSeparationTag: []byte(cometbn254.CometblsSigDST),
			TrustedInput:        trustedInput,
			TrustedValRoot:      trustedValidatorsRoot,
			UntrustedInput:      untrustedInput,
			Vote:                *vote,
			Header:              *header,
			InputsHash:          inputsHash(cometblsHeader),
		}

		err = test.IsSolved(
			&Circuit{},
			&circuit,
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type privateInputs struct {
	// DomainSeparationTag *frontend.Variable
	TrustedInput   *TendermintNonAdjacentLightClientInput
	UntrustedInput *TendermintNonAdjacentLightClientInput
}

// We try to generate a valid proof without any of the signatures from the
// validator set, by using a padding validator that is not in the set.
func AttackSelectPaddedValidator(t *testing.T, privIn privateInputs, totalPower int64, signedBytes []byte) {
	// 0. Remove all signatures
	privIn.TrustedInput.Sig = gadget.G2Affine{}

	// 1. Create a fake validator credential with maximum power
	fakePrivKey := cometbn254.GenPrivKey()
	fakeVal, err := toValidator(fakePrivKey.PubKey().Bytes(), 0)
	fakeVal.VotingPower = totalPower * 10
	if err != nil {
		t.Fatal(err)
	}

	// 2. Sign the block with the fake validator
	sig, err := fakePrivKey.Sign(signedBytes)
	if err != nil {
		t.Fatal(err)
	}
	fakeAggSignature, err := aggregateSignatures([][]byte{sig})
	if err != nil {
		t.Fatal(err)
	}

	// 3. Set the fake validator in an unused slot of the validators list,
	// and update the private inputs to only use that validator outside of
	// the range of nbOfValidators.  We don't touch the public inputs.
	fakeValidatorsInput, _, err := marshalValidators([]*tmtypes.SimpleValidator{fakeVal})
	if err != nil {
		t.Fatal(err)
	}
	var fakeBitmap big.Int
	fakeBitmap.SetBit(&fakeBitmap, 4, 1)

	privIn.TrustedInput.Validators[4] = fakeValidatorsInput[0]
	privIn.TrustedInput.Bitmap = fakeBitmap
	privIn.TrustedInput.NbOfSignature = 1
	privIn.TrustedInput.Sig = gadget.NewG2Affine(fakeAggSignature)

	privIn.UntrustedInput.Validators[4] = fakeValidatorsInput[0]
	privIn.UntrustedInput.Bitmap = fakeBitmap
	privIn.UntrustedInput.NbOfSignature = 1
	privIn.UntrustedInput.Sig = gadget.NewG2Affine(fakeAggSignature)
}

// We try to generate a valid proof without any of the signatures, by
// setting the voting power of a padding validator to a value that overflows
// the totalVotingPower to 0, so that no signatures are required to prove
// consensus.
func AttackSelectPaddedPower(t *testing.T, privIn privateInputs, totalPower int64, signedBytes []byte) {
	// 0. Remove all signatures, leaving an aggregation of "0 signatures"
	hm := cometbn254.HashToG2(signedBytes)
	privIn.TrustedInput.Sig = gadget.NewG2Affine(hm)
	privIn.UntrustedInput.Sig = gadget.NewG2Affine(hm)

	// 1. Use an empty bitmap, set 0 signatures
	var fakeBitmap big.Int
	privIn.TrustedInput.Bitmap = fakeBitmap
	privIn.TrustedInput.NbOfSignature = 0
	privIn.UntrustedInput.Bitmap = fakeBitmap
	privIn.UntrustedInput.NbOfSignature = 0

	// 2. Set the power of a padding validator to `fr.Modulus - totalPower`
	var negTotalPower big.Int
	negTotalPower.Sub(fr.Modulus(), big.NewInt(totalPower))
	privIn.TrustedInput.Validators[4].Power = negTotalPower
	privIn.UntrustedInput.Validators[4].Power = negTotalPower
}

func AttackFailing(t *testing.T, attack func(t *testing.T, privIn privateInputs, totalPower int64, signedBytes []byte)) {
	r := rand.New(rand.NewSource(0))

	nbOfValidators := uint32(4)

	privKeys := make([]cometbn254.PrivKey, nbOfValidators)
	validators := make([]*tmtypes.SimpleValidator, nbOfValidators)
	totalPower := int64(0)
	for i := 0; i < len(validators); i++ {
		privKeys[i] = cometbn254.GenPrivKey()
		val, err := toValidator(privKeys[i].PubKey().Bytes(), 100000000+r.Int63n(100000000))
		if err != nil {
			t.Fatal(err)
		}
		totalPower += val.VotingPower
		validators[i] = val
	}

	trustedValidators := validators
	untrustedValidators := validators

	trustedValidatorsInput, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	untrustedValidatorsInput, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	header, vote, cometblsHeader, cometblsVote := getBlockHeader(r, trustedValidatorsRoot, untrustedValidatorsRoot)

	signedBytes := comettypes.VoteSignBytes(cometblsHeader.ChainID, cometblsVote)

	var signatures [][]byte
	var bitmap big.Int
	votingPower := 0

	for true {
		if votingPower > int(totalPower)/3*2+1 {
			break
		}
		index := uint32(rand.Int31n(int32(nbOfValidators) - 1))
		i := index
		for bitmap.Bit(int(i)) == 1 {
			i = (i + 1) % nbOfValidators
		}
		votingPower += int(validators[i].VotingPower)
		bitmap.SetBit(&bitmap, int(i), 1)
		sig, err := privKeys[i].Sign(signedBytes)
		if err != nil {
			t.Fatal(err)
		}
		signatures = append(signatures, sig)
	}

	trustedSignatures := signatures
	untrustedSignatures := signatures

	trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	trustedBitmap := bitmap
	untrustedBitmap := bitmap

	trustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(trustedAggregatedSignature),
		Validators:    trustedValidatorsInput,
		NbOfVal:       nbOfValidators,
		NbOfSignature: len(trustedSignatures),
		Bitmap:        trustedBitmap,
	}

	untrustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(untrustedAggregatedSignature),
		Validators:    untrustedValidatorsInput,
		NbOfVal:       nbOfValidators,
		NbOfSignature: len(untrustedSignatures),
		Bitmap:        untrustedBitmap,
	}

	attack(t, privateInputs{
		TrustedInput:   &trustedInput,
		UntrustedInput: &untrustedInput,
	},
		totalPower,
		signedBytes,
	)

	circuit := Circuit{
		DomainSeparationTag: []byte(cometbn254.CometblsSigDST),
		TrustedInput:        trustedInput,
		TrustedValRoot:      trustedValidatorsRoot,
		UntrustedInput:      untrustedInput,
		Vote:                *vote,
		Header:              *header,
		InputsHash:          inputsHash(cometblsHeader),
	}

	err = test.IsSolved(
		&Circuit{},
		&circuit,
		ecc.BN254.ScalarField(),
	)
	assert.Error(t, err)
}

func TestCantSelectPaddedValidator(t *testing.T) {
	AttackFailing(t, AttackSelectPaddedValidator)
}

func TestCantSelectPaddedPower(t *testing.T) {
	AttackFailing(t, AttackSelectPaddedPower)
}

func buildCircuit(t *testing.T) (*bn254.R1CS, *witness.Witness, error) {
	r := rand.New(rand.NewSource(0xCAFEBABE))

	nbOfValidators := 1 + r.Uint32()%lightclient.MaxVal

	privKeys := make([]cometbn254.PrivKey, nbOfValidators)
	validators := make([]*tmtypes.SimpleValidator, nbOfValidators)
	totalPower := int64(0)
	for i := 0; i < len(validators); i++ {
		privKeys[i] = cometbn254.GenPrivKey()
		val, err := toValidator(privKeys[i].PubKey().Bytes(), 100000000+r.Int63n(100000000))
		if err != nil {
			t.Fatal(err)
		}
		totalPower += val.VotingPower
		validators[i] = val
	}

	trustedValidators := validators
	untrustedValidators := validators

	trustedValidatorsInput, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	untrustedValidatorsInput, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	header, vote, cometblsHeader, cometblsVote := getBlockHeader(r, trustedValidatorsRoot, untrustedValidatorsRoot)

	signedBytes := comettypes.VoteSignBytes(cometblsHeader.ChainID, cometblsVote)

	var signatures [][]byte
	var bitmap big.Int
	votingPower := 0

	for true {
		if votingPower > int(totalPower)/3*2+1 {
			break
		}
		index := uint32(rand.Int31n(int32(nbOfValidators) - 1))
		i := index
		for bitmap.Bit(int(i)) == 1 {
			i = (i + 1) % nbOfValidators
		}
		votingPower += int(validators[i].VotingPower)
		bitmap.SetBit(&bitmap, int(i), 1)
		sig, err := privKeys[i].Sign(signedBytes)
		if err != nil {
			t.Fatal(err)
		}
		signatures = append(signatures, sig)
	}

	trustedSignatures := signatures
	untrustedSignatures := signatures

	trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	trustedBitmap := bitmap
	untrustedBitmap := bitmap

	trustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(trustedAggregatedSignature),
		Validators:    trustedValidatorsInput,
		NbOfVal:       nbOfValidators,
		NbOfSignature: len(trustedSignatures),
		Bitmap:        trustedBitmap,
	}

	untrustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(untrustedAggregatedSignature),
		Validators:    untrustedValidatorsInput,
		NbOfVal:       nbOfValidators,
		NbOfSignature: len(untrustedSignatures),
		Bitmap:        untrustedBitmap,
	}

	circuit := Circuit{
		DomainSeparationTag: []byte(cometbn254.CometblsSigDST),
		TrustedInput:        trustedInput,
		TrustedValRoot:      trustedValidatorsRoot,
		UntrustedInput:      untrustedInput,
		Vote:                *vote,
		Header:              *header,
		InputsHash:          inputsHash(cometblsHeader),
	}

	err = test.IsSolved(
		&Circuit{},
		&circuit,
		ecc.BN254.ScalarField(),
	)
	assert.NoError(t, err)

	witness, err := frontend.NewWitness(&circuit, ecc.BN254.ScalarField())
	if err != nil {
		return nil, nil, err
	}

	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
	if err != nil {
		return nil, nil, err
	}

	return r1cs.(*bn254.R1CS), &witness, err
}

// func TestNonAdjacentSetup(t *testing.T) {
// 	const (
// 		nContributionsPhase1 = 1
// 		nContributionsPhase2 = 1
// 		power                = 22
// 	)

// 	assert := require.New(t)

// 	t.Log("InitPhase1")
// 	start := time.Now()
// 	var srs1 mpc.Phase1
// 	err := readFrom("final.ph1", &srs1)
// 	assert.NoError(err)
// 	t.Logf("InitPhase1: %v", time.Since(start))

// 	t.Log("Building circuit...")
// 	start = time.Now()
// 	r1cs, witness, err := buildCircuit(t)
// 	assert.NoError(err)
// 	t.Logf("Built in: %v", time.Since(start))

// 	t.Log("InitPhase2")
// 	start = time.Now()
// 	var srs2 mpc.Phase2
// 	err = readFrom("init.ph2", &srs2)
// 	assert.NoError(err)
// 	var evals mpc.Phase2Evaluations
// 	err = readFrom("evals.ph2", &evals)
// 	assert.NoError(err)
// 	t.Logf("InitPhase2: %v", time.Since(start))

// 	err = saveTo("init.ph2", &srs2)
// 	assert.NoError(err)
// 	err = saveTo("evals.ph2", &evals)
// 	assert.NoError(err)

// 	err = saveTo("contrib.ph2", &srs2)
// 	assert.NoError(err)

// 	// Make and verify contributions for phase1
// 	for i := 0; i < nContributionsPhase2; i++ {
// 		// we clone for test purposes; but in practice, participant will receive a []byte, deserialize it,
// 		// add his contribution and send back to coordinator.
// 		t.Log("ContributePhase2")
// 		assert.NoError(err)
// 		start = time.Now()
// 		srs2.Contribute()
// 		t.Logf("ContributePhase2: %v", time.Since(start))
// 		t.Log("VerifyPhase2")
// 		var prev mpc.Phase2
// 		start = time.Now()
// 		err = readFrom("contrib.ph2", &prev)
// 		t.Logf("VerifyPhase2Read: %v", time.Since(start))
// 		start = time.Now()
// 		assert.NoError(mpc.VerifyPhase2(&prev, &srs2))
// 		err = saveTo("contrib.ph2", &srs2)
// 		assert.NoError(err)
// 		t.Logf("VerifyPhase2: %v", time.Since(start))
// 	}

// 	t.Log("ExtractKeys")
// 	// Extract the proving and verifying keys
// 	pk, vk := mpc.ExtractKeys(r1cs, &srs1, &srs2, &evals)

// 	t.Log("Save")
// 	err = saveTo("r1cs.bin", r1cs)
// 	assert.NoError(err)
// 	err = saveTo("pk.bin", &pk)
// 	assert.NoError(err)
// 	err = saveTo("vk.bin", &vk)
// 	assert.NoError(err)

// 	t.Log("Prove...")
// 	proof, err := groth16.Prove(r1cs, &pk, *witness)
// 	assert.NoError(err)

// 	pubWitness, err := (*witness).Public()
// 	assert.NoError(err)

// 	t.Log("Verify...")
// 	err = groth16.Verify(proof, &vk, pubWitness)
// 	assert.NoError(err)
// }

// func readFrom(file string, obj io.ReaderFrom) error {
// 	f, err := os.OpenFile(file, os.O_RDONLY, os.ModePerm)
// 	if err != nil {
// 		return err
// 	}
// 	defer f.Close()
// 	obj.ReadFrom(f)
// 	return nil
// }

// func saveTo(file string, x io.WriterTo) error {
// 	f, err := os.Create(file)
// 	if err != nil {
// 		return err
// 	}
// 	defer f.Close()
// 	w := bufio.NewWriter(f)
// 	_, err = x.WriteTo(w)
// 	if err != nil {
// 		return err
// 	}
// 	w.Flush()
// 	return nil
// }

// func clonePhase1(phase1 *mpc.Phase1) mpc.Phase1 {
// 	r := mpc.Phase1{}
// 	r.Parameters.G1.Tau = append(r.Parameters.G1.Tau, phase1.Parameters.G1.Tau...)
// 	r.Parameters.G1.AlphaTau = append(r.Parameters.G1.AlphaTau, phase1.Parameters.G1.AlphaTau...)
// 	r.Parameters.G1.BetaTau = append(r.Parameters.G1.BetaTau, phase1.Parameters.G1.BetaTau...)

// 	r.Parameters.G2.Tau = append(r.Parameters.G2.Tau, phase1.Parameters.G2.Tau...)
// 	r.Parameters.G2.Beta = phase1.Parameters.G2.Beta

// 	r.PublicKeys = phase1.PublicKeys
// 	r.Hash = append(r.Hash, phase1.Hash...)

// 	return r
// }

// func clonePhase2(phase2 *mpc.Phase2) mpc.Phase2 {
// 	r := mpc.Phase2{}
// 	r.Parameters.G1.BasisExpSigma = make([][]curve.G1Affine, len(r.Parameters.G1.BasisExpSigma))
// 	for i := 0; i < len(r.Parameters.G1.BasisExpSigma); i++ {
// 		r.Parameters.G1.BasisExpSigma[i] = append(
// 			r.Parameters.G1.BasisExpSigma[i],
// 			phase2.Parameters.G1.BasisExpSigma[i]...,
// 		)
// 	}
// 	r.Parameters.G1.Delta = phase2.Parameters.G1.Delta
// 	r.Parameters.G1.L = append(r.Parameters.G1.L, phase2.Parameters.G1.L...)
// 	r.Parameters.G1.Z = append(r.Parameters.G1.Z, phase2.Parameters.G1.Z...)
// 	r.Parameters.G2.Delta = phase2.Parameters.G2.Delta
// 	r.Parameters.G2.GRootSigmaNeg = phase2.Parameters.G2.GRootSigmaNeg
// 	r.PublicKey = phase2.PublicKey
// 	r.Hash = append(r.Hash, phase2.Hash...)

// 	return r
// }

// func convertPtauToPhase1(ptau Ptau) (phase1 mpc.Phase1, err error) {
// 	tauG1 := make([]curve.G1Affine, len(ptau.PTauPubKey.TauG1))
// 	for i, g1 := range ptau.PTauPubKey.TauG1 {
// 		g1Affine := curve.G1Affine{}
// 		x := bytesToElement(g1[0].Bytes())
// 		g1Affine.X = x
// 		y := bytesToElement(g1[1].Bytes())
// 		g1Affine.Y = y
// 		if !g1Affine.IsOnCurve() {
// 			fmt.Printf("tauG1: \n index: %v g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
// 			panic("g1Affine is not on curve")
// 		}
// 		tauG1[i] = g1Affine
// 	}

// 	alphaTauG1 := make([]curve.G1Affine, len(ptau.PTauPubKey.AlphaTauG1))
// 	for i, g1 := range ptau.PTauPubKey.AlphaTauG1 {
// 		g1Affine := curve.G1Affine{}
// 		x := bytesToElement(g1[0].Bytes())
// 		g1Affine.X = x
// 		y := bytesToElement(g1[1].Bytes())
// 		g1Affine.Y = y
// 		if !g1Affine.IsOnCurve() {
// 			fmt.Printf("alphaTauG1: \n index: %v g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
// 			panic("g1Affine is not on curve")
// 		}
// 		alphaTauG1[i] = g1Affine
// 	}
// 	// fmt.Printf("alphaTauG1: %v \n", alphaTauG1)

// 	betaTauG1 := make([]curve.G1Affine, len(ptau.PTauPubKey.BetaTauG1))

// 	for i, g1 := range ptau.PTauPubKey.BetaTauG1 {
// 		g1Affine := curve.G1Affine{}
// 		x := bytesToElement(g1[0].Bytes())
// 		g1Affine.X = x
// 		y := bytesToElement(g1[1].Bytes())
// 		g1Affine.Y = y
// 		if !g1Affine.IsOnCurve() {
// 			fmt.Printf("betaTauG1: \n index: %v, g1Affine.X: %v \n g1Affine.Y: %v \n", i, g1Affine.X.String(), g1Affine.Y.String())
// 			panic("g1Affine is not on curve")
// 		}
// 		betaTauG1[i] = g1Affine
// 	}
// 	tauG2 := make([]curve.G2Affine, len(ptau.PTauPubKey.TauG2))
// 	for i, g2 := range ptau.PTauPubKey.TauG2 {
// 		g2Affine := curve.G2Affine{}
// 		x0 := bytesToElement(g2[0].Bytes())
// 		x1 := bytesToElement(g2[1].Bytes())
// 		g2Affine.X.A0 = x0
// 		g2Affine.X.A1 = x1
// 		y0 := bytesToElement(g2[2].Bytes())
// 		y1 := bytesToElement(g2[3].Bytes())
// 		g2Affine.Y.A0 = y0
// 		g2Affine.Y.A1 = y1
// 		if !g2Affine.IsOnCurve() {
// 			fmt.Printf("tauG2: \n index: %v, g2Affine.X.A0: %v \n g2Affine.X.A1: %v \n g2Affine.Y.A0: %v \n g2Affine.Y.A1 %v \n", i, g2Affine.X.A0.String(), g2Affine.X.A1.String(), g2Affine.Y.A0.String(), g2Affine.Y.A1.String())
// 			panic("g2Affine is not on curve")
// 		}
// 		tauG2[i] = g2Affine
// 	}

// 	betaG2 := curve.G2Affine{}
// 	{
// 		g2 := ptau.PTauPubKey.BetaG2

// 		x0 := bytesToElement(g2[0].Bytes())
// 		x1 := bytesToElement(g2[1].Bytes())
// 		betaG2.X.A0 = x0
// 		betaG2.X.A1 = x1
// 		y0 := bytesToElement(g2[2].Bytes())
// 		y1 := bytesToElement(g2[3].Bytes())
// 		betaG2.Y.A0 = y0
// 		betaG2.Y.A1 = y1

// 		if !betaG2.IsOnCurve() {
// 			fmt.Printf("g2Affine.X.A0: %v \n g2Affine.X.A1: %v \n g2Affine.Y.A0: %v \n g2Affine.Y.A1 %v \n", betaG2.X.A0.String(), betaG2.X.String(), betaG2.Y.A0.String(), betaG2.Y.A1.String())
// 			panic("g2Affine is not on curve")
// 		}
// 	}

// 	phase1.Parameters.G1.Tau = tauG1
// 	phase1.Parameters.G1.AlphaTau = alphaTauG1
// 	phase1.Parameters.G1.BetaTau = betaTauG1

// 	phase1.Parameters.G2.Tau = tauG2
// 	phase1.Parameters.G2.Beta = betaG2

// 	return phase1, nil
// }
