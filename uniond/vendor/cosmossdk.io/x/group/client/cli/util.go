package cli

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"

	"cosmossdk.io/x/group"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// parseDecisionPolicy reads and parses the decision policy.
func parseDecisionPolicy(cdc codec.Codec, decisionPolicyFile string) (group.DecisionPolicy, error) {
	if decisionPolicyFile == "" {
		return nil, errors.New("decision policy is required")
	}

	contents, err := os.ReadFile(decisionPolicyFile)
	if err != nil {
		return nil, err
	}

	var policy group.DecisionPolicy
	if err := cdc.UnmarshalInterfaceJSON(contents, &policy); err != nil {
		return nil, fmt.Errorf("failed to parse decision policy: %w", err)
	}

	return policy, nil
}

// parseMembers reads and parses the members.
func parseMembers(membersFile string) ([]group.MemberRequest, error) {
	members := struct {
		Members []group.MemberRequest `json:"members"`
	}{}

	if membersFile == "" {
		return members.Members, nil
	}

	contents, err := os.ReadFile(membersFile)
	if err != nil {
		return nil, err
	}

	if err := json.Unmarshal(contents, &members); err != nil {
		return nil, err
	}

	return members.Members, nil
}

func execFromString(execStr string) group.Exec {
	exec := group.Exec_EXEC_UNSPECIFIED
	if execStr == ExecTry {
		exec = group.Exec_EXEC_TRY
	}

	return exec
}

// Proposal defines a Msg-based group proposal for CLI purposes.
type Proposal struct {
	GroupPolicyAddress string `json:"group_policy_address"`
	// Messages defines an array of sdk.Msgs proto-JSON-encoded as Anys.
	Messages  []json.RawMessage `json:"messages,omitempty"`
	Metadata  string            `json:"metadata"`
	Proposers []string          `json:"proposers"`
	Title     string            `json:"title"`
	Summary   string            `json:"summary"`
}

func getCLIProposal(path string) (Proposal, error) {
	contents, err := os.ReadFile(path)
	if err != nil {
		return Proposal{}, err
	}

	return parseCLIProposal(contents)
}

func parseCLIProposal(contents []byte) (Proposal, error) {
	var p Proposal
	if err := json.Unmarshal(contents, &p); err != nil {
		return Proposal{}, err
	}

	return p, nil
}

func parseMsgs(cdc codec.Codec, p Proposal) ([]sdk.Msg, error) {
	msgs := make([]sdk.Msg, len(p.Messages))
	for i, anyJSON := range p.Messages {
		var msg sdk.Msg
		err := cdc.UnmarshalInterfaceJSON(anyJSON, &msg)
		if err != nil {
			return nil, err
		}

		msgs[i] = msg
	}

	return msgs, nil
}
