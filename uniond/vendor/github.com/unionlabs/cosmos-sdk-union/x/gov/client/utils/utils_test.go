package utils_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/gov/client/utils"
)

func TestNormalizeWeightedVoteOptions(t *testing.T) {
	cases := map[string]struct {
		options    string
		normalized string
	}{
		"simple Yes": {
			options:    "Yes",
			normalized: "VOTE_OPTION_YES=1",
		},
		"simple yes": {
			options:    "yes",
			normalized: "VOTE_OPTION_YES=1",
		},
		"formal yes": {
			options:    "yes=1",
			normalized: "VOTE_OPTION_YES=1",
		},
		"half yes half no": {
			options:    "yes=0.5,no=0.5",
			normalized: "VOTE_OPTION_YES=0.5,VOTE_OPTION_NO=0.5",
		},
		"3 options": {
			options:    "Yes=0.5,No=0.4,NoWithVeto=0.1",
			normalized: "VOTE_OPTION_YES=0.5,VOTE_OPTION_NO=0.4,VOTE_OPTION_NO_WITH_VETO=0.1",
		},
		"zero weight option": {
			options:    "Yes=0.5,No=0.5,NoWithVeto=0",
			normalized: "VOTE_OPTION_YES=0.5,VOTE_OPTION_NO=0.5,VOTE_OPTION_NO_WITH_VETO=0",
		},
		"minus weight option": {
			options:    "Yes=0.5,No=0.6,NoWithVeto=-0.1",
			normalized: "VOTE_OPTION_YES=0.5,VOTE_OPTION_NO=0.6,VOTE_OPTION_NO_WITH_VETO=-0.1",
		},
		"empty options": {
			options:    "",
			normalized: "=1",
		},
		"not available option": {
			options:    "Yessss=1",
			normalized: "Yessss=1",
		},
	}

	for _, tc := range cases {
		normalized := utils.NormalizeWeightedVoteOptions(tc.options)
		require.Equal(t, normalized, tc.normalized)
	}
}

func TestNormalizeProposalStatus(t *testing.T) {
	type args struct {
		status string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{"invalid", args{"unknown"}, "unknown"},
		{"deposit_period", args{"deposit_period"}, "PROPOSAL_STATUS_DEPOSIT_PERIOD"},
		{"DepositPeriod", args{"DepositPeriod"}, "PROPOSAL_STATUS_DEPOSIT_PERIOD"},
		{"voting_period", args{"deposit_period"}, "PROPOSAL_STATUS_DEPOSIT_PERIOD"},
		{"VotingPeriod", args{"DepositPeriod"}, "PROPOSAL_STATUS_DEPOSIT_PERIOD"},
		{"passed", args{"passed"}, "PROPOSAL_STATUS_PASSED"},
		{"Passed", args{"Passed"}, "PROPOSAL_STATUS_PASSED"},
		{"Rejected", args{"Rejected"}, "PROPOSAL_STATUS_REJECTED"},
		{"rejected", args{"rejected"}, "PROPOSAL_STATUS_REJECTED"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.want, utils.NormalizeProposalStatus(tt.args.status))
		})
	}
}
