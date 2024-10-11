package cli

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestParseCLIProposal(t *testing.T) {
	data := []byte(`{
			"group_policy_address": "cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf",
			"messages": [
			  {
				"@type": "/cosmos.bank.v1beta1.MsgSend",
				"from_address": "cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf",
				"to_address": "cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf",
				"amount":[{"denom": "stake","amount": "10"}]
			  }
			],
			"metadata": "4pIMOgIGx1vZGU=",
			"proposers": ["cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf"],
			"title": "test",
			"summary": "test summary"
		}`)

	result, err := parseCLIProposal(data)
	require.NoError(t, err)
	require.Equal(t, result.GroupPolicyAddress, "cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf")
	require.NotEmpty(t, result.Metadata)
	require.Equal(t, result.Metadata, "4pIMOgIGx1vZGU=")
	require.Equal(t, result.Proposers, []string{"cosmos15r295x4994egvckteam9skazy9kvfvzpak4naf"})
	require.Equal(t, result.Title, "test")
	require.Equal(t, result.Summary, "test summary")
}
