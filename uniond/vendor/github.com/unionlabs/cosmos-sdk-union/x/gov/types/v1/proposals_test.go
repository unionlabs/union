package v1_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestProposalStatus_Format(t *testing.T) {
	statusDepositPeriod, _ := v1.ProposalStatusFromString("PROPOSAL_STATUS_DEPOSIT_PERIOD")
	tests := []struct {
		pt                   v1.ProposalStatus
		sprintFArgs          string
		expectedStringOutput string
	}{
		{statusDepositPeriod, "%s", "PROPOSAL_STATUS_DEPOSIT_PERIOD"},
		{statusDepositPeriod, "%v", "1"},
	}
	for _, tt := range tests {
		got := fmt.Sprintf(tt.sprintFArgs, tt.pt)
		require.Equal(t, tt.expectedStringOutput, got)
	}
}

// TestNestedAnys tests that we can call .String() on a struct with nested Anys.
// Here, we're creating a proposal which has a Msg (1st any) with a legacy
// content (2nd any).
func TestNestedAnys(t *testing.T) {
	testProposal := v1beta1.NewTextProposal("Proposal", "testing proposal")
	msgContent, err := v1.NewLegacyContent(testProposal, "cosmos1govacct")
	require.NoError(t, err)
	proposal, err := v1.NewProposal([]sdk.Msg{msgContent}, 1, time.Now(), time.Now(), "", "title", "summary", "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)

	require.NotPanics(t, func() { _ = proposal.String() })
	require.NotEmpty(t, proposal.String())
}

func TestProposalSetExpedited(t *testing.T) {
	const startExpedited = false
	proposal, err := v1.NewProposal([]sdk.Msg{}, 1, time.Now(), time.Now(), "", "title", "summary", "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	require.Equal(t, startExpedited, proposal.Expedited)
	require.Equal(t, proposal.ProposalType, v1.ProposalType_PROPOSAL_TYPE_STANDARD)

	proposal, err = v1.NewProposal([]sdk.Msg{}, 1, time.Now(), time.Now(), "", "title", "summary", "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
	require.NoError(t, err)
	require.Equal(t, !startExpedited, proposal.Expedited)
	require.Equal(t, proposal.ProposalType, v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
}

func TestProposalGetMinDepositFromParams(t *testing.T) {
	testcases := []struct {
		proposalType       v1.ProposalType
		expectedMinDeposit math.Int
	}{
		{
			proposalType:       v1.ProposalType_PROPOSAL_TYPE_EXPEDITED,
			expectedMinDeposit: v1.DefaultMinExpeditedDepositTokens,
		},
		{
			proposalType:       v1.ProposalType_PROPOSAL_TYPE_STANDARD,
			expectedMinDeposit: v1.DefaultMinDepositTokens,
		},
	}

	for _, tc := range testcases {
		proposal, err := v1.NewProposal([]sdk.Msg{}, 1, time.Now(), time.Now(), "", "title", "summary", "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", tc.proposalType)
		require.NoError(t, err)

		actualMinDeposit := proposal.GetMinDepositFromParams(v1.DefaultParams())

		require.Equal(t, 1, len(actualMinDeposit))
		require.Equal(t, sdk.DefaultBondDenom, actualMinDeposit[0].Denom)
		require.Equal(t, tc.expectedMinDeposit, actualMinDeposit[0].Amount)
	}
}
