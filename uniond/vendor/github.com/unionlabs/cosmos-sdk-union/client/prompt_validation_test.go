package client_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
)

func TestValidatePromptNotEmpty(t *testing.T) {
	require := require.New(t)

	require.NoError(client.ValidatePromptNotEmpty("foo"))
	require.ErrorContains(client.ValidatePromptNotEmpty(""), "input cannot be empty")
}

func TestValidatePromptURL(t *testing.T) {
	require := require.New(t)

	require.NoError(client.ValidatePromptURL("https://example.com"))
	require.ErrorContains(client.ValidatePromptURL("foo"), "invalid URL")
}

func TestValidatePromptAddress(t *testing.T) {
	require := require.New(t)

	require.NoError(client.ValidatePromptAddress("cosmos1huydeevpz37sd9snkgul6070mstupukw00xkw9"))
	require.NoError(client.ValidatePromptAddress("cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u2lcnj0"))
	require.NoError(client.ValidatePromptAddress("cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h"))
	require.ErrorContains(client.ValidatePromptAddress("foo"), "invalid address")
}

func TestValidatePromptCoins(t *testing.T) {
	require := require.New(t)

	require.NoError(client.ValidatePromptCoins("100stake"))
	require.ErrorContains(client.ValidatePromptCoins("foo"), "invalid coins")
}
