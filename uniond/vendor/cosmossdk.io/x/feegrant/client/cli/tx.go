package cli

import (
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/spf13/cobra"

	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
)

// flag for feegrant module
const (
	FlagExpiration  = "expiration"
	FlagPeriod      = "period"
	FlagPeriodLimit = "period-limit"
	FlagSpendLimit  = "spend-limit"
	FlagAllowedMsgs = "allowed-messages"
)

// GetTxCmd returns the transaction commands for feegrant module
func GetTxCmd() *cobra.Command {
	feegrantTxCmd := &cobra.Command{
		Use:                        feegrant.ModuleName,
		Short:                      "Feegrant transactions sub-commands",
		Long:                       "Grant and revoke fee allowance for a grantee by a granter",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	feegrantTxCmd.AddCommand(
		NewCmdFeeGrant(),
	)

	return feegrantTxCmd
}

// NewCmdFeeGrant returns a CLI command handler to create a MsgGrantAllowance transaction.
// This command is more powerful than AutoCLI generated command as it allows a better input validation.
func NewCmdFeeGrant() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "grant <granter_key_or_address> <grantee>",
		Aliases: []string{"grant-allowance"},
		Short:   "Grant Fee allowance to an address",
		Long: strings.TrimSpace(
			fmt.Sprintf(
				`Grant authorization to pay fees from your address. Note, the '--from' flag is
				ignored as it is implied from [granter].

Examples:
%s tx %s grant cosmos1skjw... cosmos1skjw... --spend-limit 100stake --expiration 2022-01-30T15:04:05Z or
%s tx %s grant cosmos1skjw... cosmos1skjw... --spend-limit 100stake --period 3600 --period-limit 10stake --expiration 2022-01-30T15:04:05Z or
%s tx %s grant cosmos1skjw... cosmos1skjw... --spend-limit 100stake --expiration 2022-01-30T15:04:05Z 
	--allowed-messages "/cosmos.gov.v1beta1.MsgSubmitProposal,/cosmos.gov.v1beta1.MsgVote"
				`, version.AppName, feegrant.ModuleName, version.AppName, feegrant.ModuleName, version.AppName, feegrant.ModuleName,
			),
		),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := cmd.Flags().Set(flags.FlagFrom, args[0]); err != nil {
				return err
			}

			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			_, err = clientCtx.AddressCodec.StringToBytes(args[1])
			if err != nil {
				return err
			}

			granter := clientCtx.GetFromAddress()
			granterStr, err := clientCtx.AddressCodec.BytesToString(granter)
			if err != nil {
				return err
			}
			sl, err := cmd.Flags().GetString(FlagSpendLimit)
			if err != nil {
				return err
			}

			// if `FlagSpendLimit` isn't set, limit will be nil.
			// Hence, there won't be any spendlimit for the grantee.
			limit, err := sdk.ParseCoinsNormalized(sl)
			if err != nil {
				return err
			}

			exp, err := cmd.Flags().GetString(FlagExpiration)
			if err != nil {
				return err
			}

			basic := feegrant.BasicAllowance{
				SpendLimit: limit,
			}

			var expiresAtTime time.Time
			if exp != "" {
				expiresAtTime, err = time.Parse(time.RFC3339, exp)
				if err != nil {
					return err
				}
				basic.Expiration = &expiresAtTime
			}

			var grant feegrant.FeeAllowanceI
			grant = &basic

			periodClock, err := cmd.Flags().GetInt64(FlagPeriod)
			if err != nil {
				return err
			}

			periodLimitVal, err := cmd.Flags().GetString(FlagPeriodLimit)
			if err != nil {
				return err
			}

			// check any of period or periodLimit flags are set,
			// if set consider it as periodic fee allowance.
			if periodClock > 0 || periodLimitVal != "" {
				periodLimit, err := sdk.ParseCoinsNormalized(periodLimitVal)
				if err != nil {
					return err
				}

				if periodClock <= 0 {
					return errors.New("period clock was not set")
				}

				if periodLimit == nil {
					return errors.New("period limit was not set")
				}

				periodReset := getPeriodReset(periodClock)
				if exp != "" && periodReset.Sub(expiresAtTime) > 0 {
					return fmt.Errorf("period (%d) cannot reset after expiration (%v)", periodClock, exp)
				}

				periodic := feegrant.PeriodicAllowance{
					Basic:            basic,
					Period:           getPeriod(periodClock),
					PeriodSpendLimit: periodLimit,
					PeriodCanSpend:   periodLimit,
				}

				grant = &periodic
			}

			allowedMsgs, err := cmd.Flags().GetStringSlice(FlagAllowedMsgs)
			if err != nil {
				return err
			}

			if len(allowedMsgs) > 0 {
				grant, err = feegrant.NewAllowedMsgAllowance(grant, allowedMsgs)
				if err != nil {
					return err
				}
			}

			msg, err := feegrant.NewMsgGrantAllowance(grant, granterStr, args[1])
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)
	cmd.Flags().StringSlice(FlagAllowedMsgs, []string{}, "Set of allowed messages for fee allowance")
	cmd.Flags().String(FlagExpiration, "", "The RFC 3339 timestamp after which the grant expires for the user")
	cmd.Flags().String(FlagSpendLimit, "", "Spend limit specifies the max limit can be used, if not mentioned there is no limit")
	cmd.Flags().Int64(FlagPeriod, 0, "period specifies the time duration(in seconds) in which period_limit coins can be spent before that allowance is reset (ex: 3600)")
	cmd.Flags().String(FlagPeriodLimit, "", "period limit specifies the maximum number of coins that can be spent in the period")

	return cmd
}

func getPeriodReset(duration int64) time.Time {
	return time.Now().Add(getPeriod(duration))
}

func getPeriod(duration int64) time.Duration {
	return time.Duration(duration) * time.Second
}
