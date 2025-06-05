package grpc

import (
	"bufio"
	"bytes"
	context "context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	grpc "galois/grpc/api/v3"
	"galois/pkg/lightclient"
	bls12381gadget "galois/pkg/lightclient/bls12381"
	// "io"
	"math/big"
	"os"
	"sync"
	"sync/atomic"
	"time"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	backend_opts "github.com/consensys/gnark/backend"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bls12381 "github.com/consensys/gnark/backend/groth16/bls12-381"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
	cs_bls12381 "github.com/consensys/gnark/constraint/bls12-381"
	cs_bn254 "github.com/consensys/gnark/constraint/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/recursion/groth16"
	"github.com/holiman/uint256"
	"github.com/rs/zerolog/log"
)

type proverServerBls12381 struct {
	grpc.UnimplementedUnionProverAPIServer
	cs      cs_bls12381.R1CS
	pk      backend_bls12381.ProvingKey
	vk      backend_bls12381.VerifyingKey
	innerCs cs_bn254.R1CS
	innerPk backend_bn254.ProvingKey
	innerVk backend_bn254.VerifyingKey
	maxJobs uint32
	nbJobs  atomic.Uint32
	results sync.Map
}

type bls12381HashToField struct {
	data []byte
}

func (c *bls12381HashToField) Write(p []byte) (n int, err error) {
	c.data = append(c.data, p...)
	return len(p), nil
}

func (c *bls12381HashToField) Sum(b []byte) []byte {
	hmac := hmac.New(cometbn254.Hash, []byte(cometbn254.CometblsHMACKey))
	hmac.Write(c.data)
	modMinusOne := new(big.Int).Sub(fr.Modulus(), big.NewInt(1))
	num := new(big.Int).SetBytes(hmac.Sum(nil))
	num.Mod(num, modMinusOne)
	num.Add(num, big.NewInt(1))
	val, overflow := uint256.FromBig(num)
	if overflow {
		panic("impossible; qed;")
	}
	valBytes := val.Bytes32()
	var e fr.Element
	err := e.SetBytesCanonical(valBytes[:])
	if err != nil {
		panic("impossible; qed;")
	}

	eB := e.Bytes()
	return append(b, eB[:]...)
}

func (c *bls12381HashToField) Reset() {
	c.data = []byte{}
}

func (c *bls12381HashToField) Size() int {
	return fr.Bytes
}

func (c *bls12381HashToField) BlockSize() int {
	return fr.Bytes
}

func (*proverServerBls12381) mustEmbedUnimplementedUnionProverAPIServer() {}

func (p *proverServerBls12381) Poll(ctx context.Context, pollReq *grpc.PollRequest) (*grpc.PollResponse, error) {
	req := pollReq.Request

	if len(req.TrustedCommit.Validators) > lightclient.MaxVal {
		return nil, fmt.Errorf("The circuit can handle a maximum of %d validators", lightclient.MaxVal)
	}
	if len(req.UntrustedCommit.Validators) > lightclient.MaxVal {
		return nil, fmt.Errorf("The circuit can handle a maximum of %d validators", lightclient.MaxVal)
	}
	if len(req.TrustedCommit.Signatures) > len(req.TrustedCommit.Signatures) {
		return nil, fmt.Errorf("More signatures than validators")
	}
	if len(req.UntrustedCommit.Signatures) > len(req.UntrustedCommit.Signatures) {
		return nil, fmt.Errorf("More signatures than validators")
	}

	reqJson, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}
	proveKey := sha256.Sum256(reqJson)

	prove := func() (*grpc.ProveResponse, error) {
		innerProof, err := Prove(req, &p.innerCs, &p.innerPk, &p.innerVk)

		circuitVk, err := groth16.ValueOfVerifyingKey[gadget.G1Affine, gadget.G2Affine, gadget.GTEl](backend.VerifyingKey(&p.innerVk))
		if err != nil {
			return nil, fmt.Errorf("Could not get the verifying key %s", err)
		}

		circuitWitness, err := groth16.ValueOfWitness[gadget.ScalarField](innerProof.PublicWitness)
		if err != nil {
			return nil, fmt.Errorf("Could not get the witness %s", err)
		}

		circuitProof, err := groth16.ValueOfProof[gadget.G1Affine, gadget.G2Affine](innerProof.Proof)
		if err != nil {
			return nil, fmt.Errorf("Could not get the proof %s", err)
		}

		commitmentHash := cometbn254.HashToField(innerProof.ProofCommitment.Marshal())
		bls12381Witness := &bls12381gadget.Circuit{
			InnerWitness:    circuitWitness,
			Proof:           circuitProof,
			VerifyingKey:    circuitVk,
			CommitmentHash:  commitmentHash.BigInt(new(big.Int)),
			CommitmentX:     innerProof.ProofCommitment.X.BigInt(new(big.Int)),
			CommitmentY:     innerProof.ProofCommitment.Y.BigInt(new(big.Int)),
			InnerInputsHash: innerProof.InputsHash,
		}

		fmt.Println("inputs hash ", innerProof.InputsHash)

		privateWitness, err := frontend.NewWitness(bls12381Witness, ecc.BLS12_381.ScalarField())
		if err != nil {
			return nil, fmt.Errorf("Witness err %s", err)
		}

		proof, err := backend.Prove(constraint.R1CS(&p.cs), backend.ProvingKey(&p.pk), privateWitness, backend_opts.WithProverHashToFieldFunction(&bls12381HashToField{}))
		if err != nil {
			return nil, fmt.Errorf("Could not prove %s", err)
		}

		var proofCommitment []byte
		var commitmentPOK []byte
		switch _proof := proof.(type) {
		case *backend_bls12381.Proof:
			if len(p.vk.PublicAndCommitmentCommitted) != 1 {
				return nil, fmt.Errorf("Expected a single proof commitment, got: %d", len(p.vk.PublicAndCommitmentCommitted))
			}
			proofCommitment = _proof.Commitments[0].Marshal()
			commitmentPOK = _proof.CommitmentPok.Marshal()
			break
		default:
			return nil, fmt.Errorf("Impossible: proof backend must be BLS12381 at this point")
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
		// The proof is an uncompressed proof serialized by gnark, we extract A(G1)/B(G2)/C(G1) and then append the commitment and its POK.
		// The EVM verifier has been extended to support this two extra public inputs.
		evmProof := append(append(append(proofBz[:384], proofCommitment...), commitmentPOK...), innerProof.ProofCommitment.Marshal()...)

		proveRes := grpc.ProveResponse{
			Proof: &grpc.ZeroKnowledgeProof{
				Content:           proofBz,
				CompressedContent: compressedProofBz,
				PublicInputs:      publicInputs,
				EvmProof:          evmProof,
			},
			TrustedValidatorSetRoot: innerProof.TrustedValidatorsRoot,
		}

		return &proveRes, nil
	}

	result, found := p.results.LoadOrStore(proveKey, &grpc.ProveRequestPending{})
	if found {
		log.Debug().Hex("request_hash", proveKey[:]).Msg("poll")

		switch _result := result.(type) {
		case *grpc.ProveRequestPending:
			return &grpc.PollResponse{
				Result: &grpc.PollResponse_Pending{
					Pending: _result,
				},
			}, nil
		case *grpc.ProveResponse:
			return &grpc.PollResponse{
				Result: &grpc.PollResponse_Done{
					Done: &grpc.ProveRequestDone{
						Response: _result,
					},
				},
			}, nil
		case error:
			return &grpc.PollResponse{
				Result: &grpc.PollResponse_Failed{
					Failed: &grpc.ProveRequestFailed{
						Message: _result.Error(),
					},
				},
			}, nil
		}
	} else {
		log.Info().Hex("request_hash", proveKey[:]).Msg("new")

		for true {
			nbJobs := p.nbJobs.Load()
			if nbJobs >= p.maxJobs {
				p.results.Delete(proveKey)
				return nil, fmt.Errorf("busy_building")
			} else {
				if swapped := p.nbJobs.CompareAndSwap(nbJobs, nbJobs+1); swapped {
					break
				}
			}
			time.Sleep(10 * time.Millisecond)
		}

		go func() {
			proveRes, err := prove()
			if err != nil {
				log.Error().Str("action", "prove").Hex("request_hash", proveKey[:]).RawJSON("request", reqJson).Err(err).Send()
				p.results.Store(proveKey, fmt.Errorf("failed to generate proof: %v", err))
			} else {
				resJson, _ := json.Marshal(proveRes)
				log.Info().Str("action", "prove").Hex("request_hash", proveKey[:]).RawJSON("request", reqJson).RawJSON("response", resJson).Send()
				p.results.Store(proveKey, proveRes)
			}
			for true {
				value := p.nbJobs.Load()
				if swapped := p.nbJobs.CompareAndSwap(value, value-1); swapped {
					break
				}
				time.Sleep(10 * time.Millisecond)
			}
		}()
	}

	return &grpc.PollResponse{
		Result: &grpc.PollResponse_Pending{
			Pending: &grpc.ProveRequestPending{},
		},
	}, nil
}

func (p *proverServerBls12381) QueryStats(ctx context.Context, req *grpc.QueryStatsRequest) (*grpc.QueryStatsResponse, error) {
	log.Debug().Msg("Querying stats...")

	return &grpc.QueryStatsResponse{
		VariableStats: &grpc.VariableStats{
			NbInternalVariables: uint32(p.cs.GetNbInternalVariables()),
			NbSecretVariables:   uint32(p.cs.GetNbSecretVariables()),
			NbPublicVariables:   uint32(p.cs.GetNbPublicVariables()),
			NbConstraints:       uint32(p.cs.GetNbConstraints()),
			NbCoefficients:      uint32(p.cs.GetNbCoefficients()),
		},
		ProvingKeyStats: &grpc.ProvingKeyStats{
			NbG1: uint32(p.pk.NbG1()),
			NbG2: uint32(p.pk.NbG2()),
		},
		VerifyingKeyStats: &grpc.VerifyingKeyStats{
			NbG1:            uint32(p.vk.NbG1()),
			NbG2:            uint32(p.vk.NbG2()),
			NbPublicWitness: uint32(p.vk.NbPublicWitness()),
		},
		// Deprecated
		CommitmentStats: &grpc.CommitmentStats{
			NbPublicCommitted:  uint32(0),
			NbPrivateCommitted: uint32(0),
		},
	}, nil
}

func loadOrCreateBls12381(r1csPath, pkPath, vkPath, innerR1csPath, innerPkPath, innerVkPath string) (cs_bls12381.R1CS, backend_bls12381.ProvingKey, backend_bls12381.VerifyingKey, cs_bn254.R1CS, backend_bn254.ProvingKey, backend_bn254.VerifyingKey, error) {
	csInner, pkInner, vkInner, err := loadOrCreate(innerR1csPath, innerPkPath, innerVkPath)

	if err != nil {
		panic(err)
	}

	cs := cs_bls12381.R1CS{}
	pk := backend_bls12381.ProvingKey{}
	vk := backend_bls12381.VerifyingKey{}

	if _, err := os.Stat(r1csPath); err == nil {
		if _, err = os.Stat(pkPath); err == nil {
			if _, err = os.Stat(vkPath); err == nil {
				log.Info().Msg("Loading circuit...")

				log.Debug().Msg("Loading R1CS...")
				err := readFrom(r1csPath, constraint.R1CS(&cs))
				if err != nil {
					return cs, pk, vk, csInner, pkInner, vkInner, err
				}

				log.Debug().Msg("Loading proving key...")
				err = readFrom(pkPath, backend.ProvingKey(&pk))
				if err != nil {
					return cs, pk, vk, csInner, pkInner, vkInner, err
				}

				log.Debug().Msg("Loading verifying key...")
				err = readFrom(vkPath, backend.VerifyingKey(&vk))
				if err != nil {
					return cs, pk, vk, csInner, pkInner, vkInner, err
				}

				var commitmentKeyBytes bytes.Buffer
				mem := bufio.NewWriter(&commitmentKeyBytes)
				_, err = vk.CommitmentKey.WriteRawTo(mem)
				if err != nil {
					return cs, pk, vk, csInner, pkInner, vkInner, err
				}
				mem.Flush()
				commitmentKey := commitmentKeyBytes.Bytes()

				log.Debug().
					Str("alpha", vk.G1.Alpha.String()).
					Str("beta", vk.G1.Beta.String()).
					Str("gamma", vk.G2.Gamma.String()).
					Str("delta", vk.G2.Delta.String()).
					Hex("pedersen", commitmentKey).
					Msg("verifying_key")

				return cs, pk, vk, csInner, pkInner, vkInner, nil
			}
		}
	}

	circuit := &bls12381gadget.Circuit{
		Proof:        groth16.PlaceholderProof[gadget.G1Affine, gadget.G2Affine](&csInner),
		InnerWitness: groth16.PlaceholderWitness[gadget.ScalarField](&csInner),
		VerifyingKey: groth16.PlaceholderVerifyingKey[gadget.G1Affine, gadget.G2Affine, gadget.GTEl](&csInner),
	}

	log.Info().Msg("Compiling circuit...")
	r1csInstance, err := frontend.Compile(ecc.BLS12_381.ScalarField(), r1cs.NewBuilder, circuit, frontend.WithCompressThreshold(300))
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}

	cs = *r1csInstance.(*cs_bls12381.R1CS)

	log.Debug().Msg("Setup PK/VK")
	err = backend_bls12381.Setup(&cs, &pk, &vk)
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}

	err = saveTo(r1csPath, r1csInstance)
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}
	err = saveTo(pkPath, backend.ProvingKey(&pk))
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}
	err = saveTo(vkPath, backend.VerifyingKey(&vk))
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}

	var commitmentKeyBytes bytes.Buffer
	mem := bufio.NewWriter(&commitmentKeyBytes)
	_, err = vk.CommitmentKey.WriteRawTo(mem)
	if err != nil {
		return cs, pk, vk, csInner, pkInner, vkInner, err
	}
	mem.Flush()
	commitmentKey := commitmentKeyBytes.Bytes()

	log.Debug().
		Str("alpha", vk.G1.Alpha.String()).
		Str("beta", vk.G1.Beta.String()).
		Str("gamma", vk.G2.Gamma.String()).
		Str("delta", vk.G2.Delta.String()).
		Hex("pedersen", commitmentKey).
		Msg("verifying_key")

	return cs, pk, vk, csInner, pkInner, vkInner, nil
}

func (p *proverServerBls12381) Prove(ctx context.Context, req *grpc.ProveRequest) (*grpc.ProveResponse, error) {
	for true {
		pollRes, err := p.Poll(ctx, &grpc.PollRequest{
			Request: req,
		})
		if err != nil {
			return nil, fmt.Errorf("%v", err)
		}
		if done := pollRes.GetDone(); done != nil {
			return done.Response, nil
		}
		if failed := pollRes.GetFailed(); failed != nil {
			return nil, fmt.Errorf("%v", failed.Message)
		}
		time.Sleep(1 * time.Second)
	}

	panic("impossible; qed;")
}

func NewProverServerBls12381(maxJobs uint32, r1csPath, pkPath, vkPath, innerR1csPath, innerPkPath, innerVkPath string) (*proverServerBls12381, error) {
	cs, pk, vk, innerCs, innerPk, innerVk, err := loadOrCreateBls12381(r1csPath, pkPath, vkPath, innerR1csPath, innerPkPath, innerVkPath)
	if err != nil {
		return nil, err
	}

	return &proverServerBls12381{cs: cs, pk: pk, vk: vk, innerCs: innerCs, innerPk: innerPk, innerVk: innerVk, maxJobs: maxJobs}, nil
}
