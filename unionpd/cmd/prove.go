package cmd

import (
	provergrpc "cometbls-prover/grpc/api/v1"
	"context"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"log"
	"math/big"
	"time"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

var ProveCmd = &cobra.Command{
	Use:   "prove",
	Short: "",
	Long:  ``,
	Run: func(cmd *cobra.Command, args []string) {
		conn, err := grpc.Dial("localhost:9091", grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			log.Fatalf("fail to dial: %v", err)
		}
		defer conn.Close()
		client := provergrpc.NewUnionProverAPIClient(conn)
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Hour)
		defer cancel()

		decodeB64 := func(s string) []byte {
			bz, err := base64.StdEncoding.DecodeString(s)
			if err != nil {
				panic(err)
			}
			return bz
		}

		// Nb of tokens for each val in devnet
		tokens, success := new(big.Int).SetString("1000000000000000000000", 10)
		if !success {
			panic("oops")
		}

		toValidator := func(pubKey []byte) *types.SimpleValidator {
			protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
			if err != nil {
				panic(err)
			}
			return &types.SimpleValidator{
				PubKey: &protoPK,
				VotingPower: sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction),
			}
		}

		blockHash, err := hex.DecodeString("CF8FB45282F3687C4BF305090C950BC28C7A7A5E35C2A9A1F5930D56A77F3C75")
		if err != nil {
			panic(err)
		}

		partSetHeaderHash, err := hex.DecodeString("39C604A64DDBDA8F2E0F31F0DF30315CE4B8E65DB91F74F29A5ED6926C70A03F")
		if err != nil {
			panic(err)
		}

		vote := types.CanonicalVote{
			Type:   types.PrecommitType,
			Height: 1,
			Round:  0,
			BlockID: &types.CanonicalBlockID{
				Hash: blockHash,
				PartSetHeader: types.CanonicalPartSetHeader{
					Total: 1,
					Hash:  partSetHeaderHash,
				},
			},
			ChainID:   "union-devnet-1",
		}

		validators := []*types.SimpleValidator{
			toValidator(decodeB64("wiY2IMV1eUwte40Km2Lw4H1zYGQ0ZvemMPoru9rf/pQ=")),
			toValidator(decodeB64("xCIuHcyesunreiQ86q+R2+KgP/rVYaGJ+XQGP8VShNc=")),
			toValidator(decodeB64("q/8jFgPQVjyyLqCvJo0Qsk8v8M0M51Ojw0Eg1KCsebo=")),
			toValidator(decodeB64("0QPPjuq9oaGp6nRm/SKrwNJkQTQDT2DtdVQm/9yJ3g0=")),
		}

		trustedValidators := validators
		untrustedValidators := validators

		signatures := [][]byte{
			decodeB64("k6kYQdqpOikXAPAm0uZGUHv6E2J5eT0SfehBSLcsFRELX0eWzq0spupRtr1z1f9I9fvSDVXcUNAMSLIjd9Rrtw=="),
			decodeB64("5E2bEigmCNTTf21Y1mP2mSKflg5r/oM1F3uGQDyqmbwN6lUmJ5lxx2lUTcmUWsYce3860+TQE/NORlW3d79Uzw=="),
			decodeB64("x0w000Y91HCtN5+j4cSO66c9Wsdznr/SomryXMXCQRQZWuBYZWIMF7K4hL+U35Q28IHpkGaJXWnRXs/oStm0Pw=="),
			decodeB64("zNVt/Ivgvjwl9EEGOo7YH3AfOTIpJCwcf+5IpslQmmUsfvrFMrwhOybQNgNukSZAzrujxCbZimBxbNgw/a5OCw=="),
		}

		trustedSignatures := signatures
		untrustedSignatures := signatures

		var bitmap big.Int
		bitmap.SetBit(&bitmap, 0, 1)
		bitmap.SetBit(&bitmap, 1, 1)
		bitmap.SetBit(&bitmap, 2, 1)
		bitmap.SetBit(&bitmap, 3, 1)

		trustedBitmap := bitmap
		untrustedBitmap := bitmap

		res, err := client.Prove(ctx, &provergrpc.ProveRequest{
			Vote: &vote,
			TrustedCommit: &provergrpc.ValidatorSetCommit{
				Validators: trustedValidators,
				Signatures: trustedSignatures,
				Bitmap: trustedBitmap.Bytes(),
			},
			UntrustedCommit: &provergrpc.ValidatorSetCommit{
				Validators: untrustedValidators,
				Signatures: untrustedSignatures,
				Bitmap: untrustedBitmap.Bytes(),
			},
		})
		if err != nil {
			panic(err)
		}
		fmt.Println(res)
	},
}
