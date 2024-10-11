package simulation_test

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	controllerkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/keeper"
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	hostkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/simulation"
)

func TestProposalMsgs(t *testing.T) {
	// initialize parameters
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)

	tests := []struct {
		name       string
		controller *controllerkeeper.Keeper
		host       *hostkeeper.Keeper
		expMsgs    []sdk.Msg
	}{
		{
			name:       "host and controller keepers are both enabled",
			controller: &controllerkeeper.Keeper{},
			host:       &hostkeeper.Keeper{},
			expMsgs: []sdk.Msg{
				hosttypes.NewMsgUpdateParams(
					sdk.AccAddress(address.Module("gov")).String(),
					hosttypes.NewParams(false, []string{hosttypes.AllowAllHostMsgs}),
				),
				controllertypes.NewMsgUpdateParams(
					sdk.AccAddress(address.Module("gov")).String(),
					controllertypes.NewParams(false),
				),
			},
		},
		{
			name:       "host and controller keepers are not enabled",
			controller: nil,
			host:       nil,
		},
		{
			name:       "only controller keeper is enabled",
			controller: &controllerkeeper.Keeper{},
			expMsgs: []sdk.Msg{
				controllertypes.NewMsgUpdateParams(
					sdk.AccAddress(address.Module("gov")).String(),
					controllertypes.NewParams(false),
				),
			},
		},
		{
			name: "only host keeper is enabled",
			host: &hostkeeper.Keeper{},
			expMsgs: []sdk.Msg{
				hosttypes.NewMsgUpdateParams(
					sdk.AccAddress(address.Module("gov")).String(),
					hosttypes.NewParams(false, []string{hosttypes.AllowAllHostMsgs}),
				),
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// execute ProposalMsgs function
			weightedProposalMsgs := simulation.ProposalMsgs(tc.controller, tc.host)
			require.Equal(t, len(tc.expMsgs), len(weightedProposalMsgs))

			for idx, weightedMsg := range weightedProposalMsgs {
				// tests weighted interface:
				require.Equal(t, simulation.OpWeightMsgUpdateParams, weightedMsg.AppParamsKey())
				require.Equal(t, simulation.DefaultWeightMsgUpdateParams, weightedMsg.DefaultWeight())

				msg, err := weightedMsg.MsgSimulatorFn()(ctx, r, accounts, nil)
				require.NoError(t, err)

				if msgUpdateHostParams, ok := msg.(*hosttypes.MsgUpdateParams); ok {
					require.Equal(t, tc.expMsgs[idx], msgUpdateHostParams)
				} else {
					msgUpdateControllerParams, ok := msg.(*controllertypes.MsgUpdateParams)
					require.True(t, ok)
					require.Equal(t, tc.expMsgs[idx], msgUpdateControllerParams)
				}
			}
		})
	}
}
