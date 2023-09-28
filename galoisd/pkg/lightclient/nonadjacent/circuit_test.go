package nonadjacent

import (
	"encoding/base64"
	"galois/pkg/lightclient"
	"log"
	"math/big"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark/frontend"
	sdk "github.com/cosmos/cosmos-sdk/types"

	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
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

func TestRegression3(t *testing.T) {
	decodeB64 := func(s string) []byte {
		bz, err := base64.StdEncoding.DecodeString(s)
		if err != nil {
			log.Fatal(err)
		}
		return bz
	}

	toValidator := func(pubKey []byte, tokens *big.Int) *types.SimpleValidator {
		protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
		if err != nil {
			log.Fatal(err)
		}
		return &types.SimpleValidator{
			PubKey:      &protoPK,
			VotingPower: sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction),
		}
	}

	blockHash := decodeB64("z0wraQ1vIYTfIcKanrafANKkdwHBli4K13PC+kF5l7w=")

	partSetHeaderHash := decodeB64("ouHUWDwUcLJah07NEaMSiov595XLyJiPgyVQVflQ+d4=")

	vote := types.CanonicalVote{
		Type:   types.PrecommitType,
		Height: 516458,
		Round:  0,
		BlockID: &types.CanonicalBlockID{
			Hash: blockHash,
			PartSetHeader: types.CanonicalPartSetHeader{
				Total: 1,
				Hash:  partSetHeaderHash,
			},
		},
		ChainID: "union-testnet-3",
	}

	validators := []*types.SimpleValidator{
		toValidator(decodeB64("ncDNFj9SxNcXjtejrrDaN2TJ5AQJw4Hdbxu2/XJEdw4="), new(big.Int).SetUint64(1900000000000)), // cspell:disable-line
		toValidator(decodeB64("rwamLUSY+Ax3JsVtdqEO+yLwV//n9OfE2TNf9BHZRLk="), new(big.Int).SetUint64(1221870086811)), // cspell:disable-line
		toValidator(decodeB64("5lRQKoQ1z1wyCmyKFvhZnYxA54HnxdDWKnxksMw9K1Q="), new(big.Int).SetUint64(1200000000000)), // cspell:disable-line
		toValidator(decodeB64("z0wjC1SIYJ1QeZSVpcKnU7qLCSbpWlPNikV5/4WroDw="), new(big.Int).SetUint64(1001001000000)), // cspell:disable-line
		toValidator(decodeB64("5D/CuxUTxns90VZeANkLE9ZGdVChWb76xCGjikQW6Ig="), new(big.Int).SetUint64(1000000000000)), // cspell:disable-line
	}

	trustedValidators := validators
	untrustedValidators := validators

	signatures := [][]byte{
		decodeB64("xBhStL04DUCyQkHdnpghEk+V/16n0kOHdEQvoVWbo1cZp18vivnaiG0UvQVA0MJs3e3vNRcU0fqFQIDA6nfbEg=="),
		decodeB64("3SRDK+hVb7R0fqdRgXaITEApcujA1e/saiXgcocadvcStCZjudZr4i8arIJ5Um7zVNw79qz6yedPWFMAr/EPbg=="),
		decodeB64("rzFtlxtF8/+ElW1kyCxnpNOzbh/I17WRNby+ksNssqsoonVPynolYoCrfWYWqI6/si45AnlR/j0SyFZoZuFJcQ=="),
		decodeB64("hbKdcJt9znP03LtTdkG8yq4cCbCtxLkH3CmbDzySNbwAvvDDDc+1tuVVIZPE8oUS3DJDZXhqyWcyf4PLzGX1Iw=="),
		decodeB64("2ducQq7TuxbBjNArdVpvNfYnzpYQFHKwkPBR26K9HDssoPzvyq49q2n16rx/3BkOW3uwE7qexbRGQx8Pqdv7Fw=="),
	}

	trustedSignatures := signatures
	untrustedSignatures := signatures

	var bitmap big.Int
	bitmap.SetBit(&bitmap, 0, 1)
	bitmap.SetBit(&bitmap, 1, 1)
	bitmap.SetBit(&bitmap, 2, 1)
	bitmap.SetBit(&bitmap, 3, 1)
	bitmap.SetBit(&bitmap, 4, 1)

	t.Log("Bitmap: ", bitmap)

	trustedBitmap := bitmap
	untrustedBitmap := bitmap

	reverseBytes := func(numbers []byte) []byte {
		newNumbers := make([]byte, 0, len(numbers))
		for i := len(numbers) - 1; i >= 0; i-- {
			newNumbers = append(newNumbers, numbers[i])
		}
		return newNumbers
	}

	marshalValidators := func(validators []*types.SimpleValidator) ([lightclient.MaxVal][4]frontend.Variable, []byte, error) {
		validatorsProto := [lightclient.MaxVal][4]frontend.Variable{}
		// Make sure we zero initialize
		for i := 0; i < lightclient.MaxVal; i++ {
			validatorsProto[i][0] = 0
			validatorsProto[i][1] = 0
			validatorsProto[i][2] = 0
			validatorsProto[i][3] = 0
		}
		merkleTree := make([][]byte, len(validators))
		for i, val := range validators {
			protoEncoding, err := val.Marshal()
			if err != nil {
				return validatorsProto, nil, err
			}

			merkleTree[i] = protoEncoding

			tmPK, err := ce.PubKeyFromProto(*val.PubKey)
			if err != nil {
				return validatorsProto, nil, err
			}

			compressedPK := tmPK.Bytes()

			var PK curve.G1Affine
			_, err = PK.SetBytes(compressedPK)
			if err != nil {
				return validatorsProto, nil, err
			}

			PKX := PK.X.Bytes()
			PKY := PK.Y.Bytes()
			// Need to reverse to simplify circuit computation
			power := reverseBytes(protoEncoding[lightclient.ValProtoPower:])
			mask := compressedPK[0] >> 6
			validatorsProto[i][0] = PKX[:]
			validatorsProto[i][1] = PKY[:]
			validatorsProto[i][2] = power
			validatorsProto[i][3] = mask
		}
		log.Print(len(merkleTree))
		return validatorsProto, merkle.HashFromByteSlices(merkleTree), nil
	}

	aggregateSignatures := func(signatures [][]byte) (curve.G2Affine, error) {
		var aggregatedSignature curve.G2Affine
		var decompressedSignature curve.G2Affine
		for _, signature := range signatures {
			_, err := decompressedSignature.SetBytes(signature)
			if err != nil {
				return curve.G2Affine{}, err
			}
			aggregatedSignature.Add(&aggregatedSignature, &decompressedSignature)
		}
		return aggregatedSignature, nil
	}

	trustedValidatorsProto, trustedValidatorsRoot, err := marshalValidators(trustedValidators)
	if err != nil {
		log.Fatal(err)
	}
	trustedAggregatedSignature, err := aggregateSignatures(trustedSignatures)
	if err != nil {
		log.Fatal(err)
	}

	untrustedValidatorsProto, untrustedValidatorsRoot, err := marshalValidators(untrustedValidators)
	if err != nil {
		log.Fatal(err)
	}
	untrustedAggregatedSignature, err := aggregateSignatures(untrustedSignatures)
	if err != nil {
		log.Fatal(err)
	}

	trustedInput := TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(trustedAggregatedSignature),
		ProtoValidators: trustedValidatorsProto,
		NbOfVal:         len(trustedValidators),
		NbOfSignature:   len(trustedSignatures),
		Bitmap:          trustedBitmap,
	}

	untrustedInput := TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(untrustedAggregatedSignature),
		ProtoValidators: untrustedValidatorsProto,
		NbOfVal:         len(untrustedValidators),
		NbOfSignature:   len(untrustedSignatures),
		Bitmap:          untrustedBitmap,
	}

	signedBytes, err := protoio.MarshalDelimited(&vote)
	if err != nil {
		log.Fatal(err)
	}

	hmX, hmY := cometbft_bn254.HashToField2(signedBytes)

	witness := Circuit{
		TrustedInput:   trustedInput,
		UntrustedInput: untrustedInput,
		ExpectedTrustedValRoot: [2]frontend.Variable{
			trustedValidatorsRoot[0:16],
			trustedValidatorsRoot[16:32],
		},
		ExpectedUntrustedValRoot: [2]frontend.Variable{
			untrustedValidatorsRoot[0:16],
			untrustedValidatorsRoot[16:32],
		},
		Message: [2]frontend.Variable{hmX, hmY},
	}

	assert := test.NewAssert(t)

	var circuit Circuit

	assert.ProverSucceeded(&circuit, &witness, test.WithCurves(ecc.BN254), test.WithBackends(back.GROTH16), test.NoSerialization(), test.NoFuzzing())
}
