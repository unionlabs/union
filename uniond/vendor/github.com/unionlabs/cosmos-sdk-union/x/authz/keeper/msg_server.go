package keeper

import (
	"context"
	"errors"
	"strings"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/authz"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ authz.MsgServer = Keeper{}

// Grant implements the MsgServer.Grant method to create a new grant.
func (k Keeper) Grant(ctx context.Context, msg *authz.MsgGrant) (*authz.MsgGrantResponse, error) {
	if strings.EqualFold(msg.Grantee, msg.Granter) {
		return nil, authz.ErrGranteeIsGranter
	}

	grantee, err := k.authKeeper.AddressCodec().StringToBytes(msg.Grantee)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid grantee address: %s", err)
	}

	granter, err := k.authKeeper.AddressCodec().StringToBytes(msg.Granter)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid granter address: %s", err)
	}

	if err := msg.Grant.ValidateBasic(); err != nil {
		return nil, err
	}

	authorization, err := msg.GetAuthorization()
	if err != nil {
		return nil, err
	}

	t := authorization.MsgTypeURL()
	if err := k.MsgRouterService.CanInvoke(ctx, t); err != nil {
		return nil, sdkerrors.ErrInvalidType.Wrapf("%s doesn't exist", t)
	}

	// Disable granting other accounts with grant permission.
	// Preventing user from accidentally authorizing their entire account to a different account.
	if t == sdk.MsgTypeURL(&authz.MsgGrant{}) {
		return nil, sdkerrors.ErrInvalidType.Wrap("authz msgGrant is not allowed")
	}

	err = k.SaveGrant(ctx, grantee, granter, authorization, msg.Grant.Expiration)
	if err != nil {
		return nil, err
	}

	return &authz.MsgGrantResponse{}, nil
}

// Revoke implements the MsgServer.Revoke method.
func (k Keeper) Revoke(ctx context.Context, msg *authz.MsgRevoke) (*authz.MsgRevokeResponse, error) {
	if strings.EqualFold(msg.Grantee, msg.Granter) {
		return nil, authz.ErrGranteeIsGranter
	}

	grantee, err := k.authKeeper.AddressCodec().StringToBytes(msg.Grantee)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid grantee address: %s", err)
	}

	granter, err := k.authKeeper.AddressCodec().StringToBytes(msg.Granter)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid granter address: %s", err)
	}

	if msg.MsgTypeUrl == "" {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("missing msg method name")
	}

	if err = k.DeleteGrant(ctx, grantee, granter, msg.MsgTypeUrl); err != nil {
		return nil, err
	}

	return &authz.MsgRevokeResponse{}, nil
}

// RevokeAll implements the MsgServer.RevokeAll method.
func (k Keeper) RevokeAll(ctx context.Context, msg *authz.MsgRevokeAll) (*authz.MsgRevokeAllResponse, error) {
	granter, err := k.authKeeper.AddressCodec().StringToBytes(msg.Granter)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid granter address: %s", err)
	}

	if err := k.DeleteAllGrants(ctx, granter); err != nil {
		return nil, err
	}

	return &authz.MsgRevokeAllResponse{}, nil
}

// Exec implements the MsgServer.Exec method.
func (k Keeper) Exec(ctx context.Context, msg *authz.MsgExec) (*authz.MsgExecResponse, error) {
	if msg.Grantee == "" {
		return nil, errors.New("empty address string is not allowed")
	}

	grantee, err := k.authKeeper.AddressCodec().StringToBytes(msg.Grantee)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid grantee address: %s", err)
	}

	if len(msg.Msgs) == 0 {
		return nil, sdkerrors.ErrInvalidRequest.Wrapf("messages cannot be empty")
	}

	msgs, err := msg.GetMessages()
	if err != nil {
		return nil, err
	}

	if err := validateMsgs(msgs); err != nil {
		return nil, err
	}

	results, err := k.DispatchActions(ctx, grantee, msgs)
	if err != nil {
		return nil, err
	}

	return &authz.MsgExecResponse{Results: results}, nil
}

func (k Keeper) PruneExpiredGrants(ctx context.Context, msg *authz.MsgPruneExpiredGrants) (*authz.MsgPruneExpiredGrantsResponse, error) {
	// 75 is an arbitrary value, we can change it later if needed
	if err := k.DequeueAndDeleteExpiredGrants(ctx, 75); err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).Emit(&authz.EventPruneExpiredGrants{Pruner: msg.Pruner}); err != nil {
		return nil, err
	}

	return &authz.MsgPruneExpiredGrantsResponse{}, nil
}

func validateMsgs(msgs []sdk.Msg) error {
	for i, msg := range msgs {
		m, ok := msg.(sdk.HasValidateBasic)
		if !ok {
			continue
		}

		if err := m.ValidateBasic(); err != nil {
			return errorsmod.Wrapf(err, "msg %d", i)
		}
	}

	return nil
}
