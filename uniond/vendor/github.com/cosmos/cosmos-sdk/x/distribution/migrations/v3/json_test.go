package v3_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/distribution"
	v3 "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v3"
	"github.com/cosmos/cosmos-sdk/x/distribution/types"
)

func TestMigrateJSON(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	clientCtx := client.Context{}.
		WithInterfaceRegistry(encodingConfig.InterfaceRegistry).
		WithTxConfig(encodingConfig.TxConfig).
		WithCodec(encodingConfig.Codec)

	distrGenState := types.DefaultGenesisState()

	oldDistrState := distrGenState
	oldDistrState.Params.BaseProposerReward = sdk.NewDecWithPrec(1, 2)
	oldDistrState.Params.BonusProposerReward = sdk.NewDecWithPrec(4, 2)

	migrated := v3.MigrateJSON(oldDistrState)
	require.Equal(t, migrated, distrGenState)

	bz, err := clientCtx.Codec.MarshalJSON(migrated)
	require.NoError(t, err)

	// Indent the JSON bz correctly.
	var jsonObj map[string]interface{}
	err = json.Unmarshal(bz, &jsonObj)
	require.NoError(t, err)
	indentedBz, err := json.MarshalIndent(jsonObj, "", "\t")
	require.NoError(t, err)

	expected := `{
	"delegator_starting_infos": [],
	"delegator_withdraw_infos": [],
	"fee_pool": {
		"community_pool": []
	},
	"outstanding_rewards": [],
	"params": {
		"base_proposer_reward": "0.000000000000000000",
		"bonus_proposer_reward": "0.000000000000000000",
		"community_tax": "0.020000000000000000",
		"withdraw_addr_enabled": true
	},
	"previous_proposer": "",
	"validator_accumulated_commissions": [],
	"validator_current_rewards": [],
	"validator_historical_rewards": [],
	"validator_slash_events": []
}`

	require.Equal(t, expected, string(indentedBz))
}
