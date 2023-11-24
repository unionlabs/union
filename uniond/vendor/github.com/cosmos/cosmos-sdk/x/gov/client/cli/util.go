package cli

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	govutils "github.com/cosmos/cosmos-sdk/x/gov/client/utils"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
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
	Messages  []json.RawMessage `json:"messages,omitempty"`
	Metadata  string            `json:"metadata"`
	Deposit   string            `json:"deposit"`
	Title     string            `json:"title"`
	Summary   string            `json:"summary"`
	Expedited bool              `json:"expedited"`
}

// parseSubmitProposal reads and parses the proposal.
func parseSubmitProposal(cdc codec.Codec, path string) (proposal, []sdk.Msg, sdk.Coins, error) {
	var proposal proposal

	contents, err := os.ReadFile(path)
	if err != nil {
		return proposal, nil, nil, err
	}

	err = json.Unmarshal(contents, &proposal)
	if err != nil {
		return proposal, nil, nil, err
	}

	msgs := make([]sdk.Msg, len(proposal.Messages))
	for i, anyJSON := range proposal.Messages {
		var msg sdk.Msg
		err := cdc.UnmarshalInterfaceJSON(anyJSON, &msg)
		if err != nil {
			return proposal, nil, nil, err
		}

		msgs[i] = msg
	}

	deposit, err := sdk.ParseCoinsNormalized(proposal.Deposit)
	if err != nil {
		return proposal, nil, nil, err
	}

	return proposal, msgs, deposit, nil
}

// AddGovPropFlagsToCmd adds flags for defining MsgSubmitProposal fields.
//
// See also ReadGovPropFlags.
func AddGovPropFlagsToCmd(cmd *cobra.Command) {
	cmd.Flags().String(FlagDeposit, "", "The deposit to include with the governance proposal")
	cmd.Flags().String(FlagMetadata, "", "The metadata to include with the governance proposal")
	cmd.Flags().String(FlagTitle, "", "The title to put on the governance proposal")
	cmd.Flags().String(FlagSummary, "", "The summary to include with the governance proposal")
}

// ReadGovPropFlags parses a MsgSubmitProposal from the provided context and flags.
// Setting the messages is up to the caller.
//
// See also AddGovPropFlagsToCmd.
func ReadGovPropFlags(clientCtx client.Context, flagSet *pflag.FlagSet) (*govv1.MsgSubmitProposal, error) {
	rv := &govv1.MsgSubmitProposal{}

	deposit, err := flagSet.GetString(FlagDeposit)
	if err != nil {
		return nil, fmt.Errorf("could not read deposit: %w", err)
	}
	if len(deposit) > 0 {
		rv.InitialDeposit, err = sdk.ParseCoinsNormalized(deposit)
		if err != nil {
			return nil, fmt.Errorf("invalid deposit: %w", err)
		}
	}

	rv.Metadata, err = flagSet.GetString(FlagMetadata)
	if err != nil {
		return nil, fmt.Errorf("could not read metadata: %w", err)
	}

	rv.Title, err = flagSet.GetString(FlagTitle)
	if err != nil {
		return nil, fmt.Errorf("could not read title: %w", err)
	}

	rv.Summary, err = flagSet.GetString(FlagSummary)
	if err != nil {
		return nil, fmt.Errorf("could not read summary: %w", err)
	}

	rv.Proposer = clientCtx.GetFromAddress().String()

	return rv, nil
}
