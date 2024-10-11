package simulation

import (
	"math/rand"
	"time"

	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

const (
	GroupInfo       = "group-info"
	GroupMembers    = "group-members"
	GroupPolicyInfo = "group-policy-info"
	GroupProposals  = "group-proposals"
	GroupVote       = "group-vote"
)

func checkAccExists(acc string, g []*group.GroupMember, lastIndex int) bool {
	for i := 0; i < lastIndex; i++ {
		if g[i].Member.Address == acc {
			return true
		}
	}
	return false
}

func getGroups(r *rand.Rand, accounts []simtypes.Account) []*group.GroupInfo {
	groups := make([]*group.GroupInfo, 3)
	for i := 0; i < 3; i++ {
		acc, _ := simtypes.RandomAcc(r, accounts)
		accAddr := acc.AddressBech32
		groups[i] = &group.GroupInfo{
			Id:          uint64(i + 1),
			Admin:       accAddr,
			Metadata:    simtypes.RandStringOfLength(r, 10),
			Version:     1,
			TotalWeight: "10",
		}
	}
	return groups
}

func getGroupMembers(r *rand.Rand, accounts []simtypes.Account) []*group.GroupMember {
	groupMembers := make([]*group.GroupMember, 3)
	for i := 0; i < 3; i++ {
		acc, _ := simtypes.RandomAcc(r, accounts)
		accAddr := acc.AddressBech32
		for checkAccExists(accAddr, groupMembers, i) {
			acc, _ = simtypes.RandomAcc(r, accounts)
			accAddr = acc.AddressBech32
		}
		groupMembers[i] = &group.GroupMember{
			GroupId: uint64(i + 1),
			Member: &group.Member{
				Address:  accAddr,
				Weight:   "10",
				Metadata: simtypes.RandStringOfLength(r, 10),
			},
		}
	}
	return groupMembers
}

func getGroupPolicies(r *rand.Rand, simState *module.SimulationState) []*group.GroupPolicyInfo {
	var groupPolicies []*group.GroupPolicyInfo

	usedAccs := make(map[string]bool)
	for i := 0; i < 3; i++ {
		acc, _ := simtypes.RandomAcc(r, simState.Accounts)
		if usedAccs[acc.AddressBech32] {
			if len(usedAccs) != len(simState.Accounts) {
				// Go again if the account is used and there are more to take from
				i--
			}

			continue
		}
		usedAccs[acc.AddressBech32] = true

		any, err := codectypes.NewAnyWithValue(group.NewThresholdDecisionPolicy("10", time.Second, 0))
		if err != nil {
			panic(err)
		}
		groupPolicies = append(groupPolicies, &group.GroupPolicyInfo{
			GroupId:        uint64(i + 1),
			Admin:          acc.AddressBech32,
			Address:        acc.AddressBech32,
			Version:        1,
			DecisionPolicy: any,
			Metadata:       simtypes.RandStringOfLength(r, 10),
		})
	}
	return groupPolicies
}

func getProposals(r *rand.Rand, simState *module.SimulationState, groupPolicies []*group.GroupPolicyInfo) []*group.Proposal {
	proposals := make([]*group.Proposal, 3)
	addr0 := simState.Accounts[0].AddressBech32
	addr1 := simState.Accounts[1].AddressBech32
	proposers := []string{addr0, addr1}
	for i := 0; i < 3; i++ {
		idx := r.Intn(len(groupPolicies))
		groupPolicyAddress := groupPolicies[idx].Address
		to, _ := simtypes.RandomAcc(r, simState.Accounts)

		submittedAt := time.Unix(0, 0)
		timeout := submittedAt.Add(time.Second * 1000).UTC()

		proposal := &group.Proposal{
			Id:                 uint64(i + 1),
			Proposers:          proposers,
			GroupPolicyAddress: groupPolicyAddress,
			GroupVersion:       uint64(i + 1),
			GroupPolicyVersion: uint64(i + 1),
			Status:             group.PROPOSAL_STATUS_SUBMITTED,
			FinalTallyResult: group.TallyResult{
				YesCount:        "1",
				NoCount:         "1",
				AbstainCount:    "1",
				NoWithVetoCount: "0",
			},
			ExecutorResult:  group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			Metadata:        simtypes.RandStringOfLength(r, 50),
			SubmitTime:      submittedAt,
			VotingPeriodEnd: timeout,
		}

		err := proposal.SetMsgs([]sdk.Msg{&banktypes.MsgSend{
			FromAddress: groupPolicyAddress,
			ToAddress:   to.AddressBech32,
			Amount:      sdk.NewCoins(sdk.NewInt64Coin("test", 10)),
		}})
		if err != nil {
			panic(err)
		}

		proposals[i] = proposal
	}

	return proposals
}

func getVotes(r *rand.Rand, simState *module.SimulationState) []*group.Vote {
	votes := make([]*group.Vote, 3)

	for i := 0; i < 3; i++ {
		voterAddr := simState.Accounts[i].AddressBech32
		votes[i] = &group.Vote{
			ProposalId: uint64(i + 1),
			Voter:      voterAddr,
			Option:     getVoteOption(i),
			Metadata:   simtypes.RandStringOfLength(r, 50),
			SubmitTime: time.Unix(0, 0),
		}
	}

	return votes
}

func getVoteOption(index int) group.VoteOption {
	switch index {
	case 0:
		return group.VOTE_OPTION_YES
	case 1:
		return group.VOTE_OPTION_NO
	case 2:
		return group.VOTE_OPTION_ABSTAIN
	default:
		return group.VOTE_OPTION_NO_WITH_VETO
	}
}

// RandomizedGenState generates a random GenesisState for the group module.
func RandomizedGenState(simState *module.SimulationState) {
	// The test requires we have at least 3 accounts.
	if len(simState.Accounts) < 3 {
		return
	}

	// groups
	var groups []*group.GroupInfo
	simState.AppParams.GetOrGenerate(GroupInfo, &groups, simState.Rand, func(r *rand.Rand) { groups = getGroups(r, simState.Accounts) })

	// group members
	var members []*group.GroupMember
	simState.AppParams.GetOrGenerate(GroupMembers, &members, simState.Rand, func(r *rand.Rand) { members = getGroupMembers(r, simState.Accounts) })

	// group policies
	var groupPolicies []*group.GroupPolicyInfo
	simState.AppParams.GetOrGenerate(GroupPolicyInfo, &groupPolicies, simState.Rand, func(r *rand.Rand) { groupPolicies = getGroupPolicies(r, simState) })

	// proposals
	var proposals []*group.Proposal
	simState.AppParams.GetOrGenerate(GroupProposals, &proposals, simState.Rand, func(r *rand.Rand) { proposals = getProposals(r, simState, groupPolicies) })

	// votes
	var votes []*group.Vote
	simState.AppParams.GetOrGenerate(GroupVote, &votes, simState.Rand, func(r *rand.Rand) { votes = getVotes(r, simState) })

	groupGenesis := group.GenesisState{
		GroupSeq:       3,
		Groups:         groups,
		GroupMembers:   members,
		GroupPolicySeq: 3,
		GroupPolicies:  groupPolicies,
		ProposalSeq:    3,
		Proposals:      proposals,
		Votes:          votes,
	}

	simState.GenState[group.ModuleName] = simState.Cdc.MustMarshalJSON(&groupGenesis)
}
