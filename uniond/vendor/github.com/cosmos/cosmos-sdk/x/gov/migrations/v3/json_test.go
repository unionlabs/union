package v3_test

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/gov"
	v3 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v3"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

var voter = sdk.MustAccAddressFromBech32("cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh")

func TestMigrateJSON(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig(gov.AppModuleBasic{})
	clientCtx := client.Context{}.
		WithInterfaceRegistry(encodingConfig.InterfaceRegistry).
		WithTxConfig(encodingConfig.TxConfig).
		WithCodec(encodingConfig.Codec)

	govGenState := v1beta1.DefaultGenesisState()
	propTime := time.Unix(1e9, 0)
	contentAny, err := codectypes.NewAnyWithValue(v1beta1.NewTextProposal("my title", "my desc").(proto.Message))
	require.NoError(t, err)
	govGenState.Proposals = v1beta1.Proposals{
		v1beta1.Proposal{
			ProposalId:       1,
			Content:          contentAny,
			SubmitTime:       propTime,
			DepositEndTime:   propTime,
			VotingStartTime:  propTime,
			VotingEndTime:    propTime,
			Status:           v1beta1.StatusDepositPeriod,
			FinalTallyResult: v1beta1.EmptyTallyResult(),
			TotalDeposit:     sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(123))),
		},
	}
	govGenState.Votes = v1beta1.Votes{
		v1beta1.Vote{ProposalId: 1, Voter: voter.String(), Option: v1beta1.OptionAbstain},
		v1beta1.Vote{ProposalId: 2, Voter: voter.String(), Options: v1beta1.NewNonSplitVoteOption(v1beta1.OptionNo)},
	}

	migrated, err := v3.MigrateJSON(govGenState)
	require.NoError(t, err)

	// Make sure the migrated proposal's Msg signer is the gov acct.
	require.Equal(t,
		authtypes.NewModuleAddress(types.ModuleName).String(),
		migrated.Proposals[0].Messages[0].GetCachedValue().(*v1.MsgExecLegacyContent).Authority,
	)

	bz, err := clientCtx.Codec.MarshalJSON(migrated)
	require.NoError(t, err)

	// Indent the JSON bz correctly.
	var jsonObj map[string]interface{}
	err = json.Unmarshal(bz, &jsonObj)
	require.NoError(t, err)
	indentedBz, err := json.MarshalIndent(jsonObj, "", "\t")
	require.NoError(t, err)

	// Make sure about:
	// - Proposals use MsgExecLegacyContent
	expected := `{
	"deposit_params": {
		"max_deposit_period": "172800s",
		"min_deposit": [
			{
				"amount": "10000000",
				"denom": "stake"
			}
		]
	},
	"deposits": [],
	"params": null,
	"proposals": [
		{
			"deposit_end_time": "2001-09-09T01:46:40Z",
			"final_tally_result": {
				"abstain_count": "0",
				"no_count": "0",
				"no_with_veto_count": "0",
				"yes_count": "0"
			},
			"id": "1",
			"messages": [
				{
					"@type": "/cosmos.gov.v1.MsgExecLegacyContent",
					"authority": "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn",
					"content": {
						"@type": "/cosmos.gov.v1beta1.TextProposal",
						"description": "my desc",
						"title": "my title"
					}
				}
			],
			"metadata": "",
			"proposer": "",
			"status": "PROPOSAL_STATUS_DEPOSIT_PERIOD",
			"submit_time": "2001-09-09T01:46:40Z",
			"summary": "my desc",
			"title": "my title",
			"total_deposit": [
				{
					"amount": "123",
					"denom": "stake"
				}
			],
			"voting_end_time": "2001-09-09T01:46:40Z",
			"voting_start_time": "2001-09-09T01:46:40Z"
		}
	],
	"starting_proposal_id": "1",
	"tally_params": {
		"quorum": "0.334000000000000000",
		"threshold": "0.500000000000000000",
		"veto_threshold": "0.334000000000000000"
	},
	"votes": [
		{
			"metadata": "",
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
			"metadata": "",
			"options": [
				{
					"option": "VOTE_OPTION_NO",
					"weight": "1.000000000000000000"
				}
			],
			"proposal_id": "2",
			"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"
		}
	],
	"voting_params": {
		"voting_period": "172800s"
	}
}`

	require.Equal(t, expected, string(indentedBz))
}
