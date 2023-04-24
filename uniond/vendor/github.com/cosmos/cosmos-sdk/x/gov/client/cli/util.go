package cli

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	govutils "github.com/cosmos/cosmos-sdk/x/gov/client/utils"
)

type legacyProposal struct {
	Title       string
	Description string
	Type        string
	Deposit     string
}

// validate the legacyProposal
func (p legacyProposal) validate() error {
	if p.Type == "" {
		return fmt.Errorf("proposal type is required")
	}

	if p.Title == "" {
		return fmt.Errorf("proposal title is required")
	}

	if p.Description == "" {
		return fmt.Errorf("proposal description is required")
	}
	return nil
}

// parseSubmitLegacyProposal reads and parses the legacy proposal.
func parseSubmitLegacyProposal(fs *pflag.FlagSet) (*legacyProposal, error) {
	proposal := &legacyProposal{}
	proposalFile, _ := fs.GetString(FlagProposal)

	if proposalFile == "" {
		proposalType, _ := fs.GetString(FlagProposalType)
		proposal.Title, _ = fs.GetString(FlagTitle)
		proposal.Description, _ = fs.GetString(FlagDescription)
		proposal.Type = govutils.NormalizeProposalType(proposalType)
		proposal.Deposit, _ = fs.GetString(FlagDeposit)
		if err := proposal.validate(); err != nil {
			return nil, err
		}

		return proposal, nil
	}

	for _, flag := range ProposalFlags {
		if v, _ := fs.GetString(flag); v != "" {
			return nil, fmt.Errorf("--%s flag provided alongside --proposal, which is a noop", flag)
		}
	}

	contents, err := os.ReadFile(proposalFile)
	if err != nil {
		return nil, err
	}

	err = json.Unmarshal(contents, proposal)
	if err != nil {
		return nil, err
	}

	if err := proposal.validate(); err != nil {
		return nil, err
	}

	return proposal, nil
}

// proposal defines the new Msg-based proposal.
type proposal struct {
	// Msgs defines an array of sdk.Msgs proto-JSON-encoded as Anys.
	Messages []json.RawMessage `json:"messages,omitempty"`
	Metadata string            `json:"metadata"`
	Deposit  string            `json:"deposit"`
	Title    string            `json:"title"`
	Summary  string            `json:"summary"`
}

// parseSubmitProposal reads and parses the proposal.
func parseSubmitProposal(cdc codec.Codec, path string) ([]sdk.Msg, string, string, string, sdk.Coins, error) {
	var proposal proposal

	contents, err := os.ReadFile(path)
	if err != nil {
		return nil, "", "", "", nil, err
	}

	err = json.Unmarshal(contents, &proposal)
	if err != nil {
		return nil, "", "", "", nil, err
	}

	msgs := make([]sdk.Msg, len(proposal.Messages))
	for i, anyJSON := range proposal.Messages {
		var msg sdk.Msg
		err := cdc.UnmarshalInterfaceJSON(anyJSON, &msg)
		if err != nil {
			return nil, "", "", "", nil, err
		}

		msgs[i] = msg
	}

	deposit, err := sdk.ParseCoinsNormalized(proposal.Deposit)
	if err != nil {
		return nil, "", "", "", nil, err
	}

	return msgs, proposal.Metadata, proposal.Title, proposal.Summary, deposit, nil
}
