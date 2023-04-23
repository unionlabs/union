package rpc

import (
	"context"
	"fmt"
	"strconv"
	"strings"

	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

// TODO these next two functions feel kinda hacky based on their placement

// ValidatorCommand returns the validator set for a given height
func ValidatorCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tendermint-validator-set [height]",
		Short: "Get the full tendermint validator set at given height",
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			var height *int64

			// optional height
			if len(args) > 0 {
				val, err := strconv.ParseInt(args[0], 10, 64)
				if err != nil {
					return err
				}

				if val > 0 {
					height = &val
				}
			}

			page, _ := cmd.Flags().GetInt(flags.FlagPage)
			limit, _ := cmd.Flags().GetInt(flags.FlagLimit)

			result, err := GetValidators(cmd.Context(), clientCtx, height, &page, &limit)
			if err != nil {
				return err
			}

			return clientCtx.PrintObjectLegacy(result)
		},
	}

	cmd.Flags().String(flags.FlagNode, "tcp://localhost:26657", "<host>:<port> to Tendermint RPC interface for this chain")
	cmd.Flags().StringP(flags.FlagOutput, "o", "text", "Output format (text|json)")
	cmd.Flags().Int(flags.FlagPage, query.DefaultPage, "Query a specific page of paginated results")
	cmd.Flags().Int(flags.FlagLimit, 100, "Query number of results returned per page")

	return cmd
}

// Validator output
type ValidatorOutput struct {
	Address          sdk.ConsAddress    `json:"address"`
	PubKey           cryptotypes.PubKey `json:"pub_key"`
	ProposerPriority int64              `json:"proposer_priority"`
	VotingPower      int64              `json:"voting_power"`
}

// Validators at a certain height output in bech32 format
type ResultValidatorsOutput struct {
	BlockHeight int64             `json:"block_height"`
	Validators  []ValidatorOutput `json:"validators"`
	Total       uint64            `json:"total"`
}

func (rvo ResultValidatorsOutput) String() string {
	var b strings.Builder

	fmt.Fprintf(&b, "block height: %d\n", rvo.BlockHeight)
	fmt.Fprintf(&b, "total count: %d\n", rvo.Total)

	for _, val := range rvo.Validators {
		fmt.Fprintf(&b, `
  Address:          %s
  Pubkey:           %s
  ProposerPriority: %d
  VotingPower:      %d
		`,
			val.Address, val.PubKey, val.ProposerPriority, val.VotingPower,
		)
	}

	return b.String()
}

func validatorOutput(validator *tmtypes.Validator) (ValidatorOutput, error) {
	pk, err := cryptocodec.FromTmPubKeyInterface(validator.PubKey)
	if err != nil {
		return ValidatorOutput{}, err
	}

	return ValidatorOutput{
		Address:          sdk.ConsAddress(validator.Address),
		PubKey:           pk,
		ProposerPriority: validator.ProposerPriority,
		VotingPower:      validator.VotingPower,
	}, nil
}

// GetValidators from client
func GetValidators(ctx context.Context, clientCtx client.Context, height *int64, page, limit *int) (ResultValidatorsOutput, error) {
	// get the node
	node, err := clientCtx.GetNode()
	if err != nil {
		return ResultValidatorsOutput{}, err
	}

	validatorsRes, err := node.Validators(ctx, height, page, limit)
	if err != nil {
		return ResultValidatorsOutput{}, err
	}

	total := validatorsRes.Total
	if validatorsRes.Total < 0 {
		total = 0
	}
	out := ResultValidatorsOutput{
		BlockHeight: validatorsRes.BlockHeight,
		Validators:  make([]ValidatorOutput, len(validatorsRes.Validators)),
		Total:       uint64(total),
	}
	for i := 0; i < len(validatorsRes.Validators); i++ {
		out.Validators[i], err = validatorOutput(validatorsRes.Validators[i])
		if err != nil {
			return out, err
		}
	}

	return out, nil
}
