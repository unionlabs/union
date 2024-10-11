package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/errors"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

type msgServer struct {
	Keeper
}

var _ types.MsgServer = msgServer{}

// NewMsgServerImpl returns an implementation of the distribution MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

func (k msgServer) SetWithdrawAddress(ctx context.Context, msg *types.MsgSetWithdrawAddress) (*types.MsgSetWithdrawAddressResponse, error) {
	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	withdrawAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.WithdrawAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid withdraw address: %s", err)
	}

	err = k.SetWithdrawAddr(ctx, delegatorAddress, withdrawAddress)
	if err != nil {
		return nil, err
	}

	return &types.MsgSetWithdrawAddressResponse{}, nil
}

func (k msgServer) WithdrawDelegatorReward(ctx context.Context, msg *types.MsgWithdrawDelegatorReward) (*types.MsgWithdrawDelegatorRewardResponse, error) {
	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	amount, err := k.WithdrawDelegationRewards(ctx, delegatorAddress, valAddr)
	if err != nil {
		return nil, err
	}

	return &types.MsgWithdrawDelegatorRewardResponse{Amount: amount}, nil
}

func (k msgServer) WithdrawValidatorCommission(ctx context.Context, msg *types.MsgWithdrawValidatorCommission) (*types.MsgWithdrawValidatorCommissionResponse, error) {
	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	amount, err := k.Keeper.WithdrawValidatorCommission(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	return &types.MsgWithdrawValidatorCommissionResponse{Amount: amount}, nil
}

// Deprecated: DO NOT USE
// This method uses deprecated message request. Use FundCommunityPool from x/protocolpool module instead.
func (k msgServer) FundCommunityPool(ctx context.Context, msg *types.MsgFundCommunityPool) (*types.MsgFundCommunityPoolResponse, error) {
	depositor, err := k.authKeeper.AddressCodec().StringToBytes(msg.Depositor)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid depositor address: %s", err)
	}

	if err := validateAmount(msg.Amount); err != nil {
		return nil, err
	}

	if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, depositor, types.ProtocolPoolModuleName, msg.Amount); err != nil {
		return nil, err
	}

	return &types.MsgFundCommunityPoolResponse{}, nil
}

func (k msgServer) UpdateParams(ctx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if err := k.validateAuthority(msg.Authority); err != nil {
		return nil, err
	}

	if (!msg.Params.BaseProposerReward.IsNil() && !msg.Params.BaseProposerReward.IsZero()) || //nolint:staticcheck // deprecated but kept for backwards compatibility
		(!msg.Params.BonusProposerReward.IsNil() && !msg.Params.BonusProposerReward.IsZero()) { //nolint:staticcheck // deprecated but kept for backwards compatibility
		return nil, errors.Wrapf(sdkerrors.ErrInvalidRequest, "cannot update base or bonus proposer reward because these are deprecated fields")
	}

	if err := msg.Params.ValidateBasic(); err != nil {
		return nil, err
	}

	if err := k.Params.Set(ctx, msg.Params); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

// Deprecated: DO NOT USE
// This method uses deprecated message request. Use CommunityPoolSpend from x/protocolpool module instead.
func (k msgServer) CommunityPoolSpend(ctx context.Context, msg *types.MsgCommunityPoolSpend) (*types.MsgCommunityPoolSpendResponse, error) {
	if err := k.validateAuthority(msg.Authority); err != nil {
		return nil, err
	}

	if err := validateAmount(msg.Amount); err != nil {
		return nil, err
	}

	recipient, err := k.authKeeper.AddressCodec().StringToBytes(msg.Recipient)
	if err != nil {
		return nil, fmt.Errorf("invalid recipient address: %w", err)
	}

	if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ProtocolPoolModuleName, recipient, msg.Amount); err != nil {
		return nil, err
	}

	k.Logger.Info("transferred from the community pool to recipient", "amount", msg.Amount.String(), "recipient", msg.Recipient)

	return &types.MsgCommunityPoolSpendResponse{}, nil
}

func (k msgServer) DepositValidatorRewardsPool(ctx context.Context, msg *types.MsgDepositValidatorRewardsPool) (*types.MsgDepositValidatorRewardsPoolResponse, error) {
	depositor, err := k.authKeeper.AddressCodec().StringToBytes(msg.Depositor)
	if err != nil {
		return nil, fmt.Errorf("invalid depositor address: %w", err)
	}

	// deposit coins from depositor's account to the distribution module
	if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, depositor, types.ModuleName, msg.Amount); err != nil {
		return nil, err
	}

	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	validator, err := k.stakingKeeper.Validator(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	if validator == nil {
		return nil, errors.Wrap(types.ErrNoValidatorExists, msg.ValidatorAddress)
	}

	// Allocate tokens from the distribution module to the validator, which are
	// then distributed to the validator's delegators.
	reward := sdk.NewDecCoinsFromCoins(msg.Amount...)
	if err = k.AllocateTokensToValidator(ctx, validator, reward); err != nil {
		return nil, err
	}

	k.Logger.Info(
		"transferred from rewards to validator rewards pool",
		"depositor", msg.Depositor,
		"amount", msg.Amount.String(),
		"validator", msg.ValidatorAddress,
	)

	return &types.MsgDepositValidatorRewardsPoolResponse{}, nil
}

func (k *Keeper) validateAuthority(authority string) error {
	if _, err := k.authKeeper.AddressCodec().StringToBytes(authority); err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid authority address: %s", err)
	}

	if k.authority != authority {
		return errors.Wrapf(types.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, authority)
	}

	return nil
}

func validateAmount(amount sdk.Coins) error {
	if amount == nil {
		return errors.Wrap(sdkerrors.ErrInvalidCoins, "amount cannot be nil")
	}

	if err := amount.Validate(); err != nil {
		return errors.Wrap(sdkerrors.ErrInvalidCoins, amount.String())
	}

	return nil
}
