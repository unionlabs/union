package cli_test

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/x/staking"

	"github.com/cosmos/cosmos-sdk/client"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/types/module"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
)

// An example exported genesis file from a 0.37 chain. Note that evidence
// parameters only contains `max_age`.
var v037Exported = `{
	"app_hash": "",
	"app_state": {},
	"chain_id": "test",
	"consensus_params": {
		"block": {
		"max_bytes": "22020096",
		"max_gas": "-1",
		"time_iota_ms": "1000"
		},
		"evidence": { "max_age": "100000" },
		"validator": { "pub_key_types": ["ed25519"] }
	},
	"genesis_time": "2020-09-29T20:16:29.172362037Z",
	"validators": []
}`

func TestValidateGenesis(t *testing.T) {
	cdc := testutilmod.MakeTestEncodingConfig(codectestutil.CodecOptions{}, genutil.AppModule{}).Codec
	testCases := []struct {
		name      string
		genesis   string
		expErrStr string
		genMM     *module.Manager
	}{
		{
			"invalid json",
			`{"app_state": {x,}}`,
			"error at offset 16: invalid character",
			module.NewManagerFromMap(nil),
		},
		{
			"invalid: missing module config in app_state",
			func() string {
				bz, err := os.ReadFile("../../types/testdata/app_genesis.json")
				require.NoError(t, err)

				return string(bz)
			}(),
			"section is missing in the app_state",
			module.NewManagerFromMap(map[string]appmodulev2.AppModule{
				"custommod": staking.NewAppModule(cdc, nil),
			}),
		},
		{
			"exported 0.37 genesis file",
			v037Exported,
			"make sure that you have correctly migrated all CometBFT consensus params",
			module.NewManagerFromMap(nil),
		},
		{
			"valid 0.50 genesis file",
			func() string {
				bz, err := os.ReadFile("../../types/testdata/app_genesis.json")
				require.NoError(t, err)

				return string(bz)
			}(),
			"",
			module.NewManagerFromMap(nil),
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			genesisFile := testutil.WriteToNewTempFile(t, tc.genesis)
			_, err := clitestutil.ExecTestCLICmd(client.Context{}, cli.ValidateGenesisCmd(tc.genMM), []string{genesisFile.Name()})
			if tc.expErrStr != "" {
				require.Contains(t, err.Error(), tc.expErrStr)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
