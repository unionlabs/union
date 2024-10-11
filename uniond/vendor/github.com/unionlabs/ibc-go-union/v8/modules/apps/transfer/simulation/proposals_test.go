package simulation_test

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/simulation"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

func TestProposalMsgs(t *testing.T) {
	// initialize parameters
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)

	// execute ProposalMsgs function
	weightedProposalMsgs := simulation.ProposalMsgs()
	require.Equal(t, len(weightedProposalMsgs), 1)

	w0 := weightedProposalMsgs[0]

	// tests w0 interface:
	require.Equal(t, simulation.OpWeightMsgUpdateParams, w0.AppParamsKey())
	require.Equal(t, simulation.DefaultWeightMsgUpdateParams, w0.DefaultWeight())

	msg, err := w0.MsgSimulatorFn()(ctx, r, accounts, nil)
	require.NoError(t, err)
	msgUpdateParams, ok := msg.(*types.MsgUpdateParams)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgUpdateParams.Signer)
	require.EqualValues(t, msgUpdateParams.Params.SendEnabled, false)
}
