package cli

import (
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/spf13/cobra"

	"cosmossdk.io/x/authz"
	bank "cosmossdk.io/x/bank/types"
	staking "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
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
func GetTxCmd() *cobra.Command {
	authorizationTxCmd := &cobra.Command{
		Use:                        authz.ModuleName,
		Short:                      "Authorization transactions subcommands",
		Long:                       "Authorize and revoke access to execute transactions on behalf of your address",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	authorizationTxCmd.AddCommand(
		NewCmdGrantAuthorization(),
		NewCmdExecAuthorization(),
	)

	return authorizationTxCmd
}

// NewCmdExecAuthorization returns a CLI command handler for creating a MsgExec transaction.
// Deprecated: This command is deprecated in favor for the AutoCLI exec command.
// It stays here for backward compatibility, as the AutoCLI command has a small breaking change,
// but it will be removed in future versions.
func NewCmdExecAuthorization() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "legacy-exec <tx-json-file> --from <grantee>",
		Short:   "Execute tx on behalf of granter account. Deprecated, use exec instead.",
		Example: fmt.Sprintf("$ %s tx authz exec tx.json --from grantee\n $ %[1]s tx bank send [granter] [recipient] [amount] --generate-only tx.json && %[1]s tx authz exec tx.json --from grantee", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			grantee, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

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

// NewCmdGrantAuthorization returns a CLI command handler for creating a MsgGrant transaction.
// Migrating this command to AutoCLI is possible but would be CLI breaking.
func NewCmdGrantAuthorization() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grant <grantee> <authorization_type=\"send\"|\"generic\"|\"delegate\"|\"unbond\"|\"redelegate\"> --from <granter>",
		Short: "Grant authorization to an address",
		Long: fmt.Sprintf(`create a new grant authorization to an address to execute a transaction on your behalf:
Examples:
 $ %[1]s tx authz grant cosmos1skjw.. send --spend-limit=1000stake --from=cosmos1skl..
 $ %[1]s tx authz grant cosmos1skjw.. generic --msg-type=/cosmos.gov.v1.MsgVote --from=cosmos1sk..
	`, version.AppName),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			grantee := args[0]
			if _, err := clientCtx.AddressCodec.StringToBytes(grantee); err != nil {
				return err
			}

			granter, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			if strings.EqualFold(grantee, granter) {
				return errors.New("grantee and granter should be different")
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
					return errors.New("spend-limit should be greater than zero")
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

				allowed, err := bech32toAccAddresses(clientCtx, allowList)
				if err != nil {
					return err
				}

				authorization = bank.NewSendAuthorization(spendLimit, allowed, clientCtx.AddressCodec)

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
						return errors.New("spend-limit should be greater than zero")
					}
					delegateLimit = &spendLimit
				}

				allowed, err := bech32toValAddresses(clientCtx, allowValidators)
				if err != nil {
					return err
				}

				denied, err := bech32toValAddresses(clientCtx, denyValidators)
				if err != nil {
					return err
				}

				switch args[1] {
				case delegate:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, delegateLimit, clientCtx.ValidatorAddressCodec)
				case unbond:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE, delegateLimit, clientCtx.ValidatorAddressCodec)
				default:
					authorization, err = staking.NewStakeAuthorization(allowed, denied, staking.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE, delegateLimit, clientCtx.ValidatorAddressCodec)
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

			msg, err := authz.NewMsgGrant(granter, grantee, authorization, expire)
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

// bech32toValAddresses returns []ValAddress from a list of Bech32 string addresses.
func bech32toValAddresses(clientCtx client.Context, validators []string) ([]sdk.ValAddress, error) {
	vals := make([]sdk.ValAddress, len(validators))
	for i, validator := range validators {
		addr, err := clientCtx.ValidatorAddressCodec.StringToBytes(validator)
		if err != nil {
			return nil, err
		}
		vals[i] = addr
	}
	return vals, nil
}

// bech32toAccAddresses returns []AccAddress from a list of Bech32 string addresses.
func bech32toAccAddresses(clientCtx client.Context, accAddrs []string) ([]sdk.AccAddress, error) {
	addrs := make([]sdk.AccAddress, len(accAddrs))
	for i, addr := range accAddrs {
		accAddr, err := clientCtx.AddressCodec.StringToBytes(addr)
		if err != nil {
			return nil, err
		}
		addrs[i] = accAddr
	}
	return addrs, nil
}
