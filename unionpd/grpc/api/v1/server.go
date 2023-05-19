package grpc

import (
	"bufio"
	"bytes"
	context "context"
	"fmt"
	"io"
	"log"
	"math/big"
	"os"
	"runtime"
	"unionp/pkg/lightclient"
	lcgadget "unionp/pkg/lightclient/nonadjacent"

	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"

	// "github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
)

type proverServer struct {
	UnimplementedUnionProverAPIServer
	cs         constraint.ConstraintSystem
	pk         backend.ProvingKey
	vk         backend.VerifyingKey
	commitment constraint.Commitment
}

func (*proverServer) mustEmbedUnimplementedUnionProverAPIServer() {}

func (p *proverServer) Verify(ctx context.Context, req *VerifyRequest) (*VerifyResponse, error) {
	log.Println("Verifying...")

	var proof backend_bn254.Proof
	_, err := proof.ReadFrom(bytes.NewReader(req.Proof.Content))
	if err != nil {
		return nil, err
	}

	if len(req.TrustedValidatorSetRoot) != 32 {
		return nil, fmt.Errorf("The trusted validator set root must be a SHA256 hash")
	}
	if len(req.UntrustedValidatorSetRoot) != 32 {
		return nil, fmt.Errorf("The untrusted validator set root must be a SHA256 hash")
	}

	var blockX fr.Element
	err = blockX.SetBytesCanonical(req.BlockHeaderX.Value)
	if err != nil {
		return nil, fmt.Errorf("The block header X must be a BN254 fr.Element")
	}

	var blockY fr.Element
	err = blockY.SetBytesCanonical(req.BlockHeaderY.Value)
	if err != nil {
		return nil, fmt.Errorf("The block header Y must be a BN254 fr.Element")
	}

	validatorsProto := [lightclient.MaxVal][4]frontend.Variable{}
	for i := 0; i < lightclient.MaxVal; i++ {
		validatorsProto[i][0] = 0
		validatorsProto[i][1] = 0
		validatorsProto[i][2] = 0
		validatorsProto[i][3] = 0
	}

	// We don't need the private input to verify, this is present to typecheck
	dummyInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.G2Affine{},
		ProtoValidators: validatorsProto,
		NbOfVal:         0,
		NbOfSignature:   0,
		Bitmap:          0,
	}

	witness := lcgadget.Circuit{
		TrustedInput:   dummyInput,
		UntrustedInput: dummyInput,
		ExpectedTrustedValRoot: [2]frontend.Variable{
			req.TrustedValidatorSetRoot[0:16],
			req.TrustedValidatorSetRoot[16:32],
		},
		ExpectedUntrustedValRoot: [2]frontend.Variable{
			req.UntrustedValidatorSetRoot[0:16],
			req.UntrustedValidatorSetRoot[16:32],
		},
		Message: [2]frontend.Variable{req.BlockHeaderX, req.BlockHeaderY},
	}

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}

	publicWitness, err := privateWitness.Public()
	if err != nil {
		return nil, err
	}

	err = backend.Verify(backend.Proof(&proof), p.vk, publicWitness)
	if err != nil {
		log.Println(err)
		return &VerifyResponse{
			Valid: false,
		}, nil
	} else {
		return &VerifyResponse{
			Valid: true,
		}, nil
	}
}

func (p *proverServer) GenerateContract(ctx context.Context, req *GenerateContractRequest) (*GenerateContractResponse, error) {
	log.Println("Generating contract...")

	var buffer bytes.Buffer
	mem := bufio.NewWriter(&buffer)
	err := p.vk.ExportSolidity(mem)
	if err != nil {
		return nil, err
	}
	mem.Flush()

	return &GenerateContractResponse{
		Content: buffer.Bytes(),
	}, nil
}

func (p *proverServer) Prove(ctx context.Context, req *ProveRequest) (*ProveResponse, error) {
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

	trustedValidatorsProto, trustedValidatorsRoot, err := marshalValidators(req.TrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	trustedAggregatedSignature, err := aggregateSignatures(req.TrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	untrustedValidatorsProto, untrustedValidatorsRoot, err := marshalValidators(req.UntrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	untrustedAggregatedSignature, err := aggregateSignatures(req.UntrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	trustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(trustedAggregatedSignature),
		ProtoValidators: trustedValidatorsProto,
		NbOfVal:         len(req.TrustedCommit.Validators),
		NbOfSignature:   len(req.TrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(req.TrustedCommit.Bitmap),
	}

	untrustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(untrustedAggregatedSignature),
		ProtoValidators: untrustedValidatorsProto,
		NbOfVal:         len(req.UntrustedCommit.Validators),
		NbOfSignature:   len(req.UntrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(req.UntrustedCommit.Bitmap),
	}

	signedBytes, err := protoio.MarshalDelimited(req.Vote)
	if err != nil {
		return nil, err
	}

	hmX, hmY := cometbft_bn254.HashToField2(signedBytes)

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
		Message: [2]frontend.Variable{hmX, hmY},
	}

	log.Println("Witness: ", witness)

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}

	log.Println("Proving...")
	proof, err := backend.Prove(p.cs, p.pk, privateWitness)
	if err != nil {
		return nil, err
	}

	// Run GC to avoid high residency, a single prove call is very expensive in term of memory.
	runtime.GC()

	// F_r element
	var commitmentHash []byte
	// G1 uncompressed
	var proofCommitment []byte
	// Ugly but https://github.com/ConsenSys/gnark/issues/652
	switch _proof := proof.(type) {
	case *backend_bn254.Proof:
		if p.commitment.Is() {
			res, err := fr.Hash(p.commitment.SerializeCommitment(_proof.Commitment.Marshal(), []*big.Int{}, (fr.Bits-1)/8+1), []byte(constraint.CommitmentDst), 1)
			if err != nil {
				return nil, err
			}
			proofCommitment = _proof.Commitment.Marshal()
			commitmentHash = res[0].Marshal()
		}
		break
	default:
		return nil, fmt.Errorf("Impossible: proof backend must be BN254 at this point")
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

	publicInputs, err := publicWitness.MarshalBinary()
	if err != nil {
		return nil, err
	}

	return &ProveResponse{
		Proof: &ZeroKnowledgeProof{
			Content:      buffer.Bytes(),
			PublicInputs: append(append(publicInputs, commitmentHash...), proofCommitment...),
		},
		TrustedValidatorSetRoot: trustedValidatorsRoot,
		UntrustedValidatorSetRoot: untrustedValidatorsRoot,
	}, nil
}

func loadOrCreate(r1csPath string, pkPath string, vkPath string) (constraint.ConstraintSystem, backend.ProvingKey, backend.VerifyingKey, error) {
	r1csInstance := backend.NewCS(ecc.BN254)
	pk := backend.NewProvingKey(ecc.BN254)
	vk := backend.NewVerifyingKey(ecc.BN254)

	if _, err := os.Stat(r1csPath); err == nil {
		if _, err = os.Stat(pkPath); err == nil {
			if _, err = os.Stat(vkPath); err == nil {
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

	var commitment constraint.Commitment
	switch _pk := pk.(type) {
	case *backend_bn254.ProvingKey:
		switch _vk := vk.(type) {
		case *backend_bn254.VerifyingKey:
			_pk.CommitmentKey = _vk.CommitmentKey
			commitment = _vk.CommitmentInfo
			break
		}
		break
	default:
		return nil, fmt.Errorf("Impossible: vk backend must be BN254 at this point")
	}

	runtime.GC()

	return &proverServer{cs: cs, pk: pk, vk: vk, commitment: commitment}, nil
}

func readFrom(file string, obj io.ReaderFrom) error {
	f, err := os.OpenFile(file, os.O_RDONLY, os.ModePerm)
	if err != nil {
		return err
	}
	defer f.Close()
	obj.ReadFrom(f)
	return nil
}

func saveTo(file string, x io.WriterTo) error {
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
