package ibctesting_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"cosmossdk.io/x/staking/types"

	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestChangeValSet(t *testing.T) {
	coord := ibctesting.NewCoordinator(t, 2)
	chainA := coord.GetChain(ibctesting.GetChainID(1))
	chainB := coord.GetChain(ibctesting.GetChainID(2))

	path := ibctesting.NewPath(chainA, chainB)
	coord.Setup(path)

	amount, ok := sdkmath.NewIntFromString("10000000000000000000")
	require.True(t, ok)
	amount2, ok := sdkmath.NewIntFromString("30000000000000000000")
	require.True(t, ok)

	val, err := chainA.GetSimApp().StakingKeeper.GetValidators(chainA.GetContext(), 4)
	require.NoError(t, err)

	chainA.GetSimApp().StakingKeeper.Delegate(chainA.GetContext(), chainA.SenderAccounts[1].SenderAccount.GetAddress(), //nolint:errcheck // ignore error for test
		amount, types.Unbonded, val[1], true)
	chainA.GetSimApp().StakingKeeper.Delegate(chainA.GetContext(), chainA.SenderAccounts[3].SenderAccount.GetAddress(), //nolint:errcheck // ignore error for test
		amount2, types.Unbonded, val[3], true)

	coord.CommitBlock(chainA)

	// verify that update clients works even after validator update goes into effect
	err = path.EndpointB.UpdateClient()
	require.NoError(t, err)
	err = path.EndpointB.UpdateClient()
	require.NoError(t, err)
}
