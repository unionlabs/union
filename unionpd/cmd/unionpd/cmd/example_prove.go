package cmd

import (
	"context"
	"crypto/tls"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"log"
	"math/big"
	"time"
	provergrpc "unionp/grpc/api/v1"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

const (
	flagTLS = "tls"
)

// Example call to the prover `Prove` endpoint using hardcoded values dumped from a local devnet.
// The sole purpose of this command is to see a live example and understand how to interact with the prover.
func ExampleProveCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:  "example-prove [uri]",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {

			tlsEnabled, err := cmd.Flags().GetString(flagTLS)
			if err != nil {
				return err
			}

			var creds credentials.TransportCredentials
			if tlsEnabled == "yes" {
				creds = credentials.NewTLS(&tls.Config{})
			} else {
				creds = insecure.NewCredentials()
			}

			uri := args[0]
			conn, err := grpc.Dial(uri, grpc.WithTransportCredentials(creds))
			if err != nil {
				return err
			}
			defer conn.Close()
			client := provergrpc.NewUnionProverAPIClient(conn)
			ctx, cancel := context.WithTimeout(context.Background(), 3*time.Minute)
			defer cancel()

			decodeB64 := func(s string) []byte {
				bz, err := base64.StdEncoding.DecodeString(s)
				if err != nil {
					log.Fatal(err)
				}
				return bz
			}

			// Nb of tokens for each val in devnet
			tokens, success := new(big.Int).SetString("1000000000000000000000", 10)
			if !success {
				log.Fatal("Impossible; qed;")
			}

			toValidator := func(pubKey []byte) *types.SimpleValidator {
				protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
				if err != nil {
					log.Fatal(err)
				}
				return &types.SimpleValidator{
					PubKey:      &protoPK,
					VotingPower: sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction),
				}
			}

			// cSpell:disable
			/*
				"header": {
				      "version": {
				        "block": "11"
				      },
				      "chain_id": "union-devnet-1",
				      "height": "19",
				      "time": "2023-05-27T11:04:51.760274613Z",
				      "last_block_id": {
				        "hash": "C7CC6E2C14DEFEEEC193236649A9D139CDEC8709671920BB043B46AD242479FE",
				        "parts": {
				          "total": 1,
				          "hash": "6D346BF05A513257388252AC865BCFC08ED9F3CB913E4A9CF92371729C9E40FA"
				        }
				      },
				      "last_commit_hash": "5FDC2A4F647BA4AE0C30C286BBC7D05924D5FB15C2C5CA28EABA72FADB62A874",
				      "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "validators_hash": "941928B62E046F0ABA7730F63BA8C3A1E274D04A4D7E389566AB8F12F99EBE55",
				      "next_validators_hash": "941928B62E046F0ABA7730F63BA8C3A1E274D04A4D7E389566AB8F12F99EBE55",
				      "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
				      "app_hash": "E02A5AA10A7FAEB3B464BA04824DD582F766C0395C1D4699F4378D29CCBF6E01",
				      "last_results_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "proposer_address": "B64923E4C2514F5061B6CE83412B7969CF75FBCE"
				}

				"last_commit": {
				      "height": "19",
				      "round": 0,
				      "block_id": {
				        "hash": "F8579B3A521F8F704B991FFB8CA9040124A2F7BF5FFB9A013AEA9E223370AC5F",
				        "parts": {
				          "total": 1,
				          "hash": "ABE49229F06DC8E1F1BC3B564A6F06BEB5098B05F61971BF39297EAE4B1616AA"
				        }
				      },
				      "signatures": [
				        {
				          "block_id_flag": 2,
				          "validator_address": "3F2D3D0325AFEA6B8893378D635D566385207978",
				          "timestamp": "2023-05-27T11:04:57.354337747Z",
				          "signature": "0eliDuI/J2J6TpPN4oM7dy57qjWAYG5Pyea/Peor/+4Fn1OSK0d/jv+E0nM+ZmccwyT/MWRuZZ8SrNZj+dIWJA=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "647F28094C47745FFF28A03A97BC618D7EDE8CCB",
				          "timestamp": "2023-05-27T11:04:57.360753529Z",
				          "signature": "5F0LHP8lXyeHrNJhBA1cuzeCP21O0oEkvYu0e/dhHs0u6+YFPdhViB6ZbmZM11nycIAcJ33+kmlsqA/j1dugoQ=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "B64923E4C2514F5061B6CE83412B7969CF75FBCE",
				          "timestamp": "2023-05-27T11:04:57.540781878Z",
				          "signature": "rH79tXDFY5xdlWMH5LTOgo2maGE66onP+WdbZS7DAs0ocbGahCII8gGd0aRlbuiYAXLDINrBil/qAxathWwBeg=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "CCE6E5E42E299FA04AD73A609A882E2EA636811E",
				          "timestamp": "2023-05-27T11:04:57.438702389Z",
				          "signature": "wXh774QmqRjg6Ang3VW1Bi6Z+fUyIrbCatBiArnfhaICoiIKz0IGUog+LyVtZBYpRw1TK0XoQxvUMQybzONp5g=="
				        }
				      ]
				}


				"validators": [
				  {
				    "commission": {
				      "commission_rates": {
				        "max_change_rate": "0.010000000000000000",
				        "max_rate": "0.200000000000000000",
				        "rate": "0.100000000000000000"
				      },
				      "update_time": "2023-05-27T11:02:45.024436207Z"
				    },
				    "consensus_pubkey": {
				      "@type": "/cosmos.crypto.bn254.PubKey",
				      "key": "nxXVWFdwRqF8c4UPuZyhnD4hr7h1wHEQjbibSqjys3Y="
				    },
				    "delegator_shares": "1000000000000000000000.000000000000000000",
				    "description": {
				      "details": "",
				      "identity": "",
				      "moniker": "validator-3",
				      "security_contact": "",
				      "website": ""
				    },
				    "jailed": false,
				    "min_self_delegation": "1",
				    "operator_address": "unionvaloper1sw27dhxh32dz4klrllygy24g7tlse7latavr30",
				    "status": "BOND_STATUS_BONDED",
				    "tokens": "1000000000000000000000",
				    "unbonding_height": "0",
				    "unbonding_ids": [],
				    "unbonding_on_hold_ref_count": "0",
				    "unbonding_time": "1970-01-01T00:00:00Z"
				  },
				  {
				    "commission": {
				      "commission_rates": {
				        "max_change_rate": "0.010000000000000000",
				        "max_rate": "0.200000000000000000",
				        "rate": "0.100000000000000000"
				      },
				      "update_time": "2023-05-27T11:02:45.024436207Z"
				    },
				    "consensus_pubkey": {
				      "@type": "/cosmos.crypto.bn254.PubKey",
				      "key": "hqEVgoEMKkfBb2ASJ6XYc+foI6nV940grE6vIBJMFgY="
				    },
				    "delegator_shares": "1000000000000000000000.000000000000000000",
				    "description": {
				      "details": "",
				      "identity": "",
				      "moniker": "validator-1",
				      "security_contact": "",
				      "website": ""
				    },
				    "jailed": false,
				    "min_self_delegation": "1",
				    "operator_address": "unionvaloper1s36k93gu7x58zns0g4mrep8qgnr5fm4jrqddkv",
				    "status": "BOND_STATUS_BONDED",
				    "tokens": "1000000000000000000000",
				    "unbonding_height": "0",
				    "unbonding_ids": [],
				    "unbonding_on_hold_ref_count": "0",
				    "unbonding_time": "1970-01-01T00:00:00Z"
				  },
				  {
				    "commission": {
				      "commission_rates": {
				        "max_change_rate": "0.010000000000000000",
				        "max_rate": "0.200000000000000000",
				        "rate": "0.100000000000000000"
				      },
				      "update_time": "2023-05-27T11:02:45.024436207Z"
				    },
				    "consensus_pubkey": {
				      "@type": "/cosmos.crypto.bn254.PubKey",
				      "key": "l3xZBkj/4LfOxEKLGDhHXvdz5xd+jjgE+q/hniC9RW0="
				    },
				    "delegator_shares": "1000000000000000000000.000000000000000000",
				    "description": {
				      "details": "",
				      "identity": "",
				      "moniker": "validator-2",
				      "security_contact": "",
				      "website": ""
				    },
				    "jailed": false,
				    "min_self_delegation": "1",
				    "operator_address": "unionvaloper1ndm3ljzqwvc60uvxwkhczgrczc4jv2ll9etcn6",
				    "status": "BOND_STATUS_BONDED",
				    "tokens": "1000000000000000000000",
				    "unbonding_height": "0",
				    "unbonding_ids": [],
				    "unbonding_on_hold_ref_count": "0",
				    "unbonding_time": "1970-01-01T00:00:00Z"
				  },
				  {
				    "commission": {
				      "commission_rates": {
				        "max_change_rate": "0.010000000000000000",
				        "max_rate": "0.200000000000000000",
				        "rate": "0.100000000000000000"
				      },
				      "update_time": "2023-05-27T11:02:45.024436207Z"
				    },
				    "consensus_pubkey": {
				      "@type": "/cosmos.crypto.bn254.PubKey",
				      "key": "wI7T2nJFcFebw1jjemnMvtj1ARTY7qknDseziEE5DpU="
				    },
				    "delegator_shares": "1000000000000000000000.000000000000000000",
				    "description": {
				      "details": "",
				      "identity": "",
				      "moniker": "validator-0",
				      "security_contact": "",
				      "website": ""
				    },
				    "jailed": false,
				    "min_self_delegation": "1",
				    "operator_address": "unionvaloper14fldwd959h7glh2e3k45veuqfszvgm693pv868",
				    "status": "BOND_STATUS_BONDED",
				    "tokens": "1000000000000000000000",
				    "unbonding_height": "0",
				    "unbonding_ids": [],
				    "unbonding_on_hold_ref_count": "0",
				    "unbonding_time": "1970-01-01T00:00:00Z"
				  }
				]
			*/
			// cSpell:enable

			blockHash, err := hex.DecodeString("F8579B3A521F8F704B991FFB8CA9040124A2F7BF5FFB9A013AEA9E223370AC5F")
			if err != nil {
				return err
			}

			partSetHeaderHash, err := hex.DecodeString("ABE49229F06DC8E1F1BC3B564A6F06BEB5098B05F61971BF39297EAE4B1616AA")
			if err != nil {
				return err
			}

			vote := types.CanonicalVote{
				Type:   types.PrecommitType,
				Height: 19,
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

			validators := []*types.SimpleValidator{
				toValidator(decodeB64("wI7T2nJFcFebw1jjemnMvtj1ARTY7qknDseziEE5DpU=")), // cspell:disable-line
				toValidator(decodeB64("hqEVgoEMKkfBb2ASJ6XYc+foI6nV940grE6vIBJMFgY=")), // cspell:disable-line
				toValidator(decodeB64("l3xZBkj/4LfOxEKLGDhHXvdz5xd+jjgE+q/hniC9RW0=")), // cspell:disable-line
				toValidator(decodeB64("nxXVWFdwRqF8c4UPuZyhnD4hr7h1wHEQjbibSqjys3Y=")), // cspell:disable-line
			}

			trustedValidators := validators
			untrustedValidators := validators

			signatures := [][]byte{
				decodeB64("0eliDuI/J2J6TpPN4oM7dy57qjWAYG5Pyea/Peor/+4Fn1OSK0d/jv+E0nM+ZmccwyT/MWRuZZ8SrNZj+dIWJA=="), // cspell:disable-line
				decodeB64("5F0LHP8lXyeHrNJhBA1cuzeCP21O0oEkvYu0e/dhHs0u6+YFPdhViB6ZbmZM11nycIAcJ33+kmlsqA/j1dugoQ=="), // cspell:disable-line
				decodeB64("rH79tXDFY5xdlWMH5LTOgo2maGE66onP+WdbZS7DAs0ocbGahCII8gGd0aRlbuiYAXLDINrBil/qAxathWwBeg=="), // cspell:disable-line
				decodeB64("wXh774QmqRjg6Ang3VW1Bi6Z+fUyIrbCatBiArnfhaICoiIKz0IGUog+LyVtZBYpRw1TK0XoQxvUMQybzONp5g=="), // cspell:disable-line
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

			fmt.Printf("Gnark Proof: %X\n", res.Proof.Content)
			fmt.Printf("Public inputs: %X\n", res.Proof.PublicInputs)
			fmt.Printf("Trusted root: %X\n", res.TrustedValidatorSetRoot)
			fmt.Printf("Untrusted root: %X\n", res.UntrustedValidatorSetRoot)
			fmt.Printf("EVM compatible ZKP: %X\n", res.Proof.EvmProof)

			return nil
		},
	}
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
