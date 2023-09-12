package cli

import (
	"errors"
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/group"
)

const (
	FlagExec               = "exec"
	ExecTry                = "try"
	FlagGroupPolicyAsAdmin = "group-policy-as-admin"
)

// TxCmd returns a root CLI command handler for all x/group transaction commands.
func TxCmd(name string) *cobra.Command {
	txCmd := &cobra.Command{
		Use:                        name,
		Short:                      "Group transaction subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	txCmd.AddCommand(
		MsgCreateGroupCmd(),
		MsgUpdateGroupAdminCmd(),
		MsgUpdateGroupMetadataCmd(),
		MsgUpdateGroupMembersCmd(),
		MsgCreateGroupWithPolicyCmd(),
		MsgCreateGroupPolicyCmd(),
		MsgUpdateGroupPolicyAdminCmd(),
		MsgUpdateGroupPolicyDecisionPolicyCmd(),
		MsgUpdateGroupPolicyMetadataCmd(),
		MsgWithdrawProposalCmd(),
		MsgSubmitProposalCmd(),
		MsgVoteCmd(),
		MsgExecCmd(),
		MsgLeaveGroupCmd(),
		NewCmdDraftProposal(),
	)

	return txCmd
}

// MsgCreateGroupCmd creates a CLI command for Msg/CreateGroup.
func MsgCreateGroupCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group [admin] [metadata] [members-json-file]",
		Short: "Create a group which is an aggregation of member accounts with associated weights and an administrator account.",
		Long: `Create a group which is an aggregation of member accounts with associated weights and an administrator account.
Note, the '--from' flag is ignored as it is implied from [admin]. Members accounts can be given through a members JSON file that contains an array of members.`,
		Example: fmt.Sprintf(`
%s tx group create-group [admin] [metadata] [members-json-file]

Where members.json contains:

{
	"members": [
		{
			"address": "addr1",
			"weight": "1",
			"metadata": "some metadata"
		},
		{
			"address": "addr2",
			"weight": "1",
			"metadata": "some metadata"
		}
	]
}`, version.AppName),
		Args: cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			members, err := parseMembers(args[2])
			if err != nil {
				return err
			}

			msg := &group.MsgCreateGroup{
				Admin:    clientCtx.GetFromAddress().String(),
				Members:  members,
				Metadata: args[1],
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupMembersCmd creates a CLI command for Msg/UpdateGroupMembers.
func MsgUpdateGroupMembersCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-members [admin] [group-id] [members-json-file]",
		Short: "Update a group's members. Set a member's weight to \"0\" to delete it.",
		Example: fmt.Sprintf(`
%s tx group update-group-members [admin] [group-id] [members-json-file]

Where members.json contains:

{
	"members": [
		{
			"address": "addr1",
			"weight": "1",
			"metadata": "some new metadata"
		},
		{
			"address": "addr2",
			"weight": "0",
			"metadata": "some metadata"
		}
	]
}

Set a member's weight to "0" to delete it.
`, version.AppName),
		Args: cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			members, err := parseMembers(args[2])
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupMembers{
				Admin:         clientCtx.GetFromAddress().String(),
				MemberUpdates: members,
				GroupId:       groupID,
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupAdminCmd creates a CLI command for Msg/UpdateGroupAdmin.
func MsgUpdateGroupAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-admin [admin] [group-id] [new-admin]",
		Short: "Update a group's admin",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupAdmin{
				Admin:    clientCtx.GetFromAddress().String(),
				NewAdmin: args[2],
				GroupId:  groupID,
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupMetadataCmd creates a CLI command for Msg/UpdateGroupMetadata.
func MsgUpdateGroupMetadataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-metadata [admin] [group-id] [metadata]",
		Short: "Update a group's metadata",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupMetadata{
				Admin:    clientCtx.GetFromAddress().String(),
				Metadata: args[2],
				GroupId:  groupID,
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgCreateGroupWithPolicyCmd creates a CLI command for Msg/CreateGroupWithPolicy.
func MsgCreateGroupWithPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group-with-policy [admin] [group-metadata] [group-policy-metadata] [members-json-file] [decision-policy-json-file]",
		Short: "Create a group with policy which is an aggregation of member accounts with associated weights, an administrator account and decision policy.",
		Long: `Create a group with policy which is an aggregation of member accounts with associated weights,
an administrator account and decision policy. Note, the '--from' flag is ignored as it is implied from [admin].
Members accounts can be given through a members JSON file that contains an array of members.
If group-policy-as-admin flag is set to true, the admin of the newly created group and group policy is set with the group policy address itself.`,
		Example: fmt.Sprintf(`
%s tx group create-group-with-policy [admin] [group-metadata] [group-policy-metadata] members.json policy.json

where members.json contains:

{
	"members": [
		{
			"address": "addr1",
			"weight": "1",
			"metadata": "some metadata"
		},
		{
			"address": "addr2",
			"weight": "1",
			"metadata": "some metadata"
		}
	]
}

and policy.json contains:

{
    "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
    "threshold": "1",
    "windows": {
        "voting_period": "120h",
        "min_execution_period": "0s"
    }
}
`, version.AppName),
		Args: cobra.MinimumNArgs(5),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			groupPolicyAsAdmin, err := cmd.Flags().GetBool(FlagGroupPolicyAsAdmin)
			if err != nil {
				return err
			}

			members, err := parseMembers(args[3])
			if err != nil {
				return err
			}

			policy, err := parseDecisionPolicy(clientCtx.Codec, args[4])
			if err != nil {
				return err
			}

			msg, err := group.NewMsgCreateGroupWithPolicy(
				clientCtx.GetFromAddress().String(),
				members,
				args[1],
				args[2],
				groupPolicyAsAdmin,
				policy,
			)
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
	cmd.Flags().Bool(FlagGroupPolicyAsAdmin, false, "Sets admin of the newly created group and group policy with group policy address itself when true")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgCreateGroupPolicyCmd creates a CLI command for Msg/CreateGroupPolicy.
func MsgCreateGroupPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group-policy [admin] [group-id] [metadata] [decision-policy-json-file]",
		Short: `Create a group policy which is an account associated with a group and a decision policy. Note, the '--from' flag is ignored as it is implied from [admin].`,
		Example: fmt.Sprintf(`
%s tx group create-group-policy [admin] [group-id] [metadata] policy.json

where policy.json contains:

{
    "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
    "threshold": "1",
    "windows": {
        "voting_period": "120h",
        "min_execution_period": "0s"
    }
}

Here, we can use percentage decision policy when needed, where 0 < percentage <= 1:

{
    "@type": "/cosmos.group.v1.PercentageDecisionPolicy",
    "percentage": "0.5",
    "windows": {
        "voting_period": "120h",
        "min_execution_period": "0s"
    }
}`, version.AppName),
		Args: cobra.ExactArgs(4),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			policy, err := parseDecisionPolicy(clientCtx.Codec, args[3])
			if err != nil {
				return err
			}

			msg, err := group.NewMsgCreateGroupPolicy(
				clientCtx.GetFromAddress(),
				groupID,
				args[2],
				policy,
			)
			if err != nil {
				return err
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupPolicyAdminCmd creates a CLI command for Msg/UpdateGroupPolicyAdmin.
func MsgUpdateGroupPolicyAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-policy-admin [admin] [group-policy-account] [new-admin]",
		Short: "Update a group policy admin",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupPolicyAdmin{
				Admin:              clientCtx.GetFromAddress().String(),
				GroupPolicyAddress: args[1],
				NewAdmin:           args[2],
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupPolicyDecisionPolicyCmd creates a CLI command for Msg/UpdateGroupPolicyDecisionPolicy.
func MsgUpdateGroupPolicyDecisionPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-policy-decision-policy [admin] [group-policy-account] [decision-policy-json-file]",
		Short: "Update a group policy's decision policy",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			policy, err := parseDecisionPolicy(clientCtx.Codec, args[2])
			if err != nil {
				return err
			}

			accountAddress, err := sdk.AccAddressFromBech32(args[1])
			if err != nil {
				return err
			}

			msg, err := group.NewMsgUpdateGroupPolicyDecisionPolicy(
				clientCtx.GetFromAddress(),
				accountAddress,
				policy,
			)
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupPolicyMetadataCmd creates a CLI command for Msg/UpdateGroupPolicyMetadata.
func MsgUpdateGroupPolicyMetadataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-policy-metadata [admin] [group-policy-account] [new-metadata]",
		Short: "Update a group policy metadata",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[0])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupPolicyMetadata{
				Admin:              clientCtx.GetFromAddress().String(),
				GroupPolicyAddress: args[1],
				Metadata:           args[2],
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgSubmitProposalCmd creates a CLI command for Msg/SubmitProposal.
func MsgSubmitProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "submit-proposal [proposal_json_file]",
		Short: "Submit a new proposal",
		Long: `Submit a new proposal.
Parameters:
			msg_tx_json_file: path to json file with messages that will be executed if the proposal is accepted.`,
		Example: fmt.Sprintf(`
%s tx group submit-proposal path/to/proposal.json
	
	Where proposal.json contains:

{
	"group_policy_address": "cosmos1...",
	// array of proto-JSON-encoded sdk.Msgs
	"messages": [
	{
		"@type": "/cosmos.bank.v1beta1.MsgSend",
		"from_address": "cosmos1...",
		"to_address": "cosmos1...",
		"amount":[{"denom": "stake","amount": "10"}]
	}
	],
	// metadata can be any of base64 encoded, raw text, stringified json, IPFS link to json
	// see below for example metadata
	"metadata": "4pIMOgIGx1vZGU=", // base64-encoded metadata
	"title": "My proposal",
	"summary": "This is a proposal to send 10 stake to cosmos1...",
	"proposers": ["cosmos1...", "cosmos1..."],
}

metadata example: 
{
	"title": "",
	"authors": [""],
	"summary": "",
	"details": "", 
	"proposal_forum_url": "",
	"vote_option_context": "",
} 
`, version.AppName),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			prop, err := getCLIProposal(args[0])
			if err != nil {
				return err
			}

			// Since the --from flag is not required on this CLI command, we
			// ignore it, and just use the 1st proposer in the JSON file.
			if prop.Proposers == nil || len(prop.Proposers) == 0 {
				return errors.New("no proposers specified in proposal")
			}
			cmd.Flags().Set(flags.FlagFrom, prop.Proposers[0])

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msgs, err := parseMsgs(clientCtx.Codec, prop)
			if err != nil {
				return err
			}

			execStr, _ := cmd.Flags().GetString(FlagExec)

			msg, err := group.NewMsgSubmitProposal(
				prop.GroupPolicyAddress,
				prop.Proposers,
				msgs,
				prop.Metadata,
				execFromString(execStr),
				prop.Title,
				prop.Summary,
			)
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(FlagExec, "", "Set to 1 to try to execute proposal immediately after creation (proposers signatures are considered as Yes votes)")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgWithdrawProposalCmd creates a CLI command for Msg/WithdrawProposal.
func MsgWithdrawProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "withdraw-proposal [proposal-id] [group-policy-admin-or-proposer]",
		Short: "Withdraw a submitted proposal",
		Long: `Withdraw a submitted proposal.

Parameters:
			proposal-id: unique ID of the proposal.
			group-policy-admin-or-proposer: either admin of the group policy or one the proposer of the proposal.
			Note: --from flag will be ignored here.
`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[1])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgWithdrawProposal{
				ProposalId: proposalID,
				Address:    clientCtx.GetFromAddress().String(),
			}

			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgVoteCmd creates a CLI command for Msg/Vote.
func MsgVoteCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "vote [proposal-id] [voter] [vote-option] [metadata]",
		Short: "Vote on a proposal",
		Long: `Vote on a proposal.

Parameters:
			proposal-id: unique ID of the proposal
			voter: voter account addresses.
			vote-option: choice of the voter(s)
				VOTE_OPTION_UNSPECIFIED: no-op
				VOTE_OPTION_NO: no
				VOTE_OPTION_YES: yes
				VOTE_OPTION_ABSTAIN: abstain
				VOTE_OPTION_NO_WITH_VETO: no-with-veto
			Metadata: metadata for the vote
`,
		Args: cobra.ExactArgs(4),
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Flags().Set(flags.FlagFrom, args[1])
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			voteOption, err := group.VoteOptionFromString(args[2])
			if err != nil {
				return err
			}

			execStr, _ := cmd.Flags().GetString(FlagExec)

			msg := &group.MsgVote{
				ProposalId: proposalID,
				Voter:      args[1],
				Option:     voteOption,
				Metadata:   args[3],
				Exec:       execFromString(execStr),
			}
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(FlagExec, "", "Set to 1 to try to execute proposal immediately after voting")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgExecCmd creates a CLI command for Msg/Exec.
func MsgExecCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "exec [proposal-id]",
		Short: "Execute a proposal",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgExec{
				ProposalId: proposalID,
				Executor:   clientCtx.GetFromAddress().String(),
			}
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgLeaveGroupCmd creates a CLI command for Msg/LeaveGroup.
func MsgLeaveGroupCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "leave-group [member-address] [group-id]",
		Short: "Remove member from the group",
		Long: `Remove member from the group

Parameters:
		   group-id: unique id of the group
		   member-address: account address of the group member
		   Note, the '--from' flag is ignored as it is implied from [member-address]
		`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			cmd.Flags().Set(flags.FlagFrom, args[0])
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			msg := &group.MsgLeaveGroup{
				Address: clientCtx.GetFromAddress().String(),
				GroupId: groupID,
			}
			if err = msg.ValidateBasic(); err != nil {
				return fmt.Errorf("message validation failed: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}
