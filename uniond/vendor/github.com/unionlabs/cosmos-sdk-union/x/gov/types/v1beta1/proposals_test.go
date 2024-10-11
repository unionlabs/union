package v1beta1_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/gov/types/v1beta1"
)

func TestProposalStatus_Format(t *testing.T) {
	statusDepositPeriod, _ := v1beta1.ProposalStatusFromString("PROPOSAL_STATUS_DEPOSIT_PERIOD")
	tests := []struct {
		pt                   v1beta1.ProposalStatus
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

func TestContentFromProposalType(t *testing.T) {
	tests := []struct {
		proposalType string
		expectedType string
	}{
		{
			proposalType: "TextProposal",
			expectedType: "",
		},
		{
			proposalType: "text",
			expectedType: v1beta1.ProposalTypeText,
		},
		{
			proposalType: "Text",
			expectedType: v1beta1.ProposalTypeText,
		},
	}

	for _, test := range tests {
		content, ok := v1beta1.ContentFromProposalType("title", "foo", test.proposalType)
		if test.expectedType == "" {
			require.False(t, ok)
			continue
		}

		require.True(t, ok)
		require.NotNil(t, content)
		require.Equal(t, test.expectedType, content.ProposalType())
	}
}
