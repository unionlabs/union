package cli

import (
	"errors"
	"fmt"
	"strconv"

	"github.com/spf13/cobra"

	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/internal/math"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/version"
)

const (
	FlagExec               = "exec"
	ExecTry                = "try"
	FlagGroupPolicyAsAdmin = "group-policy-as-admin"
)

var errZeroGroupID = errors.New("group id cannot be 0")

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
		MsgUpdateGroupMembersCmd(),
		MsgCreateGroupWithPolicyCmd(),
		MsgCreateGroupPolicyCmd(),
		MsgUpdateGroupPolicyDecisionPolicyCmd(),
		MsgSubmitProposalCmd(),
		NewCmdDraftProposal(),
	)

	return txCmd
}

// MsgCreateGroupCmd creates a CLI command for Msg/CreateGroup.
//
// This command is being handled better here, not converting to autocli
func MsgCreateGroupCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group <admin> <metadata> <members-json-file>",
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

			for _, member := range members {
				if _, err := math.NewPositiveDecFromString(member.Weight); err != nil {
					return fmt.Errorf("invalid weight %s for %s: weight must be positive", member.Weight, member.Address)
				}
			}

			admin, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			msg := &group.MsgCreateGroup{
				Admin:    admin,
				Members:  members,
				Metadata: args[1],
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupMembersCmd creates a CLI command for Msg/UpdateGroupMembers.
//
// This command is being handled better here, not converting to autocli
func MsgUpdateGroupMembersCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-members <admin> <group-id> <members-json-file>",
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

			for _, member := range members {
				if _, err := math.NewNonNegativeDecFromString(member.Weight); err != nil {
					return fmt.Errorf("invalid weight %s for %s: weight must not be negative", member.Weight, member.Address)
				}
			}

			groupID, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return err
			}

			if groupID == 0 {
				return errZeroGroupID
			}

			admin, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			msg := &group.MsgUpdateGroupMembers{
				Admin:         admin,
				MemberUpdates: members,
				GroupId:       groupID,
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgCreateGroupWithPolicyCmd creates a CLI command for Msg/CreateGroupWithPolicy.
//
// This command is being handled better here, not converting to autocli
func MsgCreateGroupWithPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group-with-policy <admin> <group-metadata> <group-policy-metadata> <members-json-file> <decision-policy-json-file>",
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

			for _, member := range members {
				if _, err := math.NewPositiveDecFromString(member.Weight); err != nil {
					return fmt.Errorf("invalid weight %s for %s: weight must be positive", member.Weight, member.Address)
				}
			}

			policy, err := parseDecisionPolicy(clientCtx.Codec, args[4])
			if err != nil {
				return err
			}

			if err := policy.ValidateBasic(); err != nil {
				return err
			}

			admin, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			msg, err := group.NewMsgCreateGroupWithPolicy(
				admin,
				members,
				args[1],
				args[2],
				groupPolicyAsAdmin,
				policy,
			)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
	cmd.Flags().Bool(FlagGroupPolicyAsAdmin, false, "Sets admin of the newly created group and group policy with group policy address itself when true")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgCreateGroupPolicyCmd creates a CLI command for Msg/CreateGroupPolicy.
//
// This command is being handled better here, not converting to autocli
func MsgCreateGroupPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-group-policy <admin> <group-id> <metadata> <decision-policy-json-file>",
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

			if groupID == 0 {
				return errZeroGroupID
			}

			policy, err := parseDecisionPolicy(clientCtx.Codec, args[3])
			if err != nil {
				return err
			}

			if err := policy.ValidateBasic(); err != nil {
				return err
			}

			admin, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			msg, err := group.NewMsgCreateGroupPolicy(
				admin,
				groupID,
				args[2],
				policy,
			)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgUpdateGroupPolicyDecisionPolicyCmd creates a CLI command for Msg/UpdateGroupPolicyDecisionPolicy.
//
// This command is being handled better here, not converting to autocli
func MsgUpdateGroupPolicyDecisionPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-group-policy-decision-policy <admin> <group-policy-account> <decision-policy-json-file>",
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

			accountAddress, err := clientCtx.AddressCodec.StringToBytes(args[1])
			if err != nil {
				return err
			}

			adminAddr, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}
			accAddr, err := clientCtx.AddressCodec.BytesToString(accountAddress)
			if err != nil {
				return err
			}

			msg, err := group.NewMsgUpdateGroupPolicyDecisionPolicy(
				adminAddr,
				accAddr,
				policy,
			)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// MsgSubmitProposalCmd creates a CLI command for Msg/SubmitProposal.
//
// This command is being handled better here, not converting to autocli
func MsgSubmitProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "submit-proposal <proposal_json_file>",
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
			if len(prop.Proposers) == 0 {
				return errors.New("no proposers specified in proposal")
			}
			err = cmd.Flags().Set(flags.FlagFrom, prop.Proposers[0])
			if err != nil {
				return err
			}

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

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(FlagExec, "", "Set to 1 to try to execute proposal immediately after creation (proposers signatures are considered as Yes votes)")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}
