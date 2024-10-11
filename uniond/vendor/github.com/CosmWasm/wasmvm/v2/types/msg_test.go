package types

import (
	"encoding/base64"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWasmMsgInstantiateSerialization(t *testing.T) {
	// no admin
	document := []byte(`{"instantiate":{"admin":null,"code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"label":"my instance"}}`)

	var msg WasmMsg
	err := json.Unmarshal(document, &msg)
	require.NoError(t, err)

	require.Nil(t, msg.Instantiate2)
	require.Nil(t, msg.Execute)
	require.Nil(t, msg.Migrate)
	require.Nil(t, msg.UpdateAdmin)
	require.Nil(t, msg.ClearAdmin)
	require.NotNil(t, msg.Instantiate)

	require.Equal(t, "", msg.Instantiate.Admin)
	require.Equal(t, uint64(7897), msg.Instantiate.CodeID)
	require.Equal(t, []byte(`{"claim":{}}`), msg.Instantiate.Msg)
	require.Equal(t, Array[Coin]{
		{"stones", "321"},
	}, msg.Instantiate.Funds)
	require.Equal(t, "my instance", msg.Instantiate.Label)

	// admin
	document2 := []byte(`{"instantiate":{"admin":"king","code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[],"label":"my instance"}}`)

	err2 := json.Unmarshal(document2, &msg)
	require.NoError(t, err2)

	require.Nil(t, msg.Instantiate2)
	require.Nil(t, msg.Execute)
	require.Nil(t, msg.Migrate)
	require.Nil(t, msg.UpdateAdmin)
	require.Nil(t, msg.ClearAdmin)
	require.NotNil(t, msg.Instantiate)

	require.Equal(t, "king", msg.Instantiate.Admin)
	require.Equal(t, uint64(7897), msg.Instantiate.CodeID)
	require.Equal(t, []byte(`{"claim":{}}`), msg.Instantiate.Msg)
	require.Equal(t, Array[Coin]{}, msg.Instantiate.Funds)
	require.Equal(t, "my instance", msg.Instantiate.Label)
}

func TestWasmMsgInstantiate2Serialization(t *testing.T) {
	document := []byte(`{"instantiate2":{"admin":null,"code_id":7897,"label":"my instance","msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"salt":"UkOVazhiwoo="}}`)

	var msg WasmMsg
	err := json.Unmarshal(document, &msg)
	require.NoError(t, err)

	require.Nil(t, msg.Instantiate)
	require.Nil(t, msg.Execute)
	require.Nil(t, msg.Migrate)
	require.Nil(t, msg.UpdateAdmin)
	require.Nil(t, msg.ClearAdmin)
	require.NotNil(t, msg.Instantiate2)

	require.Equal(t, "", msg.Instantiate2.Admin)
	require.Equal(t, uint64(7897), msg.Instantiate2.CodeID)
	require.Equal(t, []byte(`{"claim":{}}`), msg.Instantiate2.Msg)
	require.Equal(t, Array[Coin]{
		{"stones", "321"},
	}, msg.Instantiate2.Funds)
	require.Equal(t, "my instance", msg.Instantiate2.Label)
	require.Equal(t, []byte{0x52, 0x43, 0x95, 0x6b, 0x38, 0x62, 0xc2, 0x8a}, msg.Instantiate2.Salt)
}

func TestAnyMsgSerialization(t *testing.T) {
	expectedData, err := base64.StdEncoding.DecodeString("5yu/rQ+HrMcxH1zdga7P5hpGMLE=")
	require.NoError(t, err)

	// test backwards compatibility with old stargate variant
	document1 := []byte(`{"stargate":{"type_url":"/cosmos.foo.v1beta.MsgBar","value":"5yu/rQ+HrMcxH1zdga7P5hpGMLE="}}`)
	var res CosmosMsg
	err = json.Unmarshal(document1, &res)
	require.NoError(t, err)
	require.Equal(t, CosmosMsg{
		Any: &AnyMsg{
			TypeURL: "/cosmos.foo.v1beta.MsgBar",
			Value:   expectedData,
		},
	}, res)

	// test new any variant
	document2 := []byte(`{"any":{"type_url":"/cosmos.foo.v1beta.MsgBar","value":"5yu/rQ+HrMcxH1zdga7P5hpGMLE="}}`)
	var res2 CosmosMsg
	err = json.Unmarshal(document2, &res2)
	require.NoError(t, err)
	require.Equal(t, res, res2)

	// serializing should use the new any variant
	serialized, err := json.Marshal(res)
	require.NoError(t, err)
	require.Equal(t, document2, serialized)

	// test providing both variants is rejected
	document3 := []byte(`{
		"stargate":{"type_url":"/cosmos.foo.v1beta.MsgBar","value":"5yu/rQ+HrMcxH1zdga7P5hpGMLE="},
		"any":{"type_url":"/cosmos.foo.v1beta.MsgBar","value":"5yu/rQ+HrMcxH1zdga7P5hpGMLE="}
	}`)
	var res3 CosmosMsg
	err = json.Unmarshal(document3, &res3)
	require.Error(t, err)
}

func TestGovMsgVoteSerialization(t *testing.T) {
	oldDocument := []byte(`{"vote":{"proposal_id":4,"vote":"no_with_veto"}}`)

	var msg GovMsg
	err := json.Unmarshal(oldDocument, &msg)
	require.NoError(t, err)

	require.Nil(t, msg.VoteWeighted)
	require.NotNil(t, msg.Vote)

	require.Equal(t, uint64(4), msg.Vote.ProposalId)
	require.Equal(t, NoWithVeto, msg.Vote.Option)

	newDocument := []byte(`{"vote":{"proposal_id":4,"option":"no_with_veto"}}`)

	var msg2 GovMsg
	err2 := json.Unmarshal(newDocument, &msg2)
	require.NoError(t, err2)
	require.Equal(t, msg, msg2)
}

func TestGovMsgVoteWeightedSerialization(t *testing.T) {
	document := []byte(`{"vote_weighted":{"proposal_id":25,"options":[{"option":"yes","weight":"0.25"},{"option":"no","weight":"0.25"},{"option":"abstain","weight":"0.5"}]}}`)

	var msg GovMsg
	err := json.Unmarshal(document, &msg)
	require.NoError(t, err)

	require.Nil(t, msg.Vote)
	require.NotNil(t, msg.VoteWeighted)

	require.Equal(t, uint64(25), msg.VoteWeighted.ProposalId)
	require.Equal(t, []WeightedVoteOption{
		{Yes, "0.25"},
		{No, "0.25"},
		{Abstain, "0.5"},
	}, msg.VoteWeighted.Options)
}

func TestMsgFundCommunityPoolSerialization(t *testing.T) {
	document := []byte(`{"fund_community_pool":{"amount":[{"amount":"300","denom":"adenom"},{"amount":"400","denom":"bdenom"}]}}`)

	var msg DistributionMsg
	err := json.Unmarshal(document, &msg)
	require.NoError(t, err)

	require.Equal(t, Array[Coin]{{"adenom", "300"}, {"bdenom", "400"}}, msg.FundCommunityPool.Amount)
}
