package v1_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec/address"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestEmptyGenesis(t *testing.T) {
	state1 := v1.GenesisState{}
	require.True(t, state1.Empty())

	state2 := v1.DefaultGenesisState()
	require.False(t, state2.Empty())
}

func TestValidateGenesis(t *testing.T) {
	codec := address.NewBech32Codec("cosmos")
	params := v1.DefaultParams()

	testCases := []struct {
		name         string
		genesisState func() *v1.GenesisState
		expErrMsg    string
	}{
		{
			name: "valid",
			genesisState: func() *v1.GenesisState {
				return v1.NewGenesisState(v1.DefaultStartingProposalID, params)
			},
		},
		{
			name: "invalid StartingProposalId",
			genesisState: func() *v1.GenesisState {
				return v1.NewGenesisState(0, params)
			},
			expErrMsg: "starting proposal id must be greater than 0",
		},
		{
			name: "invalid min deposit",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.MinDeposit = sdk.Coins{{
					Denom:  sdk.DefaultBondDenom,
					Amount: sdkmath.NewInt(-100),
				}}

				return v1.NewGenesisState(v1.DefaultStartingProposalID, params1)
			},
			expErrMsg: "invalid minimum deposit",
		},
		{
			name: "invalid max deposit period",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.MaxDepositPeriod = nil

				return v1.NewGenesisState(v1.DefaultStartingProposalID, params1)
			},
			expErrMsg: "maximum deposit period must not be nil",
		},
		{
			name: "invalid quorum",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.Quorum = "2"

				return v1.NewGenesisState(v1.DefaultStartingProposalID, params1)
			},
			expErrMsg: "quorum too large",
		},
		{
			name: "invalid threshold",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.Threshold = "2"

				return v1.NewGenesisState(v1.DefaultStartingProposalID, params1)
			},
			expErrMsg: "vote threshold too large",
		},
		{
			name: "invalid veto threshold",
			genesisState: func() *v1.GenesisState {
				params1 := params
				params1.VetoThreshold = "2"

				return v1.NewGenesisState(v1.DefaultStartingProposalID, params1)
			},
			expErrMsg: "veto threshold too large",
		},
		{
			name: "duplicate proposals",
			genesisState: func() *v1.GenesisState {
				state := v1.NewGenesisState(v1.DefaultStartingProposalID, params)
				state.Proposals = append(state.Proposals, &v1.Proposal{Id: 1})
				state.Proposals = append(state.Proposals, &v1.Proposal{Id: 1})

				return state
			},
			expErrMsg: "duplicate proposal id: 1",
		},
		{
			name: "duplicate votes",
			genesisState: func() *v1.GenesisState {
				state := v1.NewGenesisState(v1.DefaultStartingProposalID, params)
				state.Proposals = append(state.Proposals, &v1.Proposal{Id: 1})
				state.Votes = append(state.Votes,
					&v1.Vote{
						ProposalId: 1,
						Voter:      "voter",
					},
					&v1.Vote{
						ProposalId: 1,
						Voter:      "voter",
					})

				return state
			},
			expErrMsg: "duplicate vote",
		},
		{
			name: "duplicate deposits",
			genesisState: func() *v1.GenesisState {
				state := v1.NewGenesisState(v1.DefaultStartingProposalID, params)
				state.Proposals = append(state.Proposals, &v1.Proposal{Id: 1})
				state.Deposits = append(state.Deposits,
					&v1.Deposit{
						ProposalId: 1,
						Depositor:  "depositor",
					},
					&v1.Deposit{
						ProposalId: 1,
						Depositor:  "depositor",
					})

				return state
			},
			expErrMsg: "duplicate deposit: proposal_id:1 depositor:\"depositor\"",
		},
		{
			name: "non-existent proposal id in votes",
			genesisState: func() *v1.GenesisState {
				state := v1.NewGenesisState(v1.DefaultStartingProposalID, params)
				state.Votes = append(state.Votes,
					&v1.Vote{
						ProposalId: 1,
						Voter:      "voter",
					})

				return state
			},
			expErrMsg: "vote proposal_id:1 voter:\"voter\"  has non-existent proposal id: 1",
		},
		{
			name: "non-existent proposal id in deposits",
			genesisState: func() *v1.GenesisState {
				state := v1.NewGenesisState(v1.DefaultStartingProposalID, params)
				state.Deposits = append(state.Deposits,
					&v1.Deposit{
						ProposalId: 1,
						Depositor:  "depositor",
					})

				return state
			},
			expErrMsg: "deposit proposal_id:1 depositor:\"depositor\"",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err := v1.ValidateGenesis(codec, tc.genesisState())
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.ErrorContains(t, err, tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
