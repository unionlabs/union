package cmd

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	provergrpc "github.com/unionlabs/union/galoisd/grpc/api/v3"
	"log"
	"math/big"
	"strconv"
	"time"

	"cosmossdk.io/math"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	tmtypes "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/proto/tendermint/version"
	"github.com/cometbft/cometbft/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
)

// Example call to the prover `Prove` and then `Verify` endpoints using hardcoded values dumped from a local devnet.
// The sole purpose of this command is to see a live example and understand how to interact with the prover.
func ExampleVerifyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Short: "Simulation of a client submitting a proof creation and proof verify requests",
		Use:   "example-verify [uri] [nb_of_validators]",
		Args:  cobra.ExactArgs(2),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
			nbOfValidators, err := strconv.Atoi(args[1])
			if err != nil {
				return err
			}

			// Nb of tokens for each val in devnet
			toValidator := func(pubKey []byte) (*tmtypes.SimpleValidator, error) {
				protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
				if err != nil {
					return &tmtypes.SimpleValidator{}, err
				}
				power, err := rand.Int(rand.Reader, big.NewInt(9223372036854775807/8))
				if err != nil {
					return &tmtypes.SimpleValidator{}, err
				}
				return &tmtypes.SimpleValidator{
					PubKey:      &protoPK,
					VotingPower: sdk.TokensToConsensusPower(math.NewInt(power.Int64()), sdk.DefaultPowerReduction),
				}, nil
			}

			privKeys := make([]cometbn254.PrivKey, nbOfValidators)
			validators := make([]*tmtypes.SimpleValidator, nbOfValidators)
			totalPower := int64(0)
			for i := 0; i < len(validators); i++ {
				privKeys[i] = cometbn254.GenPrivKey()
				val, err := toValidator(privKeys[i].PubKey().Bytes())
				if err != nil {
					return err
				}
				totalPower += val.VotingPower
				validators[i] = val
			}

			validatorsHash, err := marshalValidators(validators)
			if err != nil {
				return err
			}

			randomHash := func() []byte {
				value := make([]byte, 32)
				_, err = rand.Read(value)
				if err != nil {
					panic(err)
				}
				return value
			}

			randomMiMCHash := func() []byte {
				value := randomHash()
				value[0] = 0
				return value
			}

			chainID := "union-devnet-1337"

			header := &types.Header{
				Version: version.Consensus{
					Block: 11,
					App:   0,
				},
				ChainID: chainID,
				Height:  0xCAFEBABE,
				Time:    time.Now().UTC(),
				LastBlockID: types.BlockID{
					Hash: randomMiMCHash(),
					PartSetHeader: types.PartSetHeader{
						Total: 1,
						Hash:  randomHash(),
					},
				},
				LastCommitHash:     randomHash(),
				DataHash:           randomHash(),
				ValidatorsHash:     validatorsHash,
				NextValidatorsHash: validatorsHash,
				ConsensusHash:      randomHash(),
				AppHash:            randomHash(),
				LastResultsHash:    randomHash(),
				EvidenceHash:       randomHash(),
				ProposerAddress:    randomHash(),
			}

			vote := &tmtypes.Vote{
				Type:   tmtypes.PrecommitType,
				Height: 0xCAFEBABE,
				Round:  0xC0DE,
				BlockID: tmtypes.BlockID{
					Hash: header.Hash(),
					PartSetHeader: tmtypes.PartSetHeader{
						Total: 1,
						Hash:  randomHash(),
					},
				},
			}

			signedBytes := types.VoteSignBytes(chainID, vote)
			if err != nil {
				return err
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
					return err
				}
				i := index.Int64()
				if bitmap.Bit(int(i)) == 0 {
					votingPower += int(validators[i].VotingPower)
					bitmap.SetBit(&bitmap, int(i), 1)
					sig, err := privKeys[i].Sign(signedBytes)
					if err != nil {
						return err
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

			canonicalVote := types.CanonicalizeVote(chainID, vote)

			proveRes, err := client.Prove(ctx, &provergrpc.ProveRequest{
				Vote:            &canonicalVote,
				UntrustedHeader: header.ToProto(),
				TrustedCommit: &provergrpc.ValidatorSetCommit{
					Validators: trustedValidators,
					Signatures: trustedSignatures,
					Bitmap:     trustedBitmap.Bytes(),
				},
				UntrustedCommit: &provergrpc.ValidatorSetCommit{
					Validators: untrustedValidators,
					Signatures: untrustedSignatures,
					Bitmap:     untrustedBitmap.Bytes(),
				},
			})
			if err != nil {
				return err
			}

			headerJSON, err := json.Marshal(header)
			if err != nil {
				return err
			}

			fmt.Printf("Header: %s\n", headerJSON)
			fmt.Printf("Vote: %X\n", signedBytes)
			fmt.Printf("Gnark Proof: %X\n", proveRes.Proof.Content)
			fmt.Printf("Public inputs: %X\n", proveRes.Proof.PublicInputs)
			fmt.Printf("Trusted root: %X\n", proveRes.TrustedValidatorSetRoot)
			fmt.Printf("EVM compatible ZKP: %X\n", proveRes.Proof.EvmProof)

			inputsHash := func(h *types.Header) []byte {
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

			verifyRes, err := client.Verify(ctx, &provergrpc.VerifyRequest{
				Proof:      proveRes.Proof,
				InputsHash: inputsHash(header),
			})

			if err != nil {
				log.Fatal(err)
			}

			log.Printf("Result: %v\n", verifyRes.Valid)

			return nil
		}),
	}
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
