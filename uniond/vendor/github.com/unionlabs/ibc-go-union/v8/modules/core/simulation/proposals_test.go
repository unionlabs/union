package simulation_test

import (
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/simulation"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func TestProposalMsgs(t *testing.T) {
	// initialize parameters
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)

	// execute ProposalMsgs function
	weightedProposalMsgs := simulation.ProposalMsgs()
	require.Equal(t, 4, len(weightedProposalMsgs))

	// tests w0 interface:
	w0 := weightedProposalMsgs[0]
	require.Equal(t, simulation.OpWeightMsgUpdateParams, w0.AppParamsKey())
	require.Equal(t, simulation.DefaultWeight, w0.DefaultWeight())

	msg, err := w0.MsgSimulatorFn()(ctx, r, accounts, nil)
	require.NoError(t, err)
	msgUpdateParams, ok := msg.(*clienttypes.MsgUpdateParams)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgUpdateParams.Signer)
	require.EqualValues(t, []string{"06-solomachine", "07-tendermint"}, msgUpdateParams.Params.AllowedClients)

	// tests w1 interface:
	w1 := weightedProposalMsgs[1]
	require.Equal(t, simulation.OpWeightMsgUpdateParams, w1.AppParamsKey())
	require.Equal(t, simulation.DefaultWeight, w1.DefaultWeight())

	msg1, err := w1.MsgSimulatorFn()(ctx, r, accounts, nil)
	require.NoError(t, err)
	msgUpdateConnectionParams, ok := msg1.(*connectiontypes.MsgUpdateParams)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgUpdateParams.Signer)
	require.EqualValues(t, uint64(100), msgUpdateConnectionParams.Params.MaxExpectedTimePerBlock)

	// tests w2 interface:
	w2 := weightedProposalMsgs[2]
	require.Equal(t, simulation.OpWeightMsgRecoverClient, w2.AppParamsKey())
	require.Equal(t, simulation.DefaultWeight, w2.DefaultWeight())

	msg2, err := w2.MsgSimulatorFn()(ctx, r, accounts, nil)
	require.NoError(t, err)
	msgRecoverClient, ok := msg2.(*clienttypes.MsgRecoverClient)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgRecoverClient.Signer)
	require.EqualValues(t, "07-tendermint-1", msgRecoverClient.SubstituteClientId)

	// tests w3 interface:
	w3 := weightedProposalMsgs[3]
	require.Equal(t, simulation.OpWeightMsgIBCSoftwareUpgrade, w3.AppParamsKey())
	require.Equal(t, simulation.DefaultWeight, w3.DefaultWeight())

	msg3, err := w3.MsgSimulatorFn()(ctx, r, accounts, nil)
	require.NoError(t, err)
	msgIBCSoftwareUpgrade, ok := msg3.(*clienttypes.MsgIBCSoftwareUpgrade)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgIBCSoftwareUpgrade.Signer)
	clientState, err := clienttypes.UnpackClientState(msgIBCSoftwareUpgrade.UpgradedClientState)
	require.NoError(t, err)
	require.EqualValues(t, time.Hour*24*7*2, clientState.(*ibctm.ClientState).UnbondingPeriod)
}
