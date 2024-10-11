package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"

	"github.com/manifoldco/promptui"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/address"
	govcli "cosmossdk.io/x/gov/client/cli"
	govtypes "cosmossdk.io/x/gov/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	proposalText          = "text"
	proposalOther         = "other"
	draftProposalFileName = "draft_group_proposal.json"
	draftMetadataFileName = "draft_group_metadata.json"
)

type proposalType struct {
	Name string
	Msg  sdk.Msg
}

// Prompt the proposal type values and return the proposal and its metadata.
func (p *proposalType) Prompt(cdc codec.Codec, skipMetadata bool, addressCodec address.Codec) (*Proposal, govtypes.ProposalMetadata, error) {
	// set metadata
	metadata, err := govcli.PromptMetadata(skipMetadata, addressCodec)
	if err != nil {
		return nil, metadata, fmt.Errorf("failed to set proposal metadata: %w", err)
	}

	proposal := &Proposal{
		Metadata: "ipfs://CID", // the metadata must be saved on IPFS, set placeholder
		Title:    metadata.Title,
		Summary:  metadata.Summary,
	}

	// set group policy address
	policyAddressPrompt := promptui.Prompt{
		Label:    "Enter group policy address",
		Validate: client.ValidatePromptAddress,
	}
	groupPolicyAddress, err := policyAddressPrompt.Run()
	if err != nil {
		return nil, metadata, fmt.Errorf("failed to set group policy address: %w", err)
	}
	proposal.GroupPolicyAddress = groupPolicyAddress

	// set proposer address
	proposerPrompt := promptui.Prompt{
		Label:    "Enter proposer address",
		Validate: client.ValidatePromptAddress,
	}
	proposerAddress, err := proposerPrompt.Run()
	if err != nil {
		return nil, metadata, fmt.Errorf("failed to set proposer address: %w", err)
	}
	proposal.Proposers = []string{proposerAddress}

	if p.Msg == nil {
		return proposal, metadata, nil
	}

	// set messages field
	result, err := govcli.Prompt(p.Msg, "msg", addressCodec)
	if err != nil {
		return nil, metadata, fmt.Errorf("failed to set proposal message: %w", err)
	}

	message, err := cdc.MarshalInterfaceJSON(result)
	if err != nil {
		return nil, metadata, fmt.Errorf("failed to marshal proposal message: %w", err)
	}
	proposal.Messages = append(proposal.Messages, message)

	return proposal, metadata, nil
}

// NewCmdDraftProposal let a user generate a draft proposal.
func NewCmdDraftProposal() *cobra.Command {
	flagSkipMetadata := "skip-metadata"

	cmd := &cobra.Command{
		Use:          "draft-proposal",
		Short:        "Generate a draft proposal json file. The generated proposal json contains only one message (skeleton).",
		SilenceUsage: true,
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			// prompt proposal type
			proposalTypesPrompt := promptui.Select{
				Label: "Select proposal type",
				Items: []string{proposalText, proposalOther},
			}

			_, selectedProposalType, err := proposalTypesPrompt.Run()
			if err != nil {
				return fmt.Errorf("failed to prompt proposal types: %w", err)
			}

			var proposal *proposalType
			switch selectedProposalType {
			case proposalText:
				proposal = &proposalType{Name: proposalText}
			case proposalOther:
				// prompt proposal type
				proposal = &proposalType{Name: proposalOther}
				msgPrompt := promptui.Select{
					Label: "Select proposal message type:",
					Items: func() []string {
						msgs := clientCtx.InterfaceRegistry.ListImplementations(sdk.MsgInterfaceProtoName)
						sort.Strings(msgs)
						return msgs
					}(),
				}

				_, result, err := msgPrompt.Run()
				if err != nil {
					return fmt.Errorf("failed to prompt proposal types: %w", err)
				}

				proposal.Msg, err = sdk.GetMsgFromTypeURL(clientCtx.Codec, result)
				if err != nil {
					// should never happen
					panic(err)
				}
			default:
				panic("unexpected proposal type")
			}

			skipMetadataPrompt, _ := cmd.Flags().GetBool(flagSkipMetadata)

			result, metadata, err := proposal.Prompt(clientCtx.Codec, skipMetadataPrompt, clientCtx.AddressCodec)
			if err != nil {
				return err
			}

			if err := writeFile(draftProposalFileName, result); err != nil {
				return err
			}

			if !skipMetadataPrompt {
				if err := writeFile(draftMetadataFileName, metadata); err != nil {
					return err
				}
			}

			cmd.Println("The draft proposal has successfully been generated.\nProposals should contain off-chain metadata, please upload the metadata JSON to IPFS.\nThen, replace the generated metadata field with the IPFS CID.")

			return nil
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	cmd.Flags().Bool(flagSkipMetadata, false, "skip metadata prompt")

	return cmd
}

// writeFile writes the input to the file.
func writeFile(fileName string, input any) error {
	raw, err := json.MarshalIndent(input, "", " ")
	if err != nil {
		return fmt.Errorf("failed to marshal proposal: %w", err)
	}

	if err := os.WriteFile(fileName, raw, 0o600); err != nil {
		return err
	}

	return nil
}
