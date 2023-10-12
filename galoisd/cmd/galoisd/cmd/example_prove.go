package cmd

import (
	"context"
	"encoding/hex"
	"fmt"
	provergrpc "galois/grpc/api/v1"
	"math/big"
	"math/rand"
	"strconv"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
)

// Example call to the prover `Prove` endpoint using hardcoded values dumped from a local devnet.
// The sole purpose of this command is to see a live example and understand how to interact with the prover.
func ExampleProveCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:  "example-prove [uri] [nb_validators]",
		Args: cobra.ExactArgs(2),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {

			nbOfValidators, err := strconv.Atoi(args[1])
			if err != nil {
				return err
			}

			// Nb of tokens for each val in devnet
			toValidator := func(pubKey []byte) (*types.SimpleValidator, error) {
				protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
				if err != nil {
					return &types.SimpleValidator{}, err
				}
				return &types.SimpleValidator{
					PubKey:      &protoPK,
					VotingPower: sdk.TokensToConsensusPower(sdk.NewInt(rand.Int63n(9223372036854775807/8)), sdk.DefaultPowerReduction),
				}, nil
			}

			blockHash, err := hex.DecodeString("7022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA8")
			if err != nil {
				return err
			}

			partSetHeaderHash, err := hex.DecodeString("41B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C")
			if err != nil {
				return err
			}

			vote := types.CanonicalVote{
				Type:   types.PrecommitType,
				Height: 574,
				Round:  0,
				BlockID: &types.CanonicalBlockID{
					Hash: blockHash,
					PartSetHeader: types.CanonicalPartSetHeader{
						Total: 1,
						Hash:  partSetHeaderHash,
					},
				},
				ChainID: "union-devnet-1",
			}

			privKeys := make([]cometbn254.PrivKey, nbOfValidators)
			validators := make([]*types.SimpleValidator, nbOfValidators)
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

			signedBytes, err := protoio.MarshalDelimited(&vote)
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
				index := rand.Int31n(int32(nbOfValidators))
				if bitmap.Bit(int(index)) == 0 {
					votingPower += int(validators[index].VotingPower)
					bitmap.SetBit(&bitmap, int(index), 1)
					sig, err := privKeys[index].Sign(signedBytes)
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

			res, err := client.Prove(ctx, &provergrpc.ProveRequest{
				Vote: &vote,
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

			fmt.Printf("Vote: %X\n", signedBytes)
			fmt.Printf("Gnark Proof: %X\n", res.Proof.Content)
			fmt.Printf("Public inputs: %X\n", res.Proof.PublicInputs)
			fmt.Printf("Trusted root: %X\n", res.TrustedValidatorSetRoot)
			fmt.Printf("Untrusted root: %X\n", res.UntrustedValidatorSetRoot)
			fmt.Printf("EVM compatible ZKP: %X\n", res.Proof.EvmProof)

			return nil
		}),
	}
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
