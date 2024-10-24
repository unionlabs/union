package staking

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"fmt"

	"cosmossdk.io/core/address"
	stakingcli "cosmossdk.io/x/staking/client/cli"
	stakingtypes "cosmossdk.io/x/staking/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/spf13/cobra"
)

var ()

// NewTxCmd returns a root CLI command handler for all x/staking transaction commands.
func NewTxCmd(ac address.Codec) *cobra.Command {
	stakingTxCmd := &cobra.Command{
		Use:                        "union-staking",
		Short:                      "Staking transaction subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	createValidatorCmd := stakingcli.NewCreateValidatorCmd()
	createUnionValidatorCmd := &cobra.Command{
		Use:   "create-union-validator [/path/to/validator.json] [proof_of_possession]",
		Short: "",
		Long:  ``,
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			proofOfPossession, err := hex.DecodeString(args[1])
			if err != nil {
				return fmt.Errorf("failed to decode proof of possession. The value must be a non-prefixed hex-encoded string: %w", err)
			}
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			previousOutput := clientCtx.Output
			previousGenerateOnly := clientCtx.GenerateOnly
			var msgBuffer bytes.Buffer
			writer := bufio.NewWriter(&msgBuffer)
			clientCtx.Output = writer
			clientCtx.GenerateOnly = true
			if err := client.SetCmdClientContext(cmd, clientCtx); err != nil {
				return fmt.Errorf("failed to update client context: %w", err)
			}
			err = createValidatorCmd.RunE(cmd, args)
			if err != nil {
				return fmt.Errorf("failed to create underlying msg: %w", err)
			}
			writer.Flush()
			tx, err := clientCtx.TxConfig.TxJSONDecoder()(msgBuffer.Bytes())
			if err != nil {
				return fmt.Errorf("failed to decode tx: %w", err)
			}
			msgs := tx.GetMsgs()
			underlying := msgs[0].(*stakingtypes.MsgCreateValidator)
			msg := MsgCreateUnionValidator{
				Underlying:        underlying,
				ProofOfPossession: proofOfPossession,
				ValidatorAddress:  underlying.ValidatorAddress,
			}
			clientCtx.Output = previousOutput
			clientCtx.GenerateOnly = previousGenerateOnly
			txf, err := clienttx.NewFactoryCLI(clientCtx, cmd.Flags())
			if err != nil {
				return err
			}
			return clienttx.GenerateOrBroadcastTxWithFactory(clientCtx, txf, &msg)
		},
	}

	stakingTxCmd.AddCommand(createUnionValidatorCmd)

	flags.AddTxFlagsToCmd(createUnionValidatorCmd)
	createUnionValidatorCmd.MarkFlagRequired(flags.FlagFrom)

	return stakingTxCmd
}
