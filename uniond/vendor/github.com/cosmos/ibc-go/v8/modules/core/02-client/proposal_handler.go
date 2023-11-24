package client

import (
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

// NewClientProposalHandler defines the 02-client legacy proposal handler.
//
// Deprecated: This function is deprecated and will be removed in a future release.
// Please use MsgRecoverClient and MsgIBCSoftwareUpgrade in favour of this legacy Handler.
func NewClientProposalHandler(k keeper.Keeper) govtypes.Handler { //nolint:staticcheck
	return func(ctx sdk.Context, content govtypes.Content) error {
		switch c := content.(type) {
		case *types.ClientUpdateProposal:
			// NOTE: RecoverClient is called in favour of the deprecated ClientUpdateProposal function.
			return k.RecoverClient(ctx, c.SubjectClientId, c.SubstituteClientId)
		default:
			return errorsmod.Wrapf(ibcerrors.ErrUnknownRequest, "unrecognized ibc proposal content type: %T", c)
		}
	}
}
