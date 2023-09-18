package v2_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	v2 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v2"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

func TestMigrateJSON(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig()
	clientCtx := client.Context{}.
		WithInterfaceRegistry(encodingConfig.InterfaceRegistry).
		WithTxConfig(encodingConfig.TxConfig).
		WithCodec(encodingConfig.Codec)

	voter, err := sdk.AccAddressFromBech32("cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh")
	require.NoError(t, err)
	govGenState := &v1beta1.GenesisState{
		Votes: v1beta1.Votes{
			v1beta1.Vote{ProposalId: 1, Voter: voter.String(), Option: v1beta1.OptionAbstain},
			v1beta1.Vote{ProposalId: 2, Voter: voter.String(), Option: v1beta1.OptionEmpty},
			v1beta1.Vote{ProposalId: 3, Voter: voter.String(), Option: v1beta1.OptionNo},
			v1beta1.Vote{ProposalId: 4, Voter: voter.String(), Option: v1beta1.OptionNoWithVeto},
			v1beta1.Vote{ProposalId: 5, Voter: voter.String(), Option: v1beta1.OptionYes},
		},
	}

	migrated := v2.MigrateJSON(govGenState)

	bz, err := clientCtx.Codec.MarshalJSON(migrated)
	require.NoError(t, err)

	// Indent the JSON bz correctly.
	var jsonObj map[string]interface{}
	err = json.Unmarshal(bz, &jsonObj)
	require.NoError(t, err)
	indentedBz, err := json.MarshalIndent(jsonObj, "", "\t")
	require.NoError(t, err)

	// Make sure about:
	// - Votes are all ADR-037 weighted votes with weight 1.
	expected := `{
	"deposit_params": {
		"max_deposit_period": "0s",
		"min_deposit": []
	},
	"deposits": [],
	"proposals": [],
	"starting_proposal_id": "0",
	"tally_params": {
		"quorum": "0",
		"threshold": "0",
		"veto_threshold": "0"
	},
	"votes": [
		{
			"option": "VOTE_OPTION_UNSPECIFIED",
			"options": [
				{
					"option": "VOTE_OPTION_ABSTAIN",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "1",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		},
		{
			"option": "VOTE_OPTION_UNSPECIFIED",
			"options": [
				{
					"option": "VOTE_OPTION_UNSPECIFIED",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "2",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		},
		{
			"option": "VOTE_OPTION_UNSPECIFIED",
			"options": [
				{
					"option": "VOTE_OPTION_NO",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "3",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		},
		{
			"option": "VOTE_OPTION_UNSPECIFIED",
			"options": [
				{
					"option": "VOTE_OPTION_NO_WITH_VETO",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "4",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		},
		{
			"option": "VOTE_OPTION_UNSPECIFIED",
			"options": [
				{
					"option": "VOTE_OPTION_YES",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "5",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		}
	],
	"voting_params": {
		"voting_period": "0s"
	}
}`

	require.Equal(t, expected, string(indentedBz))
}
