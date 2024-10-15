package cli

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/pflag"

	govutils "cosmossdk.io/x/gov/client/utils"
	govv1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
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
		return errors.New("proposal type is required")
	}

	if p.Title == "" {
		return errors.New("proposal title is required")
	}

	if p.Description == "" {
		return errors.New("proposal description is required")
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

		if strings.EqualFold(proposalType, "text") {
			proposal.Type = v1beta1.ProposalTypeText
		}
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
	Messages        []json.RawMessage `json:"messages,omitempty"`
	Metadata        string            `json:"metadata"`
	Deposit         string            `json:"deposit"`
	Title           string            `json:"title"`
	Summary         string            `json:"summary"`
	ProposalTypeStr string            `json:"proposal_type,omitempty"`

	proposalType govv1.ProposalType
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

	proposalType := govv1.ProposalType_PROPOSAL_TYPE_STANDARD
	if proposal.ProposalTypeStr != "" {
		proposalType = govutils.NormalizeProposalType(proposal.ProposalTypeStr)
	}
	proposal.proposalType = proposalType

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
	cmd.Flags().Bool(FlagExpedited, false, "Whether to expedite the governance proposal")
}

// ReadGovPropCmdFlags parses a MsgSubmitProposal from the provided context and flags.
// Setting the messages is up to the caller.
//
// See also AddGovPropFlagsToCmd.
func ReadGovPropCmdFlags(proposer string, flagSet *pflag.FlagSet) (*govv1.MsgSubmitProposal, error) {
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

	expedited, err := flagSet.GetBool(FlagExpedited)
	if err != nil {
		return nil, fmt.Errorf("could not read expedited: %w", err)
	}
	if expedited {
		rv.Expedited = true
		rv.ProposalType = govv1.ProposalType_PROPOSAL_TYPE_EXPEDITED
	}

	rv.Proposer = proposer

	return rv, nil
}

// ReadGovPropFlags parses a MsgSubmitProposal from the provided context and flags.
// Setting the messages is up to the caller.
//
// See also AddGovPropFlagsToCmd.
// Deprecated: use ReadPropCmdFlags instead, as this depends on global bech32 prefixes.
func ReadGovPropFlags(clientCtx client.Context, flagSet *pflag.FlagSet) (*govv1.MsgSubmitProposal, error) {
	addr, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
	if err != nil {
		return nil, err
	}

	return ReadGovPropCmdFlags(addr, flagSet)
}
