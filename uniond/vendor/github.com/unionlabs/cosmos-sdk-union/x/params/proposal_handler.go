package params

import (
	"context"
	"fmt"

	errorsmod "cosmossdk.io/errors"
	govtypes "cosmossdk.io/x/gov/types/v1beta1"
	"cosmossdk.io/x/params/keeper"
	"cosmossdk.io/x/params/types/proposal"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// NewParamChangeProposalHandler creates a new governance Handler for a ParamChangeProposal
func NewParamChangeProposalHandler(k keeper.Keeper) govtypes.Handler {
	return func(ctx context.Context, content govtypes.Content) error {
		// UnwrapSDKContext makes x/params baseapp compatible only and not server/v2
		// We should investigate if we want to make x/params server/v2 compatible
		sdkCtx := sdk.UnwrapSDKContext(ctx)

		switch c := content.(type) {
		case *proposal.ParameterChangeProposal:
			return handleParameterChangeProposal(sdkCtx, k, c)

		default:
			return errorsmod.Wrapf(sdkerrors.ErrUnknownRequest, "unrecognized param proposal content type: %T", c)
		}
	}
}

func handleParameterChangeProposal(ctx sdk.Context, k keeper.Keeper, p *proposal.ParameterChangeProposal) error {
	for _, c := range p.Changes {
		ss, ok := k.GetSubspace(c.Subspace)
		if !ok {
			return errorsmod.Wrap(proposal.ErrUnknownSubspace, c.Subspace)
		}

		k.Logger(ctx).Info(
			fmt.Sprintf("attempt to set new parameter value; key: %s, value: %s", c.Key, c.Value),
		)

		if err := ss.Update(ctx, []byte(c.Key), []byte(c.Value)); err != nil {
			return errorsmod.Wrapf(proposal.ErrSettingParameter, "key: %s, value: %s, err: %s", c.Key, c.Value, err.Error())
		}
	}

	return nil
}
