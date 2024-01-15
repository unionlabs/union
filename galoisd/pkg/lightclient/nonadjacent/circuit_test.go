package nonadjacent

import (
	"crypto/rand"
	"fmt"
	"galois/pkg/lightclient"
	"log"
	"math/big"

	"cosmossdk.io/math"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark/frontend"
	sdk "github.com/cosmos/cosmos-sdk/types"

	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	back "github.com/consensys/gnark/backend"

	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"

	"testing"

	"github.com/consensys/gnark/test"
)

func Test(t *testing.T) {
	nbOfValidators := lightclient.MaxVal

	// Nb of tokens for each val in devnet
	toValidator := func(pubKey []byte) (*types.SimpleValidator, error) {
		protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
		if err != nil {
			return &types.SimpleValidator{}, err
		}
		power, err := rand.Int(rand.Reader, big.NewInt(922337203685477586))
		if err != nil {
			return &types.SimpleValidator{}, err
		}
		return &types.SimpleValidator{
			PubKey:      &protoPK,
			VotingPower: sdk.TokensToConsensusPower(math.NewInt(power.Int64()), sdk.DefaultPowerReduction),
		}, nil
	}

	blockHash := make([]byte, 32)
	_, err := rand.Read(blockHash)
	if err != nil {
		t.Fatal(err)
	}

	partSetHeaderHash := make([]byte, 32)
	_, err = rand.Read(partSetHeaderHash)
	if err != nil {
		t.Fatal(err)
	}

	height, err := rand.Int(rand.Reader, new(big.Int).SetUint64(^uint64(0)))
	if err != nil {
		t.Fatal(err)
	}

	round, err := rand.Int(rand.Reader, new(big.Int).SetUint64(^uint64(0)))
	if err != nil {
		t.Fatal(err)
	}

	chainId, err := rand.Int(rand.Reader, new(big.Int).SetUint64(^uint64(0)))
	if err != nil {
		t.Fatal(err)
	}

	vote := types.CanonicalVote{
		Type:   types.PrecommitType,
		Height: height.Int64(),
		Round:  round.Int64(),
		BlockID: &types.CanonicalBlockID{
			Hash: blockHash,
			PartSetHeader: types.CanonicalPartSetHeader{
				Total: 1,
				Hash:  partSetHeaderHash,
			},
		},
		ChainID: fmt.Sprintf("union-devnet-%d", chainId.Uint64()),
	}

	privKeys := make([]cometbn254.PrivKey, nbOfValidators)
	validators := make([]*types.SimpleValidator, nbOfValidators)
	totalPower := int64(0)
	for i := 0; i < len(validators); i++ {
		privKeys[i] = cometbn254.GenPrivKey()
		val, err := toValidator(privKeys[i].PubKey().Bytes())
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
		if votingPower >= int(totalPower)/3*2 {
			break
		}
		index, err := rand.Int(rand.Reader, big.NewInt(int64(nbOfValidators)))
		if err != nil {
			t.Fatal(err)
		}
		i := index.Int64()
		if bitmap.Bit(int(i)) == 0 {
			votingPower += int(validators[i].VotingPower)
			bitmap.SetBit(&bitmap, int(i), 1)
			sig, err := privKeys[i].Sign(signedBytes)
			if err != nil {
				t.Fatal(err)
			}
			signatures = append(signatures, sig)
		}
	}

	trustedValidators := validators
	untrustedValidators := validators

	trustedSignatures := signatures
	untrustedSignatures := signatures

	trustedBitmap := bitmap
	untrustedBitmap := bitmap

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

	log.Println("Marshalling trusted validators...")
	trustedValidatorsInput, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	log.Println("Aggregating trusted signature...")
	trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	log.Println("Marshalling untrusted validators...")
	untrustedValidatorsInput, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
	if err != nil {
		t.Fatal(err)
	}

	log.Println("Aggregating untrusted signature...")
	untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
	if err != nil {
		t.Fatal(err)
	}

	t.Logf("Nb of validators: %d", len(trustedValidators))
	t.Logf("Nb of signatures: %d", len(trustedSignatures))

	trustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(trustedAggregatedSignature),
		Validators:    trustedValidatorsInput,
		NbOfVal:       len(trustedValidators),
		NbOfSignature: len(trustedSignatures),
		Bitmap:        trustedBitmap,
	}

	untrustedInput := TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(untrustedAggregatedSignature),
		Validators:    untrustedValidatorsInput,
		NbOfVal:       len(untrustedValidators),
		NbOfSignature: len(untrustedSignatures),
		Bitmap:        untrustedBitmap,
	}

	hmX, hmY := cometbn254.HashToField2(signedBytes)

	circuit := Circuit{
		TrustedInput:             trustedInput,
		UntrustedInput:           untrustedInput,
		ExpectedTrustedValRoot:   trustedValidatorsRoot,
		ExpectedUntrustedValRoot: untrustedValidatorsRoot,
		Message:                  [2]frontend.Variable{hmX, hmY},
	}

	assert := test.NewAssert(t)
	assert.CheckCircuit(&Circuit{}, test.WithValidAssignment(&circuit), test.WithCurves(ecc.BN254), test.WithBackends(back.GROTH16), test.NoFuzzing())
}
