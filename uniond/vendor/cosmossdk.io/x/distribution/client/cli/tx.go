package cli

import (
	"errors"
	"fmt"

	"github.com/spf13/cobra"

	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
)

// Transaction flags for the x/distribution module
var (
	FlagCommission       = "commission"
	FlagMaxMessagesPerTx = "max-msgs"
)

const (
	MaxMessagesPerTxDefault = 0
)

// NewTxCmd returns a root CLI command handler for all x/distribution transaction commands.
func NewTxCmd() *cobra.Command {
	distTxCmd := &cobra.Command{
		Use:                        types.ModuleName,
		Short:                      "Distribution transactions subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	distTxCmd.AddCommand(NewWithdrawAllRewardsCmd())

	return distTxCmd
}

// NewWithdrawAllRewardsCmd returns a CLI command handler for creating a MsgWithdrawDelegatorReward transaction.
// This command is more powerful than AutoCLI generated command as it allows sending batch of messages.
func NewWithdrawAllRewardsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "withdraw-all-rewards",
		Short:   "Withdraw all delegations rewards for a delegator",
		Example: fmt.Sprintf("%s tx distribution withdraw-all-rewards --from mykey", version.AppName),
		Args:    cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			delAddr, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			// The transaction cannot be generated offline since it requires a query
			// to get all the validators.
			if clientCtx.Offline {
				return errors.New("cannot generate tx in offline mode")
			}

			queryClient := types.NewQueryClient(clientCtx)
			delValsRes, err := queryClient.DelegatorValidators(cmd.Context(), &types.QueryDelegatorValidatorsRequest{DelegatorAddress: delAddr})
			if err != nil {
				return err
			}

			validators := delValsRes.Validators
			// build multi-message transaction
			msgs := make([]sdk.Msg, 0, len(validators))
			for _, valAddr := range validators {
				_, err := clientCtx.ValidatorAddressCodec.StringToBytes(valAddr)
				if err != nil {
					return err
				}

				msg := types.NewMsgWithdrawDelegatorReward(delAddr, valAddr)
				msgs = append(msgs, msg)
			}

			chunkSize, _ := cmd.Flags().GetInt(FlagMaxMessagesPerTx)
			if chunkSize == 0 {
				return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msgs...)
			}

			// split messages into slices of length chunkSize
			totalMessages := len(msgs)
			for i := 0; i < len(msgs); i += chunkSize {

				sliceEnd := i + chunkSize
				if sliceEnd > totalMessages {
					sliceEnd = totalMessages
				}

				msgChunk := msgs[i:sliceEnd]
				if err := tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msgChunk...); err != nil {
					return err
				}
			}

			return nil
		},
	}

	cmd.Flags().Int(FlagMaxMessagesPerTx, MaxMessagesPerTxDefault, "Limit the number of messages per tx (0 for unlimited)")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}
