package grpc

import (
	"bufio"
	"bytes"
	context "context"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	grpc "galois/grpc/api/v3"
	"galois/pkg/lightclient"
	bls12381gadget "galois/pkg/lightclient/bls12381"
	lcgadget "galois/pkg/lightclient/nonadjacent"
	// "io"
	"math/big"
	"os"
	"sync"
	"sync/atomic"
	"time"

	types "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	// ce "github.com/cometbft/cometbft/crypto/encoding"
	// "github.com/cometbft/cometbft/crypto/merkle"
	"github.com/consensys/gnark-crypto/ecc"
	// "github.com/consensys/gnark-crypto/ecc/bn254"
	// "github.com/consensys/gnark-crypto/ecc/bn254/fr"
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

		log.Debug().Msg("Marshaling trusted validators...")
		trustedValidators, trustedValidatorsRoot, err := MarshalValidators(req.TrustedCommit.Validators)
		if err != nil {
			return nil, fmt.Errorf("Could not marshal trusted validators %s", err)
		}

		log.Debug().Msg("Aggregating trusted signature...")
		trustedAggregatedSignature, err := AggregateSignatures(req.TrustedCommit.Signatures)
		if err != nil {
			return nil, fmt.Errorf("Could not aggregate trusted signature %s", err)
		}

		log.Debug().Msg("Marshaling untrusted validators...")
		untrustedValidators, _, err := MarshalValidators(req.UntrustedCommit.Validators)
		if err != nil {
			return nil, fmt.Errorf("Could not marshal untrusted validators %s", err)
		}

		log.Debug().Msg("Aggregating untrusted signature...")
		untrustedAggregatedSignature, err := AggregateSignatures(req.UntrustedCommit.Signatures)
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

		uncons := func(b []byte) lightclient.UnconsHash {
			return lightclient.UnconsHash{
				Head: b[0],
				Tail: b[1:],
			}
		}

		getInputsHash := func(chainID string, h *types.Header, trustedValidatorsHash []byte) []byte {
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
			writeMiMCHash([]byte(chainID))
			writeI64(h.Height)
			writeI64(h.Time.Unix())
			writeI64(int64(h.Time.Nanosecond()))
			writeMiMCHash(h.ValidatorsHash)
			writeMiMCHash(h.NextValidatorsHash)
			writeHash(h.AppHash)
			writeMiMCHash(trustedValidatorsHash)
			hash := sha256.Sum256(buff)
			return hash[1:]
		}

		inputsHash := getInputsHash(req.Vote.ChainID, req.UntrustedHeader, trustedValidatorsRoot)

		log.Debug().Hex("request_hash", proveKey[:]).Hex("inputs_hash", inputsHash).Send()

		witness := lcgadget.Circuit{
			DomainSeparationTag: []byte(cometbn254.CometblsSigDST),
			TrustedInput:        trustedInput,
			TrustedValRoot:      trustedValidatorsRoot,
			UntrustedInput:      untrustedInput,
			Vote: lightclient.BlockVote{
				BlockPartSetHeaderTotal: req.Vote.BlockID.PartSetHeader.Total,
				BlockPartSetHeaderHash:  uncons(req.Vote.BlockID.PartSetHeader.Hash),
				Round:                   req.Vote.Round,
			},
			Header: lightclient.BlockHeader{
				VersionBlock:                req.UntrustedHeader.Version.Block,
				VersionApp:                  req.UntrustedHeader.Version.App,
				ChainID:                     []byte(req.UntrustedHeader.ChainID),
				Height:                      req.UntrustedHeader.Height,
				TimeSecs:                    req.UntrustedHeader.Time.Unix(),
				TimeNanos:                   req.UntrustedHeader.Time.Nanosecond(),
				LastBlockHash:               req.UntrustedHeader.LastBlockId.Hash,
				LastBlockPartSetHeaderTotal: req.UntrustedHeader.LastBlockId.PartSetHeader.Total,
				LastBlockPartSetHeaderHash:  uncons(req.UntrustedHeader.LastBlockId.PartSetHeader.Hash),
				LastCommitHash:              uncons(req.UntrustedHeader.LastCommitHash),
				DataHash:                    uncons(req.UntrustedHeader.DataHash),
				ValidatorsHash:              req.UntrustedHeader.ValidatorsHash,
				NextValidatorsHash:          req.UntrustedHeader.NextValidatorsHash,
				ConsensusHash:               uncons(req.UntrustedHeader.ConsensusHash),
				AppHash:                     uncons(req.UntrustedHeader.AppHash),
				LastResultsHash:             uncons(req.UntrustedHeader.LastResultsHash),
				EvidenceHash:                uncons(req.UntrustedHeader.EvidenceHash),
				ProposerAddress:             uncons(req.UntrustedHeader.ProposerAddress),
			},
			InputsHash: inputsHash,
		}

		privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
		if err != nil {
			return nil, fmt.Errorf("Could not create witness %s", err)
		}

		log.Debug().Hex("request_hash", proveKey[:]).Msg("proving")
		proof, err := backend.Prove(constraint.R1CS(&p.innerCs), backend.ProvingKey(&p.innerPk), privateWitness, backend_opts.WithProverHashToFieldFunction(&cometblsHashToField{}))
		if err != nil {
			return nil, fmt.Errorf("Prover failed with %s", err)
		}

		publicWitness, err := privateWitness.Public()
		if err != nil {
			return nil, fmt.Errorf("Could not extract public inputs from witness %s", err)
		}

		// TODO(aeryz): this is probably not necessary, but could be a nice assertion idk
		err = backend.Verify(
			proof,
			backend.VerifyingKey(&p.innerVk),
			publicWitness,
		)

		if err != nil {
			return nil, fmt.Errorf("Could not verify the groth16 proof %s", err)
		}

		circuitVk, err := groth16.ValueOfVerifyingKey[gadget.G1Affine, gadget.G2Affine, gadget.GTEl](backend.VerifyingKey(&p.vk))
		if err != nil {
			return nil, fmt.Errorf("Could not get the verifying key %s", err)
		}

		circuitWitness, err := groth16.ValueOfWitness[gadget.ScalarField](publicWitness)
		if err != nil {
			return nil, fmt.Errorf("Could not get the witness %s", err)
		}

		circuitProof, err := groth16.ValueOfProof[gadget.G1Affine, gadget.G2Affine](proof)
		if err != nil {
			return nil, fmt.Errorf("Could not get the proof %s", err)
		}

		bls12381Witness := &bls12381gadget.Circuit[gadget.ScalarField, gadget.G1Affine, gadget.G2Affine, gadget.GTEl]{
			InnerWitness: circuitWitness,
			Proof:        circuitProof,
			VerifyingKey: circuitVk,
		}

		privateWitness, err = frontend.NewWitness(bls12381Witness, ecc.BLS12_381.ScalarField())

		proof, err = backend.Prove(constraint.R1CS(&p.cs), backend.ProvingKey(&p.pk), privateWitness)

		var proofCommitment []byte
		var commitmentPOK []byte
		switch _proof := proof.(type) {
		case *backend_bn254.Proof:
			if len(p.vk.PublicAndCommitmentCommitted) != 1 {
				return nil, fmt.Errorf("Expected a single proof commitment, got: %d", len(p.vk.PublicAndCommitmentCommitted))
			}
			proofCommitment = _proof.Commitments[0].Marshal()
			commitmentPOK = _proof.CommitmentPok.Marshal()
			break
		default:
			return nil, fmt.Errorf("Impossible: proof backend must be BN254 at this point")
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
		evmProof := append(append(proofBz[:256], proofCommitment...), commitmentPOK...)

		proveRes := grpc.ProveResponse{
			Proof: &grpc.ZeroKnowledgeProof{
				Content:           proofBz,
				CompressedContent: compressedProofBz,
				PublicInputs:      publicInputs,
				EvmProof:          evmProof,
			},
			TrustedValidatorSetRoot: trustedValidatorsRoot,
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

func (p *proverServer) VerifyBls12381(ctx context.Context, req *grpc.VerifyRequest) (*grpc.VerifyResponse, error) {
	log.Debug().Msg("Verifying...")

	var proof backend_bn254.Proof
	_, err := proof.ReadFrom(bytes.NewReader(req.Proof.CompressedContent))
	if err != nil {
		return nil, fmt.Errorf("Failed to read compressed proof: %w", err)
	}

	witness := lcgadget.Circuit{
		InputsHash: req.InputsHash,
	}

	publicWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField(), frontend.PublicOnly())
	if err != nil {
		return nil, fmt.Errorf("Unable to create private witness: %w", err)
	}

	reqJson, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}

	err = backend.Verify(
		backend.Proof(&proof),
		backend.VerifyingKey(&p.vk),
		publicWitness,
		backend_opts.WithVerifierHashToFieldFunction(&cometblsHashToField{}),
	)

	if err != nil {
		log.Error().RawJSON("request", reqJson).Hex("inputs_hash", req.InputsHash).Str("action", "verify").Err(err).Send()
		return &grpc.VerifyResponse{
			Valid: false,
		}, nil
	} else {
		log.Info().RawJSON("request", reqJson).Hex("inputs_hash", req.InputsHash).Str("action", "verify").Send()
		return &grpc.VerifyResponse{
			Valid: true,
		}, nil
	}
}

// func (p *proverServer) QueryStats(ctx context.Context, req *grpc.QueryStatsRequest) (*grpc.QueryStatsResponse, error) {
// 	log.Debug().Msg("Querying stats...")

// 	return &grpc.QueryStatsResponse{
// 		VariableStats: &grpc.VariableStats{
// 			NbInternalVariables: uint32(p.cs.GetNbInternalVariables()),
// 			NbSecretVariables:   uint32(p.cs.GetNbSecretVariables()),
// 			NbPublicVariables:   uint32(p.cs.GetNbPublicVariables()),
// 			NbConstraints:       uint32(p.cs.GetNbConstraints()),
// 			NbCoefficients:      uint32(p.cs.GetNbCoefficients()),
// 		},
// 		ProvingKeyStats: &grpc.ProvingKeyStats{
// 			NbG1: uint32(p.pk.NbG1()),
// 			NbG2: uint32(p.pk.NbG2()),
// 		},
// 		VerifyingKeyStats: &grpc.VerifyingKeyStats{
// 			NbG1:            uint32(p.vk.NbG1()),
// 			NbG2:            uint32(p.vk.NbG2()),
// 			NbPublicWitness: uint32(p.vk.NbPublicWitness()),
// 		},
// 		// Deprecated
// 		CommitmentStats: &grpc.CommitmentStats{
// 			NbPublicCommitted:  uint32(0),
// 			NbPrivateCommitted: uint32(0),
// 		},
// 	}, nil
// }

// Deprecated in favor of the Poll api
func (p *proverServer) ProveBls12381(ctx context.Context, req *grpc.ProveRequest) (*grpc.ProveResponse, error) {
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

	circuit := &bls12381gadget.Circuit[gadget.ScalarField, gadget.G1Affine, gadget.G2Affine, gadget.GTEl]{
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

func NewProverServerBls12381(maxJobs uint32, r1csPath, pkPath, vkPath, innerR1csPath, innerPkPath, innerVkPath string) (*proverServerBls12381, error) {
	cs, pk, vk, innerCs, innerPk, innerVk, err := loadOrCreateBls12381(r1csPath, pkPath, vkPath, innerR1csPath, innerPkPath, innerVkPath)
	if err != nil {
		return nil, err
	}

	return &proverServerBls12381{cs: cs, pk: pk, vk: vk, innerCs: innerCs, innerPk: innerPk, innerVk: innerVk, maxJobs: maxJobs}, nil
}
