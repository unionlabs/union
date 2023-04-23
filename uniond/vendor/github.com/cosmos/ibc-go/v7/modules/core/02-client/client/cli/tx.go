package cli

import (
	"fmt"
	"os"
	"strconv"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	govcli "github.com/cosmos/cosmos-sdk/x/gov/client/cli"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"
	"github.com/spf13/cobra"

	"github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

// NewCreateClientCmd defines the command to create a new IBC light client.
func NewCreateClientCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create [path/to/client_state.json] [path/to/consensus_state.json]",
		Short: "create new IBC client",
		Long: `create a new IBC client with the specified client state and consensus state
	- ClientState JSON example: {"@type":"/ibc.lightclients.solomachine.v1.ClientState","sequence":"1","frozen_sequence":"0","consensus_state":{"public_key":{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AtK50+5pJOoaa04qqAqrnyAqsYrwrR/INnA6UPIaYZlp"},"diversifier":"testing","timestamp":"10"},"allow_update_after_proposal":false}
	- ConsensusState JSON example: {"@type":"/ibc.lightclients.solomachine.v1.ConsensusState","public_key":{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AtK50+5pJOoaa04qqAqrnyAqsYrwrR/INnA6UPIaYZlp"},"diversifier":"testing","timestamp":"10"}`,
		Example: fmt.Sprintf("%s tx ibc %s create [path/to/client_state.json] [path/to/consensus_state.json] --from node0 --home ../node0/<app>cli --chain-id $CID", version.AppName, types.SubModuleName),
		Args:    cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			// attempt to unmarshal client state argument
			var clientState exported.ClientState
			clientContentOrFileName := args[0]
			if err := cdc.UnmarshalInterfaceJSON([]byte(clientContentOrFileName), &clientState); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(clientContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for client state were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &clientState); err != nil {
					return fmt.Errorf("error unmarshalling client state file: %w", err)
				}
			}

			// attempt to unmarshal consensus state argument
			var consensusState exported.ConsensusState
			consensusContentOrFileName := args[1]
			if err := cdc.UnmarshalInterfaceJSON([]byte(consensusContentOrFileName), &consensusState); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(consensusContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for consensus state were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &consensusState); err != nil {
					return fmt.Errorf("error unmarshalling consensus state file: %w", err)
				}
			}

			msg, err := types.NewMsgCreateClient(clientState, consensusState, clientCtx.GetFromAddress().String())
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// NewUpdateClientCmd defines the command to update an IBC client.
func NewUpdateClientCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "update [client-id] [path/to/client_msg.json]",
		Short:   "update existing client with a client message",
		Long:    "update existing client with a client message, for example a header, misbehaviour or batch update",
		Example: fmt.Sprintf("%s tx ibc %s update [client-id] [path/to/client_msg.json] --from node0 --home ../node0/<app>cli --chain-id $CID", version.AppName, types.SubModuleName),
		Args:    cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			clientID := args[0]

			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			var clientMsg exported.ClientMessage
			clientMsgContentOrFileName := args[1]
			if err := cdc.UnmarshalInterfaceJSON([]byte(clientMsgContentOrFileName), &clientMsg); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(clientMsgContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for header were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &clientMsg); err != nil {
					return fmt.Errorf("error unmarshalling header file: %w", err)
				}
			}

			msg, err := types.NewMsgUpdateClient(clientID, clientMsg, clientCtx.GetFromAddress().String())
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// NewSubmitMisbehaviourCmd defines the command to submit a misbehaviour to prevent
// future updates.
// Deprecated: NewSubmitMisbehaviourCmd is deprecated and will be removed in a future release.
// Please use NewUpdateClientCmd instead.
func NewSubmitMisbehaviourCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "misbehaviour [clientID] [path/to/misbehaviour.json]",
		Short:   "submit a client misbehaviour",
		Long:    "submit a client misbehaviour to prevent future updates",
		Example: fmt.Sprintf("%s tx ibc %s misbehaviour [clientID] [path/to/misbehaviour.json] --from node0 --home ../node0/<app>cli --chain-id $CID", version.AppName, types.SubModuleName),
		Args:    cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			var misbehaviour exported.ClientMessage
			clientID := args[0]
			misbehaviourContentOrFileName := args[1]
			if err := cdc.UnmarshalInterfaceJSON([]byte(misbehaviourContentOrFileName), &misbehaviour); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(misbehaviourContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for misbehaviour were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, misbehaviour); err != nil {
					return fmt.Errorf("error unmarshalling misbehaviour file: %w", err)
				}
			}

			msg, err := types.NewMsgSubmitMisbehaviour(clientID, misbehaviour, clientCtx.GetFromAddress().String())
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// NewUpgradeClientCmd defines the command to upgrade an IBC light client.
func NewUpgradeClientCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "upgrade [client-identifier] [path/to/client_state.json] [path/to/consensus_state.json] [upgrade-client-proof] [upgrade-consensus-state-proof]",
		Short: "upgrade an IBC client",
		Long: `upgrade the IBC client associated with the provided client identifier while providing proof committed by the counterparty chain to the new client and consensus states
	- ClientState JSON example: {"@type":"/ibc.lightclients.solomachine.v1.ClientState","sequence":"1","frozen_sequence":"0","consensus_state":{"public_key":{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AtK50+5pJOoaa04qqAqrnyAqsYrwrR/INnA6UPIaYZlp"},"diversifier":"testing","timestamp":"10"},"allow_update_after_proposal":false}
	- ConsensusState JSON example: {"@type":"/ibc.lightclients.solomachine.v1.ConsensusState","public_key":{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AtK50+5pJOoaa04qqAqrnyAqsYrwrR/INnA6UPIaYZlp"},"diversifier":"testing","timestamp":"10"}`,
		Example: fmt.Sprintf("%s tx ibc %s upgrade [client-identifier] [path/to/client_state.json] [path/to/consensus_state.json] [client-state-proof] [consensus-state-proof] --from node0 --home ../node0/<app>cli --chain-id $CID", version.AppName, types.SubModuleName),
		Args:    cobra.ExactArgs(5),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)
			clientID := args[0]

			// attempt to unmarshal client state argument
			var clientState exported.ClientState
			clientContentOrFileName := args[1]
			if err := cdc.UnmarshalInterfaceJSON([]byte(clientContentOrFileName), &clientState); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(clientContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for client state were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &clientState); err != nil {
					return fmt.Errorf("error unmarshalling client state file: %w", err)
				}
			}

			// attempt to unmarshal consensus state argument
			var consensusState exported.ConsensusState
			consensusContentOrFileName := args[2]
			if err := cdc.UnmarshalInterfaceJSON([]byte(consensusContentOrFileName), &consensusState); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(consensusContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for consensus state were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &consensusState); err != nil {
					return fmt.Errorf("error unmarshalling consensus state file: %w", err)
				}
			}

			proofUpgradeClient := []byte(args[3])
			proofUpgradeConsensus := []byte(args[4])

			msg, err := types.NewMsgUpgradeClient(clientID, clientState, consensusState, proofUpgradeClient, proofUpgradeConsensus, clientCtx.GetFromAddress().String())
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// NewCmdSubmitUpdateClientProposal implements a command handler for submitting an update IBC client proposal transaction.
func NewCmdSubmitUpdateClientProposal() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update-client [subject-client-id] [substitute-client-id]",
		Args:  cobra.ExactArgs(2),
		Short: "Submit an update IBC client proposal",
		Long: "Submit an update IBC client proposal along with an initial deposit.\n" +
			"Please specify a subject client identifier you want to update..\n" +
			"Please specify the substitute client the subject client will be updated to.",
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			title, err := cmd.Flags().GetString(govcli.FlagTitle) //nolint:staticcheck // need this till full govv1 conversion.
			if err != nil {
				return err
			}

			description, err := cmd.Flags().GetString(govcli.FlagDescription) //nolint:staticcheck // need this till full govv1 conversion.
			if err != nil {
				return err
			}

			subjectClientID := args[0]
			substituteClientID := args[1]

			content := types.NewClientUpdateProposal(title, description, subjectClientID, substituteClientID)

			from := clientCtx.GetFromAddress()

			depositStr, err := cmd.Flags().GetString(govcli.FlagDeposit)
			if err != nil {
				return err
			}
			deposit, err := sdk.ParseCoinsNormalized(depositStr)
			if err != nil {
				return err
			}

			msg, err := govtypes.NewMsgSubmitProposal(content, deposit, from)
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(govcli.FlagTitle, "", "title of proposal")             //nolint:staticcheck // need this till full govv1 conversion.
	cmd.Flags().String(govcli.FlagDescription, "", "description of proposal") //nolint:staticcheck // need this till full govv1 conversion.
	cmd.Flags().String(govcli.FlagDeposit, "", "deposit of proposal")

	return cmd
}

// NewCmdSubmitUpgradeProposal implements a command handler for submitting an upgrade IBC client proposal transaction.
func NewCmdSubmitUpgradeProposal() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "ibc-upgrade [name] [height] [path/to/upgraded_client_state.json] [flags]",
		Args:  cobra.ExactArgs(3),
		Short: "Submit an IBC upgrade proposal",
		Long: "Submit an IBC client breaking upgrade proposal along with an initial deposit.\n" +
			"The client state specified is the upgraded client state representing the upgraded chain\n" +
			`Example Upgraded Client State JSON: 
{
	"@type":"/ibc.lightclients.tendermint.v1.ClientState",
 	"chain_id":"testchain1",
	"unbonding_period":"1814400s",
	"latest_height":{"revision_number":"0","revision_height":"2"},
	"proof_specs":[{"leaf_spec":{"hash":"SHA256","prehash_key":"NO_HASH","prehash_value":"SHA256","length":"VAR_PROTO","prefix":"AA=="},"inner_spec":{"child_order":[0,1],"child_size":33,"min_prefix_length":4,"max_prefix_length":12,"empty_child":null,"hash":"SHA256"},"max_depth":0,"min_depth":0},{"leaf_spec":{"hash":"SHA256","prehash_key":"NO_HASH","prehash_value":"SHA256","length":"VAR_PROTO","prefix":"AA=="},"inner_spec":{"child_order":[0,1],"child_size":32,"min_prefix_length":1,"max_prefix_length":1,"empty_child":null,"hash":"SHA256"},"max_depth":0,"min_depth":0}],
	"upgrade_path":["upgrade","upgradedIBCState"],
}
			`,
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			title, err := cmd.Flags().GetString(govcli.FlagTitle) //nolint:staticcheck // need this till full govv1 conversion.
			if err != nil {
				return err
			}

			description, err := cmd.Flags().GetString(govcli.FlagDescription) //nolint:staticcheck // need this till full govv1 conversion.
			if err != nil {
				return err
			}

			name := args[0]

			height, err := strconv.ParseInt(args[1], 10, 64)
			if err != nil {
				return err
			}

			plan := upgradetypes.Plan{
				Name:   name,
				Height: height,
			}

			// attempt to unmarshal client state argument
			var clientState exported.ClientState
			clientContentOrFileName := args[2]
			if err := cdc.UnmarshalInterfaceJSON([]byte(clientContentOrFileName), &clientState); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(clientContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for client state were provided: %w", err)
				}

				if err := cdc.UnmarshalInterfaceJSON(contents, &clientState); err != nil {
					return fmt.Errorf("error unmarshalling client state file: %w", err)
				}
			}

			content, err := types.NewUpgradeProposal(title, description, plan, clientState)
			if err != nil {
				return err
			}

			from := clientCtx.GetFromAddress()

			depositStr, err := cmd.Flags().GetString(govcli.FlagDeposit)
			if err != nil {
				return err
			}
			deposit, err := sdk.ParseCoinsNormalized(depositStr)
			if err != nil {
				return err
			}

			msg, err := govtypes.NewMsgSubmitProposal(content, deposit, from)
			if err != nil {
				return err
			}

			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(govcli.FlagTitle, "", "title of proposal")             //nolint:staticcheck // need this till full govv1 conversion.
	cmd.Flags().String(govcli.FlagDescription, "", "description of proposal") //nolint:staticcheck // need this till full govv1 conversion.
	cmd.Flags().String(govcli.FlagDeposit, "", "deposit of proposal")

	return cmd
}
