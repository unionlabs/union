package prompt_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/client/v2/internal/prompt"
)

func TestValidatePromptNotEmpty(t *testing.T) {
	require := require.New(t)

	require.NoError(prompt.ValidatePromptNotEmpty("foo"))
	require.ErrorContains(prompt.ValidatePromptNotEmpty(""), "input cannot be empty")
}

func TestValidatePromptURL(t *testing.T) {
	require := require.New(t)

	require.NoError(prompt.ValidatePromptURL("https://example.com"))
	require.ErrorContains(prompt.ValidatePromptURL("foo"), "invalid URL")
}

func TestValidatePromptCoins(t *testing.T) {
	require := require.New(t)

	require.NoError(prompt.ValidatePromptCoins("100stake"))
	require.ErrorContains(prompt.ValidatePromptCoins("foo"), "invalid coins")
}
