package nonadjacent

import (
	"encoding/hex"
	"fmt"
	"galois/pkg/lightclient"
	"math/big"
	"math/rand"

	"cosmossdk.io/math"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/assert"

	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"

	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"

	"testing"

	"github.com/consensys/gnark/test"
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

func FuzzNonadjacent(f *testing.F) {
	marshalValidators := func(validators []*types.SimpleValidator) ([lightclient.MaxVal]lightclient.Validator, []byte, error) {
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

	aggregateSignatures := func(signatures [][]byte) (curve.G2Affine, error) {
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
	toValidator := func(pubKey []byte, power int64) (*types.SimpleValidator, error) {
		protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
		if err != nil {
			return &types.SimpleValidator{}, err
		}
		return &types.SimpleValidator{
			PubKey:      &protoPK,
			VotingPower: sdk.TokensToConsensusPower(math.NewInt(power), sdk.DefaultPowerReduction),
		}, nil
	}

	f.Fuzz(func(t *testing.T, seed int64) {
		t.Parallel()
		r := rand.New(rand.NewSource(seed))

		nbOfValidators := 1 + r.Uint32()%lightclient.MaxVal

		blockHash := make([]byte, 32)
		_, err := r.Read(blockHash)
		if err != nil {
			t.Fatal(err)
		}

		partSetHeaderHash := make([]byte, 32)
		_, err = r.Read(partSetHeaderHash)
		if err != nil {
			t.Fatal(err)
		}

		vote := types.CanonicalVote{
			Type:   types.PrecommitType,
			Height: r.Int63(),
			Round:  r.Int63(),
			BlockID: &types.CanonicalBlockID{
				Hash: blockHash,
				PartSetHeader: types.CanonicalPartSetHeader{
					Total: 1,
					Hash:  partSetHeaderHash,
				},
			},
			ChainID: fmt.Sprintf("union-devnet-%d", r.Uint64()),
		}

		privKeys := make([]cometbn254.PrivKey, nbOfValidators)
		validators := make([]*types.SimpleValidator, nbOfValidators)
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

		signedBytes, err := protoio.MarshalDelimited(&vote)
		if err != nil {
			t.Fatal(err)
		}

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

		trustedValidators := validators
		untrustedValidators := validators

		trustedSignatures := signatures
		untrustedSignatures := signatures

		trustedBitmap := bitmap
		untrustedBitmap := bitmap

		trustedValidatorsInput, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
		if err != nil {
			t.Fatal(err)
		}

		trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
		if err != nil {
			t.Fatal(err)
		}

		untrustedValidatorsInput, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
		if err != nil {
			t.Fatal(err)
		}

		untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
		if err != nil {
			t.Fatal(err)
		}

		if !trustedAggregatedSignature.IsInSubGroup() || !trustedAggregatedSignature.IsOnCurve() || trustedAggregatedSignature.IsInfinity() {
			panic("")
		}

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

		message := cometbn254.HashToField(signedBytes)

		hashedMessage := cometbn254.HashToG2(signedBytes)

		circuit := Circuit{
			DomainSeparationTag:      []byte(cometbn254.CometblsSigDST),
			TrustedInput:             trustedInput,
			UntrustedInput:           untrustedInput,
			ExpectedTrustedValRoot:   trustedValidatorsRoot,
			ExpectedUntrustedValRoot: untrustedValidatorsRoot,
			Message:                  message,
			HashedMessage:            gadget.NewG2Affine(hashedMessage),
		}

		err = test.IsSolved(
			&Circuit{},
			&circuit,
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}
