package cli

import (
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/spf13/cobra"

	"cosmossdk.io/core/address"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
	"github.com/cosmos/cosmos-sdk/x/authz"
	bank "github.com/cosmos/cosmos-sdk/x/bank/types"
	staking "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// Flag names and values
const (
	FlagSpendLimit        = "spend-limit"
	FlagMsgType           = "msg-type"
	FlagExpiration        = "expiration"
	FlagAllowedValidators = "allowed-validators"
	FlagDenyValidators    = "deny-validators"
	FlagAllowList         = "allow-list"
	delegate              = "delegate"
	redelegate            = "redelegate"
	unbond                = "unbond"
)

// GetTxCmd returns the transaction commands for this module
func GetTxCmd(ac address.Codec) *cobra.Command {
	AuthorizationTxCmd := &cobra.Command{
		Use:                        authz.ModuleName,
		Short:                      "Authorization transactions subcommands",
		Long:                       "Authorize and revoke access to execute transactions on behalf of your address",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	AuthorizationTxCmd.AddCommand(
		NewCmdGrantAuthorization(ac),
		NewCmdRevokeAuthorization(ac),
		NewCmdExecAuthorization(),
	)

	return AuthorizationTxCmd
}

// NewCmdGrantAuthorization returns a CLI command handler for creating a MsgGrant transaction.
func NewCmdGrantAuthorization(ac address.Codec) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grant <grantee> <authorization_type=\"send\"|\"generic\"|\"delegate\"|\"unbond\"|\"redelegate\"> --from <granter>",
		Short: "Grant authorization to an address",
		Long: strings.TrimSpace(
			fmt.Sprintf(`create a new grant authorization to an address to execute a transaction on your behalf:

Examples:
 $ %s tx %s grant cosmos1skjw.. send --spend-limit=1000stake --from=cosmos1skl..
 $ %s tx %s grant cosmos1skjw.. generic --msg-type=/cosmos.gov.v1.MsgVote --from=cosmos1sk..
	`, version.AppName, authz.ModuleName, version.AppName, authz.ModuleName),
		),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			if strings.EqualFold(args[0], clientCtx.GetFromAddress().String()) {
				return errors.New("grantee and granter should be different")
			}

			grantee, err := ac.StringToBytes(args[0])
			if err != nil {
				return err
			}

			var authorization authz.Authorization
			switch args[1] {
			case "send":
				limit, err := cmd.Flags().GetString(FlagSpendLimit)
				if err != nil {
					return err
				}

				spendLimit, err := sdk.ParseCoinsNormalized(limit)
				if err != nil {
					return err
				}

				if !spendLimit.IsAllPositive() {
					return fmt.Errorf("spend-limit should be greater than zero")
				}

				allowList, err := cmd.Flags().GetStringSlice(FlagAllowList)
				if err != nil {
					return err
				}

				// check for duplicates
				for i := 0; i < len(allowList); i++ {
					for j := i + 1; j < len(allowList); j++ {
						if allowList[i] == allowList[j] {
							return fmt.Errorf("duplicate address %s in allow-list", allowList[i])
						}
					}
				}

				allowed, err := bech32toAccAddresses(allowList, ac)
				if err != nil {
					return err
				}

				authorization = bank.NewSendAuthorization(spendLimit, allowed)

			case "generic":
				msgType, err := cmd.Flags().GetString(FlagMsgType)
				if err != nil {
					return err
				}

				authorization = authz.NewGenericAuthorization(msgType)
			case delegate, unbond, redelegate:
				limit, err := cmd.Flags().GetString(FlagSpendLimit)
				if err != nil {
					return err
				}

				allowValidators, err := cmd.Flags().GetStringSlice(FlagAllowedValidators)
				if err != nil {
					return err
				}

				denyValidators, err := cmd.Flags().GetStringSlice(FlagDenyValidators)
				if err != nil {
					return err
				}

				var delegateLimit *sdk.Coin
				if limit != "" {
					spendLimit, err := sdk.ParseCoinNormalized(limit)
					if err != nil {
						return err
					}
					queryClient := staking.NewQueryClient(clientCtx)

					res, err := queryClient.Params(cmd.Context(), &staking.QueryParamsRequest{})
					if err != nil {
						return err
					}

					if spendLimit.Denom != res.Params.BondDenom {
						return fmt.Errorf("invalid denom %s; coin denom should match the current bond denom %s", spendLimit.Denom, res.Params.BondDenom)
					}

					if !spendLimit.IsPositive() {
						return fmt.Errorf("spend-limit should be greater than zero")
					}
					delegateLimit = &spendLimit
				}

				allowed, err := bech32toValAddresses(allowValidators)
				if err != nil {
					return err
				}

				denied, err := bech32toValAddresses(denyValidators)
				if err != nil {
					return err
				}

				switch args[1] {
				case delegate:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, delegateLimit)
				case unbond:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE, delegateLimit)
				default:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE, delegateLimit)
				}
				if err != nil {
					return err
				}

			default:
				return fmt.Errorf("invalid authorization type, %s", args[1])
			}

			expire, err := getExpireTime(cmd)
			if err != nil {
				return err
			}

			msg, err := authz.NewMsgGrant(clientCtx.GetFromAddress(), grantee, authorization, expire)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
	flags.AddTxFlagsToCmd(cmd)
	cmd.Flags().String(FlagMsgType, "", "The Msg method name for which we are creating a GenericAuthorization")
	cmd.Flags().String(FlagSpendLimit, "", "SpendLimit for Send Authorization, an array of Coins allowed spend")
	cmd.Flags().StringSlice(FlagAllowedValidators, []string{}, "Allowed validators addresses separated by ,")
	cmd.Flags().StringSlice(FlagDenyValidators, []string{}, "Deny validators addresses separated by ,")
	cmd.Flags().StringSlice(FlagAllowList, []string{}, "Allowed addresses grantee is allowed to send funds separated by ,")
	cmd.Flags().Int64(FlagExpiration, 0, "Expire time as Unix timestamp. Set zero (0) for no expiry. Default is 0.")
	return cmd
}

func getExpireTime(cmd *cobra.Command) (*time.Time, error) {
	exp, err := cmd.Flags().GetInt64(FlagExpiration)
	if err != nil {
		return nil, err
	}
	if exp == 0 {
		return nil, nil
	}
	e := time.Unix(exp, 0)
	return &e, nil
}

// NewCmdRevokeAuthorization returns a CLI command handler for creating a MsgRevoke transaction.
func NewCmdRevokeAuthorization(ac address.Codec) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "revoke [grantee] [msg-type-url] --from=[granter]",
		Short: "revoke authorization",
		Long: strings.TrimSpace(
			fmt.Sprintf(`revoke authorization from a granter to a grantee:
Example:
 $ %s tx %s revoke cosmos1skj.. %s --from=cosmos1skj..
			`, version.AppName, authz.ModuleName, bank.SendAuthorization{}.MsgTypeURL()),
		),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			grantee, err := ac.StringToBytes(args[0])
			if err != nil {
				return err
			}

			granter := clientCtx.GetFromAddress()
			msgAuthorized := args[1]
			msg := authz.NewMsgRevoke(granter, grantee, msgAuthorized)

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
	}
	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// NewCmdExecAuthorization returns a CLI command handler for creating a MsgExec transaction.
func NewCmdExecAuthorization() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "exec [tx-json-file] --from [grantee]",
		Short: "execute tx on behalf of granter account",
		Long: strings.TrimSpace(
			fmt.Sprintf(`execute tx on behalf of granter account:
Example:
 $ %s tx %s exec tx.json --from grantee
 $ %s tx bank send <granter> <recipient> --from <granter> --chain-id <chain-id> --generate-only > tx.json && %s tx %s exec tx.json --from grantee
			`, version.AppName, authz.ModuleName, version.AppName, version.AppName, authz.ModuleName),
		),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			grantee := clientCtx.GetFromAddress()

			if offline, _ := cmd.Flags().GetBool(flags.FlagOffline); offline {
				return errors.New("cannot broadcast tx during offline mode")
			}

			theTx, err := authclient.ReadTxFromFile(clientCtx, args[0])
			if err != nil {
				return err
			}
			msg := authz.NewMsgExec(grantee, theTx.GetMsgs())

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

// bech32toValAddresses returns []ValAddress from a list of Bech32 string addresses.
func bech32toValAddresses(validators []string) ([]sdk.ValAddress, error) {
	vals := make([]sdk.ValAddress, len(validators))
	for i, validator := range validators {
		addr, err := sdk.ValAddressFromBech32(validator)
		if err != nil {
			return nil, err
		}
		vals[i] = addr
	}
	return vals, nil
}

// bech32toAccAddresses returns []AccAddress from a list of Bech32 string addresses.
func bech32toAccAddresses(accAddrs []string, ac address.Codec) ([]sdk.AccAddress, error) {
	addrs := make([]sdk.AccAddress, len(accAddrs))
	for i, addr := range accAddrs {
		accAddr, err := ac.StringToBytes(addr)
		if err != nil {
			return nil, err
		}
		addrs[i] = accAddr
	}
	return addrs, nil
}
