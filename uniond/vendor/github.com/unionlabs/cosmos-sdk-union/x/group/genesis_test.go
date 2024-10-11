package group

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	banktypes "cosmossdk.io/x/bank/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	memberPub  = secp256k1.GenPrivKey().PubKey()
	accPub     = secp256k1.GenPrivKey().PubKey()
	accAddr    = sdk.AccAddress(accPub.Address())
	memberAddr = sdk.AccAddress(memberPub.Address())
)

func TestGenesisStateValidate(t *testing.T) {
	submittedAt := time.Now().UTC()
	timeout := submittedAt.Add(time.Second * 1).UTC()
	addressCodec := codectestutil.CodecOptions{}.GetAddressCodec()

	accStrAddr, err := addressCodec.BytesToString(accAddr)
	require.NoError(t, err)
	memberStrAdrr, err := addressCodec.BytesToString(memberAddr)
	require.NoError(t, err)

	groupPolicy := &GroupPolicyInfo{
		Address:  accStrAddr,
		GroupId:  1,
		Admin:    accStrAddr,
		Version:  1,
		Metadata: "policy metadata",
	}
	err = groupPolicy.SetDecisionPolicy(&ThresholdDecisionPolicy{
		Threshold: "1",
		Windows: &DecisionPolicyWindows{
			VotingPeriod: time.Second,
		},
	})
	require.NoError(t, err)

	// create another group policy to set invalid decision policy for testing
	groupPolicy2 := &GroupPolicyInfo{
		Address:  accStrAddr,
		GroupId:  1,
		Admin:    accStrAddr,
		Version:  1,
		Metadata: "policy metadata",
	}
	err = groupPolicy2.SetDecisionPolicy(&ThresholdDecisionPolicy{
		Threshold: "1",
		Windows: &DecisionPolicyWindows{
			VotingPeriod: 0,
		},
	})
	require.NoError(t, err)

	proposal := &Proposal{
		Id:                 1,
		GroupPolicyAddress: accStrAddr,
		Metadata:           "proposal metadata",
		GroupVersion:       1,
		GroupPolicyVersion: 1,
		Proposers: []string{
			memberStrAdrr,
		},
		SubmitTime: submittedAt,
		Status:     PROPOSAL_STATUS_ACCEPTED,
		FinalTallyResult: TallyResult{
			YesCount:        "1",
			NoCount:         "0",
			AbstainCount:    "0",
			NoWithVetoCount: "0",
		},
		VotingPeriodEnd: timeout,
		ExecutorResult:  PROPOSAL_EXECUTOR_RESULT_SUCCESS,
	}
	err = proposal.SetMsgs([]sdk.Msg{&banktypes.MsgSend{
		FromAddress: accStrAddr,
		ToAddress:   memberStrAdrr,
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}})
	require.NoError(t, err)

	testCases := []struct {
		name         string
		genesisState GenesisState
		expErr       bool
	}{
		{
			"valid genesisState",
			GenesisState{
				GroupSeq:       2,
				Groups:         []*GroupInfo{{Id: 1, Admin: accStrAddr, Metadata: "1", Version: 1, TotalWeight: "1"}, {Id: 2, Admin: accStrAddr, Metadata: "2", Version: 2, TotalWeight: "2"}},
				GroupMembers:   []*GroupMember{{GroupId: 1, Member: &Member{Address: memberStrAdrr, Weight: "1", Metadata: "member metadata"}}, {GroupId: 2, Member: &Member{Address: memberStrAdrr, Weight: "2", Metadata: "member metadata"}}},
				GroupPolicySeq: 1,
				GroupPolicies:  []*GroupPolicyInfo{groupPolicy},
				ProposalSeq:    1,
				Proposals:      []*Proposal{proposal},
				Votes:          []*Vote{{ProposalId: proposal.Id, Voter: memberStrAdrr, SubmitTime: submittedAt, Option: VOTE_OPTION_YES}},
			},
			false,
		},
		{
			"empty genesisState",
			GenesisState{},
			false,
		},
		{
			"empty group id",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          0,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
			},
			true,
		},
		{
			"invalid group admin",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       "invalid admin",
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
			},
			true,
		},
		{
			"invalid group version",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     0,
						TotalWeight: "1",
					},
				},
			},
			true,
		},
		{
			"invalid group TotalWeight",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "-1",
					},
				},
			},
			true,
		},
		{
			"invalid group policy address",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					{
						Address:  "invalid address",
						GroupId:  1,
						Admin:    accStrAddr,
						Version:  1,
						Metadata: "policy metadata",
					},
				},
			},
			true,
		},
		{
			"invalid group policy admin",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					{
						Address:  accStrAddr,
						GroupId:  1,
						Admin:    "invalid admin",
						Version:  1,
						Metadata: "policy metadata",
					},
				},
			},
			true,
		},
		{
			"invalid group policy's group id",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					{
						Address:  accStrAddr,
						GroupId:  0,
						Admin:    accStrAddr,
						Version:  1,
						Metadata: "policy metadata",
					},
				},
			},
			true,
		},
		{
			"invalid group policy version",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					{
						Address:  accStrAddr,
						GroupId:  1,
						Admin:    accStrAddr,
						Version:  0,
						Metadata: "policy metadata",
					},
				},
			},
			true,
		},
		{
			"invalid group policy's decision policy",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					{
						Address:        accStrAddr,
						GroupId:        1,
						Admin:          accStrAddr,
						Version:        1,
						Metadata:       "policy metadata",
						DecisionPolicy: groupPolicy2.DecisionPolicy,
					},
				},
			},
			true,
		},
		{
			"invalid group member's group id",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupMembers: []*GroupMember{
					{
						GroupId: 0,
						Member: &Member{
							Address: memberStrAdrr,
							Weight:  "1", Metadata: "member metadata",
						},
					},
				},
			},
			true,
		},
		{
			"invalid group member address",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupMembers: []*GroupMember{
					{
						GroupId: 1,
						Member: &Member{
							Address: "invalid address",
							Weight:  "1", Metadata: "member metadata",
						},
					},
				},
			},
			true,
		},
		{
			"invalid group member weight",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupMembers: []*GroupMember{
					{
						GroupId: 1,
						Member: &Member{
							Address: memberStrAdrr,
							Weight:  "-1", Metadata: "member metadata",
						},
					},
				},
			},
			true,
		},
		{
			"invalid proposal id",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 0,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
					},
				},
			},
			true,
		},
		{
			"invalid group policy address of proposal",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: "invalid address",
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
					},
				},
			},
			true,
		},
		{
			"invalid group version of proposal",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       0,
						GroupPolicyVersion: 1,
					},
				},
			},
			true,
		},
		{
			"invalid group policy version",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 0,
					},
				},
			},
			true,
		},
		{
			"invalid FinalTallyResult with negative YesCount",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
						Proposers: []string{
							memberStrAdrr,
						},
						SubmitTime: submittedAt,
						Status:     PROPOSAL_STATUS_ACCEPTED,
						FinalTallyResult: TallyResult{
							YesCount:        "-1",
							NoCount:         "0",
							AbstainCount:    "0",
							NoWithVetoCount: "0",
						},
					},
				},
			},
			true,
		},
		{
			"invalid FinalTallyResult with negative NoCount",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
						Proposers: []string{
							memberStrAdrr,
						},
						SubmitTime: submittedAt,
						Status:     PROPOSAL_STATUS_ACCEPTED,
						FinalTallyResult: TallyResult{
							YesCount:        "0",
							NoCount:         "-1",
							AbstainCount:    "0",
							NoWithVetoCount: "0",
						},
					},
				},
			},
			true,
		},
		{
			"invalid FinalTallyResult with negative AbstainCount",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
						Proposers: []string{
							memberStrAdrr,
						},
						SubmitTime: submittedAt,
						Status:     PROPOSAL_STATUS_ACCEPTED,
						FinalTallyResult: TallyResult{
							YesCount:        "0",
							NoCount:         "0",
							AbstainCount:    "-1",
							NoWithVetoCount: "0",
						},
					},
				},
			},
			true,
		},
		{
			"invalid FinalTallyResult with negative VetoCount",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					{
						Id:                 1,
						GroupPolicyAddress: accStrAddr,
						Metadata:           "proposal metadata",
						GroupVersion:       1,
						GroupPolicyVersion: 1,
						Proposers: []string{
							memberStrAdrr,
						},
						SubmitTime: submittedAt,
						Status:     PROPOSAL_STATUS_ACCEPTED,
						FinalTallyResult: TallyResult{
							YesCount:        "0",
							NoCount:         "0",
							AbstainCount:    "0",
							NoWithVetoCount: "-1",
						},
					},
				},
			},
			true,
		},
		{
			"invalid voter",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					proposal,
				},
				Votes: []*Vote{
					{
						ProposalId: proposal.Id,
						Voter:      "invalid voter",
						SubmitTime: submittedAt,
						Option:     VOTE_OPTION_YES,
					},
				},
			},
			true,
		},
		{
			"invalid proposal id",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					proposal,
				},
				Votes: []*Vote{
					{
						ProposalId: 0,
						Voter:      memberStrAdrr,
						SubmitTime: submittedAt,
						Option:     VOTE_OPTION_YES,
					},
				},
			},
			true,
		},
		{
			"vote on proposal that doesn't exist",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					proposal,
				},
				Votes: []*Vote{
					{
						ProposalId: 2,
						Voter:      memberStrAdrr,
						SubmitTime: submittedAt,
						Option:     VOTE_OPTION_YES,
					},
				},
			},
			true,
		},
		{
			"invalid vote option",
			GenesisState{
				Groups: []*GroupInfo{
					{
						Id:          1,
						Admin:       accStrAddr,
						Metadata:    "1",
						Version:     1,
						TotalWeight: "1",
					},
				},
				GroupPolicies: []*GroupPolicyInfo{
					groupPolicy,
				},
				Proposals: []*Proposal{
					proposal,
				},
				Votes: []*Vote{
					{
						ProposalId: proposal.Id,
						Voter:      memberStrAdrr,
						SubmitTime: submittedAt,
						Option:     VOTE_OPTION_UNSPECIFIED,
					},
				},
			},
			true,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.genesisState.Validate()
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
