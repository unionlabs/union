package grpc

import (
	"bufio"
	"bytes"
	"cometbls-prover/pkg/lightclient"
	lcgadget "cometbls-prover/pkg/lightclient/nonadjacent"
	context "context"
	"crypto/hmac"
	"fmt"
	"io"
	"log"
	"math/big"
	"os"

	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/holiman/uint256"
	"golang.org/x/crypto/sha3"
)

func hashToField(msg []byte) fp.Element {
	hmac := hmac.New(sha3.NewLegacyKeccak256, []byte("CometBLS"))
	hmac.Write(msg)
	modMinusOne := new(big.Int).Sub(fp.Modulus(), big.NewInt(1))
	num := new(big.Int).SetBytes(hmac.Sum(nil))
	num.Mod(num, modMinusOne)
	num.Add(num, big.NewInt(1))
	val, overflow := uint256.FromBig(num)
	if overflow {
		panic("impossible; qed;")
	}
	valBytes := val.Bytes32()
	var element fp.Element
	err := element.SetBytesCanonical(valBytes[:])
	if err != nil {
		panic("impossible; qed;")
	}
	return element
}

func hashToField2(msg []byte) curve.E2 {
	e0 := hashToField(append([]byte{0}, msg...))
	e1 := hashToField(append([]byte{1}, msg...))
	return curve.E2{A0: e0, A1: e1}
}

type proverServer struct {
	UnimplementedUnionProverAPIServer
	cs constraint.ConstraintSystem
	pk backend.ProvingKey
	vk backend.VerifyingKey
}

func (*proverServer) mustEmbedUnimplementedUnionProverAPIServer() {}

func (p *proverServer) Prove(c context.Context, request *ProveRequest) (*ProveResponse, error) {
	log.Println("Proving...")

	reverseBytes := func(numbers []byte) []byte {
		newNumbers := make([]byte, 0, len(numbers))
		for i := len(numbers) - 1; i >= 0; i-- {
			newNumbers = append(newNumbers, numbers[i])
		}
		return newNumbers
	}

	marshalValidators := func(validators []*types.SimpleValidator) ([lightclient.MaxVal][4]frontend.Variable, []byte, error) {
		validatorsProto := [lightclient.MaxVal][4]frontend.Variable{}
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

			TmPK, err := ce.PubKeyFromProto(*val.PubKey)
			if err != nil {
				return validatorsProto, nil, err
			}

			compressedPK := TmPK.Bytes()

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

	trustedValidatorsProto, trustedValidatorsRoot, err := marshalValidators(request.TrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	trustedAggregatedSignature, err := aggregateSignatures(request.TrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	untrustedValidatorsProto, untrustedValidatorsRoot, err := marshalValidators(request.UntrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	untrustedAggregatedSignature, err := aggregateSignatures(request.UntrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	trustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(trustedAggregatedSignature),
		ProtoValidators: trustedValidatorsProto,
		NbOfVal:         len(request.TrustedCommit.Validators),
		NbOfSignature:   len(request.TrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(request.TrustedCommit.Bitmap),
	}

	untrustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(untrustedAggregatedSignature),
		ProtoValidators: untrustedValidatorsProto,
		NbOfVal:         len(request.UntrustedCommit.Validators),
		NbOfSignature:   len(request.UntrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(request.UntrustedCommit.Bitmap),
	}

	signedBytes, err := protoio.MarshalDelimited(request.Vote)
	if err != nil {
		return nil, err
	}

	hm := hashToField2(signedBytes)

	witness := lcgadget.Circuit{
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
		Message: [2]frontend.Variable{hm.A0.BigInt(new(big.Int)), hm.A1.BigInt(new(big.Int))},
	}

	log.Println("Witness: ", witness)

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}

	var commitment constraint.Commitment
	switch _pk := p.pk.(type) {
	case *backend_bn254.ProvingKey:
		switch _vk := p.vk.(type) {
		case *backend_bn254.VerifyingKey:
			_pk.CommitmentKey = _vk.CommitmentKey
			commitment = _vk.CommitmentInfo
			break
		}
		break
	default:
		return nil, fmt.Errorf("Invalid proving key type, must be BN254")
	}

	log.Println("Proving...")
	proof, err := backend.Prove(p.cs, p.pk, privateWitness)
	if err != nil {
		return nil, err
	}

	var commitmentHash []byte
	var proofCommitment []byte
	switch _proof := proof.(type) {
	case *backend_bn254.Proof:
		if commitment.Is() {
			res, err := fr.Hash(commitment.SerializeCommitment(_proof.Commitment.Marshal(), []*big.Int{}, (fr.Bits-1)/8+1), []byte(constraint.CommitmentDst), 1)
			if err != nil {
				return nil, err
			}
			proofCommitment = _proof.Commitment.Marshal()
			commitmentHash = res[0].Marshal()
		}
		break
	default:
		return nil, fmt.Errorf("Invalid proof type, must be BN254")
	}

	var buffer bytes.Buffer
	mem := bufio.NewWriter(&buffer)
	_, err = proof.WriteRawTo(mem)
	if err != nil {
		return nil, err
	}
	mem.Flush()

	publicWitness, err := privateWitness.Public()
	if err != nil {
		return nil, err
	}

	fmt.Println(publicWitness)

	publicInputs, err := publicWitness.MarshalBinary()
	if err != nil {
		return nil, err
	}

	proofBz := append(append(publicInputs, commitmentHash...), proofCommitment...)

	return &ProveResponse{
		Proof: &ZeroKnowledgeProof{
			Content: proofBz,
		},
	}, nil
}

func loadOrCreate(r1csPath string, pkPath string, vkPath string) (constraint.ConstraintSystem, backend.ProvingKey, backend.VerifyingKey, error) {
	r1csInstance := backend.NewCS(ecc.BN254)
	pk := backend.NewProvingKey(ecc.BN254)
	vk := backend.NewVerifyingKey(ecc.BN254)

	if _, err := os.Stat(r1csPath); err == nil {
		if _, err = os.Stat(pkPath); err == nil {
			if _, err = os.Stat(vkPath); err == nil {
				readFrom := func(file string, obj io.ReaderFrom) error {
					f, err := os.OpenFile(file, os.O_RDONLY, os.ModePerm)
					if err != nil {
						return err
					}
					defer f.Close()
					obj.ReadFrom(f)
					return nil
				}

				log.Println("Loading R1CS...")
				err := readFrom(r1csPath, r1csInstance)
				if err != nil {
					return nil, nil, nil, err
				}

				log.Println("Loading proving key...")
				err = readFrom(pkPath, pk)
				if err != nil {
					return nil, nil, nil, err
				}

				log.Println("Loading verifying key...")
				err = readFrom(vkPath, vk)
				if err != nil {
					return nil, nil, nil, err
				}

				return r1csInstance, pk, vk, nil
			}
		}
	}

	var circuit lcgadget.Circuit

	log.Println("Compiling circuit...")
	r1csInstance, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit, frontend.WithCompressThreshold(300))
	if err != nil {
		return nil, nil, nil, err
	}

	log.Println("Setup PK/VK")
	pk, vk, err = backend.Setup(r1csInstance)
	if err != nil {
		return nil, nil, nil, err
	}

	saveTo := func(file string, x io.WriterTo) error {
		log.Printf("Saving %s\n", file)
		f, err := os.Create(file)
		if err != nil {
			return err
		}
		defer f.Close()
		w := bufio.NewWriter(f)
		written, err := x.WriteTo(w)
		if err != nil {
			return err
		}
		log.Printf("Saved %d bytes\n", written)
		w.Flush()
		return nil
	}

	err = saveTo(r1csPath, r1csInstance)
	if err != nil {
		return nil, nil, nil, err
	}
	err = saveTo(pkPath, pk)
	if err != nil {
		return nil, nil, nil, err
	}
	err = saveTo(vkPath, vk)
	if err != nil {
		return nil, nil, nil, err
	}

	return r1csInstance, pk, vk, nil
}

func NewProverServer(r1csPath string, pkPath string, vkPath string) (*proverServer, error) {
	cs, pk, vk, err := loadOrCreate(r1csPath, pkPath, vkPath)
	if err != nil {
		return nil, err
	}
	return &proverServer{cs: cs, pk: pk, vk: vk}, nil
}
