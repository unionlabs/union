package grpc

import (
	"bufio"
	"bytes"
	context "context"
	"encoding/json"
	"fmt"
	"galois/pkg/lightclient"
	lcgadget "galois/pkg/lightclient/nonadjacent"
	"io"
	"log"
	"math/big"
	"os"
	"runtime"
	"sync/atomic"
	"time"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/logger"
	"github.com/rs/zerolog"

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
	proving    atomic.Bool
}

func (*proverServer) mustEmbedUnimplementedUnionProverAPIServer() {}

func (p *proverServer) Verify(ctx context.Context, req *VerifyRequest) (*VerifyResponse, error) {
	log.Println("Verifying...")

	reqJson, err := json.MarshalIndent(req, "", "    ")
	if err != nil {
		return nil, err
	}
	log.Println(string(reqJson))

	var proof backend_bn254.Proof
	_, err = proof.ReadFrom(bytes.NewReader(req.Proof.CompressedContent))
	if err != nil {
		return nil, fmt.Errorf("Failed to read compressed proof: %w", err)
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
		return nil, fmt.Errorf("The block header X must be a BN254 fr.Element: %w", err)
	}

	var blockY fr.Element
	err = blockY.SetBytesCanonical(req.BlockHeaderY.Value)
	if err != nil {
		return nil, fmt.Errorf("The block header Y must be a BN254 fr.Element: %w", err)
	}

	validators := [lightclient.MaxVal]lightclient.Validator{}
	for i := 0; i < lightclient.MaxVal; i++ {
		validators[i].HashableX = 0
		validators[i].HashableY = 0
		validators[i].HashableXMSB = 0
		validators[i].HashableYMSB = 0
		validators[i].Power = 0
	}

	// We don't need the private input to verify, this is present to typecheck
	dummyInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:           gadget.G2Affine{},
		Validators:    validators,
		NbOfVal:       0,
		NbOfSignature: 0,
		Bitmap:        0,
	}

	witness := lcgadget.Circuit{
		TrustedInput:             dummyInput,
		UntrustedInput:           dummyInput,
		ExpectedTrustedValRoot:   req.TrustedValidatorSetRoot,
		ExpectedUntrustedValRoot: req.UntrustedValidatorSetRoot,
		Message:                  [2]frontend.Variable{req.BlockHeaderX.Value, req.BlockHeaderY.Value},
	}

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, fmt.Errorf("Unable to create private witness: %w", err)
	}

	publicWitness, err := privateWitness.Public()
	if err != nil {
		return nil, fmt.Errorf("Unable to extract public witness: %w", err)
	}

	err = backend.Verify(backend.Proof(&proof), p.vk, publicWitness)
	if err != nil {
		log.Println("Verification failed: %w", err)
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

func (p *proverServer) QueryStats(ctx context.Context, req *QueryStatsRequest) (*QueryStatsResponse, error) {
	log.Println("Querying stats...")

	return &QueryStatsResponse{
		VariableStats: &VariableStats{
			NbInternalVariables: uint32(p.cs.GetNbInternalVariables()),
			NbSecretVariables:   uint32(p.cs.GetNbSecretVariables()),
			NbPublicVariables:   uint32(p.cs.GetNbPublicVariables()),
			NbConstraints:       uint32(p.cs.GetNbConstraints()),
			NbCoefficients:      uint32(p.cs.GetNbCoefficients()),
		},
		ProvingKeyStats: &ProvingKeyStats{
			NbG1: uint32(p.pk.NbG1()),
			NbG2: uint32(p.pk.NbG2()),
		},
		VerifyingKeyStats: &VerifyingKeyStats{
			NbG1:            uint32(p.vk.NbG1()),
			NbG2:            uint32(p.vk.NbG2()),
			NbPublicWitness: uint32(p.vk.NbPublicWitness()),
		},
		CommitmentStats: &CommitmentStats{
			NbPublicCommitted:  uint32(p.commitment.NbPublicCommitted()),
			NbPrivateCommitted: uint32(p.commitment.NbPrivateCommitted),
		},
	}, nil
}

func (p *proverServer) Prove(ctx context.Context, req *ProveRequest) (*ProveResponse, error) {
	log.Println("Proving...")

	for true {
		swapped := p.proving.CompareAndSwap(false, true)
		if swapped {
			break
		} else {
			time.Sleep(1000)
		}
	}

	reqJson, err := json.MarshalIndent(req, "", "    ")
	if err != nil {
		return nil, err
	}
	log.Println(string(reqJson))

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
			var public bn254.G1Affine
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
			if leaf.MsbX {
				lcValidators[i].HashableXMSB = 1
			} else {
				lcValidators[i].HashableXMSB = 0
			}
			if leaf.MsbY {
				lcValidators[i].HashableYMSB = 1
			} else {
				lcValidators[i].HashableYMSB = 0
			}
			lcValidators[i].Power = leaf.VotingPower

			merkleTree[i] = leaf.Hash()
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
	trustedValidators, trustedValidatorsRoot, err := marshalValidators(req.TrustedCommit.Validators)
	if err != nil {
		return nil, fmt.Errorf("Could not marshal trusted validators %s", err)
	}

	log.Println("Aggregating trusted signature...")
	trustedAggregatedSignature, err := aggregateSignatures(req.TrustedCommit.Signatures)
	if err != nil {
		return nil, fmt.Errorf("Could not aggregate trusted signature %s", err)
	}

	log.Println("Marshalling untrusted validators...")
	untrustedValidators, untrustedValidatorsRoot, err := marshalValidators(req.UntrustedCommit.Validators)
	if err != nil {
		return nil, fmt.Errorf("Could not marshal untrusted validators %s", err)
	}

	log.Println("Aggregating untrusted signature...")
	untrustedAggregatedSignature, err := aggregateSignatures(req.UntrustedCommit.Signatures)
	if err != nil {
		return nil, fmt.Errorf("Could not aggregate untrusted signature %s", err)
	}

	trustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(trustedAggregatedSignature),
		Validators:    trustedValidators,
		NbOfVal:       len(req.TrustedCommit.Validators),
		NbOfSignature: len(req.TrustedCommit.Signatures),
		Bitmap:        new(big.Int).SetBytes(req.TrustedCommit.Bitmap),
	}

	untrustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:           gadget.NewG2Affine(untrustedAggregatedSignature),
		Validators:    untrustedValidators,
		NbOfVal:       len(req.UntrustedCommit.Validators),
		NbOfSignature: len(req.UntrustedCommit.Signatures),
		Bitmap:        new(big.Int).SetBytes(req.UntrustedCommit.Bitmap),
	}

	signedBytes, err := protoio.MarshalDelimited(req.Vote)
	if err != nil {
		return nil, fmt.Errorf("Could not marshal the vote %s", err)
	}

	hmX, hmY := cometbn254.HashToField2(signedBytes)

	witness := lcgadget.Circuit{
		TrustedInput:             trustedInput,
		UntrustedInput:           untrustedInput,
		ExpectedTrustedValRoot:   trustedValidatorsRoot,
		ExpectedUntrustedValRoot: untrustedValidatorsRoot,
		Message:                  [2]frontend.Variable{hmX, hmY},
	}

	witnessJson, err := json.MarshalIndent(witness, "", "    ")
	if err != nil {
		return nil, err
	}
	log.Println(string(witnessJson))

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, fmt.Errorf("Could not create witness %s", err)
	}

	logger.SetOutput(os.Stdout)
	logger.Logger().Level(zerolog.TraceLevel)

	log.Println("Executing proving backend...")
	proof, err := backend.Prove(p.cs, p.pk, privateWitness)
	if err != nil {
		return nil, fmt.Errorf("Prover failed with %s", err)
	}

	// Run GC to avoid high residency, a single prove call is very expensive in term of memory.
	runtime.GC()

	p.proving.Store(false)

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

	publicWitness, err := privateWitness.Public()
	if err != nil {
		return nil, fmt.Errorf("Could not extract public inputs from witness %s", err)
	}

	publicInputs, err := publicWitness.MarshalBinary()
	if err != nil {
		return nil, fmt.Errorf("Could not marshal public witness %s", err)
	}

	var proofBuffer bytes.Buffer
	mem := bufio.NewWriter(&proofBuffer)
	_, err = proof.WriteRawTo(mem)
	if err != nil {
		return nil, err
	}
	mem.Flush()
	proofBz := proofBuffer.Bytes()

	var compressedProofBuffer bytes.Buffer
	mem = bufio.NewWriter(&compressedProofBuffer)
	_, err = proof.WriteTo(mem)
	if err != nil {
		return nil, err
	}
	mem.Flush()
	compressedProofBz := compressedProofBuffer.Bytes()

	// Due to how gnark proves, we not only need the ZKP A/B/C points, but also a commitment hash and proof commitment.
	// The proof is an uncompressed proof serialized by gnark, we extract A(G1)/B(G2)/C(G1) and then append the commitment hash and commitment proof from the public inputs.
	// The EVM verifier has been extended to support this two extra public inputs.
	evmProof := append(append(proofBz[:256], commitmentHash...), proofCommitment...)

	return &ProveResponse{
		Proof: &ZeroKnowledgeProof{
			Content:           proofBz,
			CompressedContent: compressedProofBz,
			PublicInputs:      publicInputs,
			EvmProof:          evmProof,
		},
		TrustedValidatorSetRoot:   trustedValidatorsRoot,
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
