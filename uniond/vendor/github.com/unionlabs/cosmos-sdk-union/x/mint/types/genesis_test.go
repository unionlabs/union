package types

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestGenesis(t *testing.T) {
	minter := DefaultInitialMinter()
	params := DefaultParams()

	gs := NewGenesisState(minter, params)
	err := ValidateGenesis(*gs)
	require.NoError(t, err)

	defaultGs := DefaultGenesisState()
	err = ValidateGenesis(*defaultGs)
	require.NoError(t, err)
	require.Equal(t, gs, defaultGs)
}
