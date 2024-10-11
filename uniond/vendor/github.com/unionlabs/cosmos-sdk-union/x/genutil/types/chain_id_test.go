package types_test

import (
	_ "embed"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

//go:embed testdata/parse_chain_id.json
var BenchmarkGenesis string

func TestParseChainIDFromGenesis(t *testing.T) {
	testCases := []struct {
		name       string
		json       string
		expChainID string
		expError   string
	}{
		{
			"success",
			`{
				"state": {
				  "accounts": {
					"a": {}
				  }
				},
				"chain_id": "test-chain-id"
			}`,
			"test-chain-id",
			"",
		},
		{
			"nested",
			`{
				"state": {
				  "accounts": {
					"a": {}
				  },
				  "chain_id": "test-chain-id"
				}
			}`,
			"",
			"missing chain-id in genesis file",
		},
		{
			"not exist",
			`{
				"state": {
				  "accounts": {
					"a": {}
				  }
				},
				"chain-id": "test-chain-id"
			}`,
			"",
			"missing chain-id in genesis file",
		},
		{
			"invalid type",
			`{
				"chain-id": 1,
			}`,
			"",
			"invalid character '}' looking for beginning of object key string",
		},
		{
			"invalid json",
			`[ " ': }`,
			"",
			"expected {, got [",
		},
		{
			"empty chain_id",
			`{"chain_id": ""}`,
			"",
			"genesis doc must include non-empty chain_id",
		},
		{
			"whitespace chain_id",
			`{"chain_id": "   "}`,
			"",
			"genesis doc must include non-empty chain_id",
		},
		{
			"chain_id too long",
			`{"chain_id": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}`,
			"",
			"chain_id in genesis doc is too long",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			chain_id, err := types.ParseChainIDFromGenesis(strings.NewReader(tc.json))
			if tc.expChainID == "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expError)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.expChainID, chain_id)
			}
		})
	}
}

func BenchmarkParseChainID(b *testing.B) {
	expChainID := "cronosmainnet_25-1"
	b.ReportAllocs()
	b.Run("new", func(b *testing.B) {
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			chainId, err := types.ParseChainIDFromGenesis(strings.NewReader(BenchmarkGenesis))
			require.NoError(b, err)
			require.Equal(b, expChainID, chainId)
		}
	})

	b.Run("old", func(b *testing.B) {
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			doc, err := types.AppGenesisFromReader(strings.NewReader(BenchmarkGenesis))
			require.NoError(b, err)
			require.Equal(b, expChainID, doc.ChainID)
		}
	})
}
