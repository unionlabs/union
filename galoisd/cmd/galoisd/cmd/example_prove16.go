package cmd

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	provergrpc "galois/grpc/api/v1"
	"log"
	"math/big"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
)

// Example call to the prover `Prove` endpoint using hardcoded values dumped from a local devnet.
// The sole purpose of this command is to see a live example and understand how to interact with the prover.
func ExampleProve16Cmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:  "example-prove16 [uri]",
		Args: cobra.ExactArgs(1),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
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
				{
				  "block_id": {
				    "hash": "3CECE4F2364BFA92040E6217FE7D43DA93F09F02FB5269177214BCDEF05298E1",
				    "parts": {
				      "total": 1,
				      "hash": "749946B107F3BE49728999F84FD7E16C4BC028C6E6538F8A287DE04203AC10A1"
				    }
				  },
				  "block": {
				    "header": {
				      "version": {
				        "block": "11"
				      },
				      "chain_id": "union-devnet-1",
				      "height": "1",
				      "time": "2023-09-18T19:40:37.830641577Z",
				      "last_block_id": {
				        "hash": "",
				        "parts": {
				          "total": 0,
				          "hash": ""
				        }
				      },
				      "last_commit_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "validators_hash": "D7622F989219A191951500F2829907D31B1F05FC200D6250392B6FB3060DA6E8",
				      "next_validators_hash": "D7622F989219A191951500F2829907D31B1F05FC200D6250392B6FB3060DA6E8",
				      "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
				      "app_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "last_results_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "proposer_address": "00B978986867D21B0A93DACD62A7EDD3D913F3D9"
				    },
				    "data": {
				      "txs": null
				    },
				    "evidence": {
				      "evidence": null
				    },
				    "last_commit": {
				      "height": "0",
				      "round": 0,
				      "block_id": {
				        "hash": "",
				        "parts": {
				          "total": 0,
				          "hash": ""
				        }
				      },
				      "signatures": null
				    }
				  }
				}


				{
				  "block_id": {
				    "hash": "0E9922F487B1F1AD2821680748DB17AF5E5A5A396B6D4147C1030A377DDEEC97",
				    "parts": {
				      "total": 1,
				      "hash": "EBE618A59041FA8455EC79659205A630D5F8C7A52995A87CD31A49462E367002"
				    }
				  },
				  "block": {
				    "header": {
				      "version": {
				        "block": "11"
				      },
				      "chain_id": "union-devnet-1",
				      "height": "2",
				      "time": "2023-09-19T10:35:41.311344117Z",
				      "last_block_id": {
				        "hash": "3CECE4F2364BFA92040E6217FE7D43DA93F09F02FB5269177214BCDEF05298E1",
				        "parts": {
				          "total": 1,
				          "hash": "749946B107F3BE49728999F84FD7E16C4BC028C6E6538F8A287DE04203AC10A1"
				        }
				      },
				      "last_commit_hash": "723767B95D280F25099CC97FE44F05CEE0B1264A5531694044F11007EB942314",
				      "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "validators_hash": "D7622F989219A191951500F2829907D31B1F05FC200D6250392B6FB3060DA6E8",
				      "next_validators_hash": "D7622F989219A191951500F2829907D31B1F05FC200D6250392B6FB3060DA6E8",
				      "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
				      "app_hash": "3D1B75075A78904B83905F434461DDC7F047E3C1CA789433F7603F782CB2135E",
				      "last_results_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
				      "proposer_address": "0C26B59A0B65D191A86D969D5D3F2DC40DD9C977"
				    },
				    "data": {
				      "txs": null
				    },
				    "evidence": {
				      "evidence": null
				    },
				    "last_commit": {
				      "height": "1",
				      "round": 0,
				      "block_id": {
				        "hash": "3CECE4F2364BFA92040E6217FE7D43DA93F09F02FB5269177214BCDEF05298E1",
				        "parts": {
				          "total": 1,
				          "hash": "749946B107F3BE49728999F84FD7E16C4BC028C6E6538F8A287DE04203AC10A1"
				        }
				      },
				      "signatures": [
				        {
				          "block_id_flag": 2,
				          "validator_address": "00B978986867D21B0A93DACD62A7EDD3D913F3D9",
				          "timestamp": "2023-09-19T10:35:41.311468435Z",
				          "signature": "lBIMmBoSo0jcJfFtwpZVzpltsZeBPYk+WLpaokCHoYUgU4d9n2GtSvfOBG7XQKf5DZNaKM+7JvnAuFrKiM8qeA=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "0C26B59A0B65D191A86D969D5D3F2DC40DD9C977",
				          "timestamp": "2023-09-19T10:35:41.317548709Z",
				          "signature": "jTFVLTwKgONfRq/9HHqRTYFIYUzQo/Qxs2UrhtRDLYcrQiarjXecINudtZ1FXTvHXr6irOimrrqixZ5Hq2BVmw=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "196D6009588DA28CF40039C957A53B08104723F9",
				          "timestamp": "2023-09-19T10:35:41.317615216Z",
				          "signature": "x9uPPVyOrwEtd+2vwYu2k90rr2qtDnXqyBo1Twl66hYQgeY42nuuRdAIgB1jTxjPFxhw9/TzzrkG1gh1gOOGlw=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "19963640A11B2EC4F08E5B5000CD30D8641AA569",
				          "timestamp": "2023-09-19T10:35:41.311339439Z",
				          "signature": "1W3/De1lakrcHWK/M7y2ikKNbX1925MYUE+X28DG1S0b6gMY50omYl7V/z1pBgGmbSMLRjyGYCWLXIGtthNZhw=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "245DC189905D4F57D26EAE5120377707ED56ECA0",
				          "timestamp": "2023-09-19T10:35:41.317560283Z",
				          "signature": "ihNyjyK1No97Nz0/jsc1oDueaIfMwEAYpt9aZZyW7zMYxpq17D8mWAjXpDOg015W09ZNK5d03a1HRotNMcLXMw=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "257BC7E3F7BAD2C5EB1A11318003FE6CB5A52BE5",
				          "timestamp": "2023-09-19T10:35:41.303518751Z",
				          "signature": "w6u2qCzMNILalSua6Jr9c3R23c35hTH7ppuCpVho9p4YkStg3wuvVrXmrtpth3bJjua8SPs6ZtcBrigwrLj0Rg=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "2DCE4E05E127F97B23F8099E4D1DBDEB7587DC8B",
				          "timestamp": "2023-09-19T10:35:41.311422879Z",
				          "signature": "n8OQzqpYl5vvRFhBSkNsTHHi+qAH6Nw4xMxpHqRb6rIj1NM9BCDg9uW7tt8qRmJb2V6kqL6OcPq8gsISF0X+LA=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "36E1644D94064ED11521041E9138A0D1CCA9C31C",
				          "timestamp": "2023-09-19T10:35:41.508361788Z",
				          "signature": "7l/Ro9QII39SX1Pzjjs1LX0pMXB9m6GFHV+Rf6gRhuoBXcoT9CClA8ItHokeaO7ijCzI2CEUICzgFysaE2nKsw=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "4CE57693C82B50F830731DAB14FA759327762456",
				          "timestamp": "2023-09-19T10:35:41.303517231Z",
				          "signature": "q8n1IEwXBf7PtsexuFKBKxonbzTGJrQINwGtjmA3hYUFx8+ejOGjDwB6KEjgVodGV6A7KKLGoArVWZKLcMvnoQ=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "503662ECE25CF73487F100EDD02D775EEFEFCD0E",
				          "timestamp": "2023-09-19T10:35:41.305274844Z",
				          "signature": "zA5EYZUYXU7ApTRRVwzlXsGubbOr4WsICo+d3ikoo3YkBBuPqtQQ01Tx91WOqB34TOVK8dvBIgGwR3KoDqMJ7w=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "94FEEE87198F3AD180733D7B3C10FD2F150C3E62",
				          "timestamp": "2023-09-19T10:35:41.311344117Z",
				          "signature": "5suzZsuuzvE8iCESiiZ30RalvUljUFZwZgtJtkpxKUkmElVpVCxzrDTi7MpqzhuSmqqIHSzwutOXbji8SH2GOQ=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "C5AFE5C76192ACD502AB9D9D88CBC9C75597C411",
				          "timestamp": "2023-09-19T10:35:41.4058391Z",
				          "signature": "yL99BSFXDR5DTGJjlIp0FcwWnIYDVOYbHTHDtL99hzIZsY8Cl6ZNRAfJbCUoUlJdNx4Tg6EXi8Itfkypkm/6kQ=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "D619CD0E08ED87F92BB7BC2171071CFAC7BE1A4B",
				          "timestamp": "2023-09-19T10:35:41.303519341Z",
				          "signature": "7fFHIB6pG65TQCxxcj9bICHrZ9O1iPmKW7QfQL4902AjRiChsSOdc5UsccdwayzjI6Jg0rDk+Omg2PWdF391dQ=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "DFA8398671155E09BBA8244C2D7C295F980F4A2A",
				          "timestamp": "2023-09-19T10:35:41.514480165Z",
				          "signature": "rIEUr8Gt+CWiIJI+rDmuuwsIWk/5RQAJ3lOpd+Lpux8E7R0IafXV1DsjrWdPOzeBbbT9b/5reHvZAJIc2zCVsg=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "EFB1D8B3A56D97F2AB24AC5F0B04F48535F74DA9",
				          "timestamp": "2023-09-19T10:35:41.213336321Z",
				          "signature": "kY52piLql5OBdkGqbJ60ruJOlUZ91OCXAskx8YyB0XkKa+WcJAiHGsKyoe9gOLhnv3k5cFOJSkCJ6FeE12/e1w=="
				        },
				        {
				          "block_id_flag": 2,
				          "validator_address": "F3A5615BEB78B0D297FE37254433D7C0C367158A",
				          "timestamp": "2023-09-19T10:35:41.311341594Z",
				          "signature": "rsnq7gcKdEYtPi9NL01QNErl6SO12VIqliiYu3S0cLwL3AFe/6q6a6G1KyRj5gWmql//fQI5Un3NtrVqIMKhng=="
				        }
				      ]
				    }
				  }
				}


				{
				  "pagination": {
				    "next_key": null,
				    "total": "0"
				  },
				  "validators": [
				    {
				      "commission": {
				        "commission_rates": {
				          "max_change_rate": "0.010000000000000000",
				          "max_rate": "0.200000000000000000",
				          "rate": "0.100000000000000000"
				        },
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "ley/CHKtnWvu5aVxbfU9jgcWRkWV+j2bSmYNqgK8nAY="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-12",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1xy8p8c8v6lkjflp4svsfe5jwqny9f36e3mtgyp",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "mMjsEy9PZLJLGURHF1KXRlpgdS38eCbztA/wYUUuO+w="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-5",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1fhw5vm4mxxv3t3nftp2y5g5r0xyy270eg70rgs",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "gAM5W+LDW4eFkZw2n3WDmCCd565WTDd5E7L2L7yzOW8="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-10",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper12j6c882v773tjcyqrd7z086yttfjz0hguu898t",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "j2f0mRA51Iz9VQNu131t/7V0a4k19azWsyiimmUPkoU="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-15",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1vfugh97026jcqmt0pq22kmfveynyq2m49pxwly",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "p7jGEk8mMgsCp1KPonEoJoo48AHxIj7csAU61OlEEhs="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-6",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1w0lsj30p8wcp40tpwqxspywaee92l4y0mmjpck",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "k/tDqzvtGyDwEI6mUX9qpL+pbP+GeYPpZC5XQiSU12Q="
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "7ZAoR4jcMmiqojusF0tkv/Q27wYPXAVieQWEzvUsW9g="
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "wV8abg4Z83e0/NFv8E2yoj07lzSmxZGsHfi7NkEbKX0="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-9",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1sukjrc6cr4ag33h3w25rfsl834q22zp06d7h6p",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "wUIdx4VSAyjBSD7KGxEHhE19IczlZFFEmNFf2dIzklM="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-8",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper13t0nnfuv9vt2e95ss0yhuqvvc40agn68x6hwtj",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "jZiv55ih+4mChYy+Jm3M/u/MA5ZK530uMkgqgBcQnfo="
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "ht8ttsjmD9S+0ZQKLjKp9iUSnhOlFWAjqfGDnoCjHfg="
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
				    },
				    {
				      "commission": {
				        "commission_rates": {
				          "max_change_rate": "0.010000000000000000",
				          "max_rate": "0.200000000000000000",
				          "rate": "0.100000000000000000"
				        },
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "nI931rYm57np2qqZLxwGLZYQkrXiMUPckaxneyZss98="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-4",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1hmrw08ttdkkmv8kt9dsjcfap8thqkc9zwugk3k",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "4KRIncS9hK37sD0cHGDcFI0EEu8T7I/JFEiGKVefx3U="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-13",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1csrs76kwlna8j4demjpyhe0nknvv7hz3ket7r0",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "rW4uEup6ZPtH6RHeCBltigC7P6y+mTF0XSkAu8zfXnk="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-7",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1cnwslh2ghgadxz65x9yjcn8x9sn3hlxr3w46ah",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "hHtgMOdYMU8muqxX5PrdjYWRsIZ9cwezbE2gz5vVFpo="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-11",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper1mktwvdckj02j0nd5mzey6u7wqfd5ner3erxpuj",
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
				        "update_time": "2023-09-18T19:40:37.830641577Z"
				      },
				      "consensus_pubkey": {
				        "@type": "/cosmos.crypto.bn254.PubKey",
				        "key": "mUhqAu7NxIjEALZXq2X0jeYW0wTcaccgsrnxbusUKCQ="
				      },
				      "delegator_shares": "1000000000000000000000.000000000000000000",
				      "description": {
				        "details": "",
				        "identity": "",
				        "moniker": "validator-14",
				        "security_contact": "",
				        "website": ""
				      },
				      "jailed": false,
				      "min_self_delegation": "1",
				      "operator_address": "unionvaloper173t2zxc9ndft4gvpafm3aqnsj57sxfk8wmu3za",
				      "status": "BOND_STATUS_BONDED",
				      "tokens": "1000000000000000000000",
				      "unbonding_height": "0",
				      "unbonding_ids": [],
				      "unbonding_on_hold_ref_count": "0",
				      "unbonding_time": "1970-01-01T00:00:00Z"
				    }
				  ]
				}
			*/
			// cSpell:enable

			blockHash, err := hex.DecodeString("3CECE4F2364BFA92040E6217FE7D43DA93F09F02FB5269177214BCDEF05298E1")
			if err != nil {
				log.Fatal(err)
			}

			partSetHeaderHash, err := hex.DecodeString("749946B107F3BE49728999F84FD7E16C4BC028C6E6538F8A287DE04203AC10A1")
			if err != nil {
				log.Fatal(err)
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
				ChainID: "union-devnet-1",
			}

			validators := []*types.SimpleValidator{
				toValidator(decodeB64("ht8ttsjmD9S+0ZQKLjKp9iUSnhOlFWAjqfGDnoCjHfg=")),
				toValidator(decodeB64("7ZAoR4jcMmiqojusF0tkv/Q27wYPXAVieQWEzvUsW9g=")),
				toValidator(decodeB64("jZiv55ih+4mChYy+Jm3M/u/MA5ZK530uMkgqgBcQnfo=")),
				toValidator(decodeB64("k/tDqzvtGyDwEI6mUX9qpL+pbP+GeYPpZC5XQiSU12Q=")),
				toValidator(decodeB64("nI931rYm57np2qqZLxwGLZYQkrXiMUPckaxneyZss98=")),
				toValidator(decodeB64("mMjsEy9PZLJLGURHF1KXRlpgdS38eCbztA/wYUUuO+w=")),
				toValidator(decodeB64("p7jGEk8mMgsCp1KPonEoJoo48AHxIj7csAU61OlEEhs=")),
				toValidator(decodeB64("rW4uEup6ZPtH6RHeCBltigC7P6y+mTF0XSkAu8zfXnk=")),
				toValidator(decodeB64("wUIdx4VSAyjBSD7KGxEHhE19IczlZFFEmNFf2dIzklM=")),
				toValidator(decodeB64("wV8abg4Z83e0/NFv8E2yoj07lzSmxZGsHfi7NkEbKX0=")),
				toValidator(decodeB64("gAM5W+LDW4eFkZw2n3WDmCCd565WTDd5E7L2L7yzOW8=")),
				toValidator(decodeB64("hHtgMOdYMU8muqxX5PrdjYWRsIZ9cwezbE2gz5vVFpo=")),
				toValidator(decodeB64("ley/CHKtnWvu5aVxbfU9jgcWRkWV+j2bSmYNqgK8nAY=")),
				toValidator(decodeB64("4KRIncS9hK37sD0cHGDcFI0EEu8T7I/JFEiGKVefx3U=")),
				toValidator(decodeB64("mUhqAu7NxIjEALZXq2X0jeYW0wTcaccgsrnxbusUKCQ=")),
				toValidator(decodeB64("j2f0mRA51Iz9VQNu131t/7V0a4k19azWsyiimmUPkoU=")),
			}

			trustedValidators := validators
			untrustedValidators := validators

			signatures := [][]byte{
				decodeB64("lBIMmBoSo0jcJfFtwpZVzpltsZeBPYk+WLpaokCHoYUgU4d9n2GtSvfOBG7XQKf5DZNaKM+7JvnAuFrKiM8qeA=="),
				decodeB64("jTFVLTwKgONfRq/9HHqRTYFIYUzQo/Qxs2UrhtRDLYcrQiarjXecINudtZ1FXTvHXr6irOimrrqixZ5Hq2BVmw=="),
				decodeB64("x9uPPVyOrwEtd+2vwYu2k90rr2qtDnXqyBo1Twl66hYQgeY42nuuRdAIgB1jTxjPFxhw9/TzzrkG1gh1gOOGlw=="),
				decodeB64("1W3/De1lakrcHWK/M7y2ikKNbX1925MYUE+X28DG1S0b6gMY50omYl7V/z1pBgGmbSMLRjyGYCWLXIGtthNZhw=="),
				decodeB64("ihNyjyK1No97Nz0/jsc1oDueaIfMwEAYpt9aZZyW7zMYxpq17D8mWAjXpDOg015W09ZNK5d03a1HRotNMcLXMw=="),
				decodeB64("w6u2qCzMNILalSua6Jr9c3R23c35hTH7ppuCpVho9p4YkStg3wuvVrXmrtpth3bJjua8SPs6ZtcBrigwrLj0Rg=="),
				decodeB64("n8OQzqpYl5vvRFhBSkNsTHHi+qAH6Nw4xMxpHqRb6rIj1NM9BCDg9uW7tt8qRmJb2V6kqL6OcPq8gsISF0X+LA=="),
				decodeB64("7l/Ro9QII39SX1Pzjjs1LX0pMXB9m6GFHV+Rf6gRhuoBXcoT9CClA8ItHokeaO7ijCzI2CEUICzgFysaE2nKsw=="),
				decodeB64("q8n1IEwXBf7PtsexuFKBKxonbzTGJrQINwGtjmA3hYUFx8+ejOGjDwB6KEjgVodGV6A7KKLGoArVWZKLcMvnoQ=="),
				decodeB64("zA5EYZUYXU7ApTRRVwzlXsGubbOr4WsICo+d3ikoo3YkBBuPqtQQ01Tx91WOqB34TOVK8dvBIgGwR3KoDqMJ7w=="),
				decodeB64("5suzZsuuzvE8iCESiiZ30RalvUljUFZwZgtJtkpxKUkmElVpVCxzrDTi7MpqzhuSmqqIHSzwutOXbji8SH2GOQ=="),
				decodeB64("yL99BSFXDR5DTGJjlIp0FcwWnIYDVOYbHTHDtL99hzIZsY8Cl6ZNRAfJbCUoUlJdNx4Tg6EXi8Itfkypkm/6kQ=="),
				decodeB64("7fFHIB6pG65TQCxxcj9bICHrZ9O1iPmKW7QfQL4902AjRiChsSOdc5UsccdwayzjI6Jg0rDk+Omg2PWdF391dQ=="),
				decodeB64("rIEUr8Gt+CWiIJI+rDmuuwsIWk/5RQAJ3lOpd+Lpux8E7R0IafXV1DsjrWdPOzeBbbT9b/5reHvZAJIc2zCVsg=="),
				decodeB64("kY52piLql5OBdkGqbJ60ruJOlUZ91OCXAskx8YyB0XkKa+WcJAiHGsKyoe9gOLhnv3k5cFOJSkCJ6FeE12/e1w=="),
				decodeB64("rsnq7gcKdEYtPi9NL01QNErl6SO12VIqliiYu3S0cLwL3AFe/6q6a6G1KyRj5gWmql//fQI5Un3NtrVqIMKhng=="),
			}

			trustedSignatures := signatures
			untrustedSignatures := signatures

			var bitmap big.Int
			for i := 0; i < 16; i++ {
				bitmap.SetBit(&bitmap, i, 1)
			}

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
				log.Fatal(err)
			}

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
