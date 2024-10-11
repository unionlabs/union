package keeper

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"slices"
	"strconv"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

type msgServer struct {
	*Keeper
}

// NewMsgServerImpl returns an implementation of the staking MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper *Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

// CreateValidator defines a method for creating a new validator.
// The validator's params should not be nil for this function to execute successfully.
func (k msgServer) CreateValidator(ctx context.Context, msg *types.MsgCreateValidator) (*types.MsgCreateValidatorResponse, error) {
	valAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	if err := msg.Validate(k.validatorAddressCodec); err != nil {
		return nil, err
	}

	minCommRate, err := k.MinCommissionRate(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Commission.Rate.LT(minCommRate) {
		return nil, errorsmod.Wrapf(types.ErrCommissionLTMinRate, "cannot set validator commission to less than minimum rate of %s", minCommRate)
	}

	// check to see if the pubkey or sender has been registered before
	if _, err := k.GetValidator(ctx, valAddr); err == nil {
		return nil, types.ErrValidatorOwnerExists
	}

	cv := msg.Pubkey.GetCachedValue()
	if cv == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidType, "Pubkey cached value is nil")
	}

	pk, ok := cv.(cryptotypes.PubKey)
	if !ok {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidType, "Expecting cryptotypes.PubKey, got %T", cv)
	}

	pubkeyTypes, err := k.consensusKeeper.ValidatorPubKeyTypes(ctx)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "failed to query consensus params: %s", err)
	}

	pkType := pk.Type()
	if !slices.Contains(pubkeyTypes, pkType) {
		return nil, errorsmod.Wrapf(
			types.ErrValidatorPubKeyTypeNotSupported,
			"got: %s, expected: %s", pk.Type(), pubkeyTypes,
		)
	}

	if pubkeyTypes == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "validator params are not set")
	}

	if err = validatePubKey(pk, pubkeyTypes); err != nil {
		return nil, err
	}

	err = k.checkConsKeyAlreadyUsed(ctx, pk)
	if err != nil {
		return nil, err
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Value.Denom != bondDenom {
		return nil, errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest, "invalid coin denomination: got %s, expected %s", msg.Value.Denom, bondDenom,
		)
	}

	if _, err := msg.Description.EnsureLength(); err != nil {
		return nil, err
	}

	validator, err := types.NewValidator(msg.ValidatorAddress, pk, msg.Description)
	if err != nil {
		return nil, err
	}

	commission := types.NewCommissionWithTime(
		msg.Commission.Rate, msg.Commission.MaxRate,
		msg.Commission.MaxChangeRate, k.HeaderService.HeaderInfo(ctx).Time,
	)

	validator, err = validator.SetInitialCommission(commission)
	if err != nil {
		return nil, err
	}

	validator.MinSelfDelegation = msg.MinSelfDelegation

	err = k.SetValidator(ctx, validator)
	if err != nil {
		return nil, err
	}

	err = k.SetValidatorByConsAddr(ctx, validator)
	if err != nil {
		return nil, err
	}

	err = k.SetNewValidatorByPowerIndex(ctx, validator)
	if err != nil {
		return nil, err
	}

	// call the after-creation hook
	if err := k.Hooks().AfterValidatorCreated(ctx, valAddr); err != nil {
		return nil, err
	}

	// move coins from the msg.Address account to a (self-delegation) delegator account
	// the validator account and global shares are updated within here
	// NOTE source will always be from a wallet which are unbonded
	_, err = k.Keeper.Delegate(ctx, sdk.AccAddress(valAddr), msg.Value.Amount, types.Unbonded, validator, true)
	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeCreateValidator,
		event.NewAttribute(types.AttributeKeyValidator, msg.ValidatorAddress),
		event.NewAttribute(sdk.AttributeKeyAmount, msg.Value.String()),
	); err != nil {
		return nil, err
	}

	return &types.MsgCreateValidatorResponse{}, nil
}

// EditValidator defines a method for editing an existing validator
func (k msgServer) EditValidator(ctx context.Context, msg *types.MsgEditValidator) (*types.MsgEditValidatorResponse, error) {
	valAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	if msg.Description == (types.Description{}) {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "empty description")
	}

	if msg.MinSelfDelegation != nil && !msg.MinSelfDelegation.IsPositive() {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"minimum self delegation must be a positive integer",
		)
	}

	if msg.CommissionRate != nil {
		if msg.CommissionRate.GT(math.LegacyOneDec()) || msg.CommissionRate.IsNegative() {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "commission rate must be between 0 and 1 (inclusive)")
		}

		minCommissionRate, err := k.MinCommissionRate(ctx)
		if err != nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrLogic, err.Error())
		}

		if msg.CommissionRate.LT(minCommissionRate) {
			return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "commission rate cannot be less than the min commission rate %s", minCommissionRate.String())
		}
	}

	// validator must already be registered
	validator, err := k.GetValidator(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	// replace all editable fields (clients should autofill existing values)
	description, err := validator.Description.UpdateDescription(msg.Description)
	if err != nil {
		return nil, err
	}

	validator.Description = description

	if msg.CommissionRate != nil {
		commission, err := k.UpdateValidatorCommission(ctx, validator, *msg.CommissionRate)
		if err != nil {
			return nil, err
		}

		// call the before-modification hook since we're about to update the commission
		if err := k.Hooks().BeforeValidatorModified(ctx, valAddr); err != nil {
			return nil, err
		}

		validator.Commission = commission
	}

	if msg.MinSelfDelegation != nil {
		if !msg.MinSelfDelegation.GT(validator.MinSelfDelegation) {
			return nil, types.ErrMinSelfDelegationDecreased
		}

		if msg.MinSelfDelegation.GT(validator.Tokens) {
			return nil, types.ErrSelfDelegationBelowMinimum
		}

		validator.MinSelfDelegation = *msg.MinSelfDelegation
	}

	err = k.SetValidator(ctx, validator)
	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeEditValidator,
		event.NewAttribute(types.AttributeKeyCommissionRate, validator.Commission.String()),
		event.NewAttribute(types.AttributeKeyMinSelfDelegation, validator.MinSelfDelegation.String()),
	); err != nil {
		return nil, err
	}

	return &types.MsgEditValidatorResponse{}, nil
}

// Delegate defines a method for performing a delegation of coins from a delegator to a validator
func (k msgServer) Delegate(ctx context.Context, msg *types.MsgDelegate) (*types.MsgDelegateResponse, error) {
	valAddr, valErr := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if valErr != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", valErr)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	if !msg.Amount.IsValid() || !msg.Amount.Amount.IsPositive() {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"invalid delegation amount",
		)
	}

	validator, err := k.GetValidator(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Amount.Denom != bondDenom {
		return nil, errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest, "invalid coin denomination: got %s, expected %s", msg.Amount.Denom, bondDenom,
		)
	}

	// NOTE: source funds are always unbonded
	newShares, err := k.Keeper.Delegate(ctx, delegatorAddress, msg.Amount.Amount, types.Unbonded, validator, true)
	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeDelegate,
		event.NewAttribute(types.AttributeKeyValidator, msg.ValidatorAddress),
		event.NewAttribute(types.AttributeKeyDelegator, msg.DelegatorAddress),
		event.NewAttribute(sdk.AttributeKeyAmount, msg.Amount.String()),
		event.NewAttribute(types.AttributeKeyNewShares, newShares.String()),
	); err != nil {
		return nil, err
	}

	return &types.MsgDelegateResponse{}, nil
}

// BeginRedelegate defines a method for performing a redelegation of coins from a source validator to a destination validator of given delegator
func (k msgServer) BeginRedelegate(ctx context.Context, msg *types.MsgBeginRedelegate) (*types.MsgBeginRedelegateResponse, error) {
	valSrcAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorSrcAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid source validator address: %s", err)
	}

	valDstAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorDstAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid destination validator address: %s", err)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	if !msg.Amount.IsValid() || !msg.Amount.Amount.IsPositive() {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"invalid shares amount",
		)
	}

	shares, err := k.ValidateUnbondAmount(
		ctx, delegatorAddress, valSrcAddr, msg.Amount.Amount,
	)
	if err != nil {
		return nil, err
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Amount.Denom != bondDenom {
		return nil, errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest, "invalid coin denomination: got %s, expected %s", msg.Amount.Denom, bondDenom,
		)
	}

	completionTime, err := k.BeginRedelegation(
		ctx, delegatorAddress, valSrcAddr, valDstAddr, shares,
	)
	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeRedelegate,
		event.NewAttribute(types.AttributeKeySrcValidator, msg.ValidatorSrcAddress),
		event.NewAttribute(types.AttributeKeyDstValidator, msg.ValidatorDstAddress),
		event.NewAttribute(sdk.AttributeKeyAmount, msg.Amount.String()),
		event.NewAttribute(types.AttributeKeyCompletionTime, completionTime.Format(time.RFC3339)),
	); err != nil {
		return nil, err
	}

	return &types.MsgBeginRedelegateResponse{
		CompletionTime: completionTime,
	}, nil
}

// Undelegate defines a method for performing an undelegation from a delegate and a validator
func (k msgServer) Undelegate(ctx context.Context, msg *types.MsgUndelegate) (*types.MsgUndelegateResponse, error) {
	addr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	if !msg.Amount.IsValid() || !msg.Amount.Amount.IsPositive() {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"invalid shares amount",
		)
	}

	shares, err := k.ValidateUnbondAmount(
		ctx, delegatorAddress, addr, msg.Amount.Amount,
	)
	if err != nil {
		return nil, err
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Amount.Denom != bondDenom {
		return nil, errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest, "invalid coin denomination: got %s, expected %s", msg.Amount.Denom, bondDenom,
		)
	}

	completionTime, undelegatedAmt, err := k.Keeper.Undelegate(ctx, delegatorAddress, addr, shares)
	if err != nil {
		return nil, err
	}

	undelegatedCoin := sdk.NewCoin(msg.Amount.Denom, undelegatedAmt)

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeUnbond,
		event.NewAttribute(types.AttributeKeyValidator, msg.ValidatorAddress),
		event.NewAttribute(types.AttributeKeyDelegator, msg.DelegatorAddress),
		event.NewAttribute(sdk.AttributeKeyAmount, undelegatedCoin.String()),
		event.NewAttribute(types.AttributeKeyCompletionTime, completionTime.Format(time.RFC3339)),
	); err != nil {
		return nil, err
	}

	return &types.MsgUndelegateResponse{
		CompletionTime: completionTime,
		Amount:         undelegatedCoin,
	}, nil
}

// CancelUnbondingDelegation defines a method for canceling the unbonding delegation
// and delegate back to the validator.
func (k msgServer) CancelUnbondingDelegation(ctx context.Context, msg *types.MsgCancelUnbondingDelegation) (*types.MsgCancelUnbondingDelegationResponse, error) {
	valAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid validator address: %s", err)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(msg.DelegatorAddress)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid delegator address: %s", err)
	}

	if !msg.Amount.IsValid() || !msg.Amount.Amount.IsPositive() {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"invalid amount",
		)
	}

	if msg.CreationHeight <= 0 {
		return nil, errorsmod.Wrap(
			sdkerrors.ErrInvalidRequest,
			"invalid height",
		)
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}

	if msg.Amount.Denom != bondDenom {
		return nil, errorsmod.Wrapf(
			sdkerrors.ErrInvalidRequest, "invalid coin denomination: got %s, expected %s", msg.Amount.Denom, bondDenom,
		)
	}

	validator, err := k.GetValidator(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	// In some situations, the exchange rate becomes invalid, e.g. if
	// Validator loses all tokens due to slashing. In this case,
	// make all future delegations invalid.
	if validator.InvalidExRate() {
		return nil, types.ErrDelegatorShareExRateInvalid
	}

	if validator.IsJailed() {
		return nil, types.ErrValidatorJailed
	}

	ubd, err := k.GetUnbondingDelegation(ctx, delegatorAddress, valAddr)
	if err != nil {
		return nil, status.Errorf(
			codes.NotFound,
			"unbonding delegation with delegator %s not found for validator %s",
			msg.DelegatorAddress, msg.ValidatorAddress,
		)
	}

	var (
		unbondEntry      types.UnbondingDelegationEntry
		unbondEntryIndex int64 = -1
	)

	for i, entry := range ubd.Entries {
		if entry.CreationHeight == msg.CreationHeight {
			unbondEntry = entry
			unbondEntryIndex = int64(i)
			break
		}
	}
	if unbondEntryIndex == -1 {
		return nil, sdkerrors.ErrNotFound.Wrapf("unbonding delegation entry is not found at block height %d", msg.CreationHeight)
	}

	if unbondEntry.Balance.LT(msg.Amount.Amount) {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("amount is greater than the unbonding delegation entry balance")
	}

	headerInfo := k.HeaderService.HeaderInfo(ctx)
	if unbondEntry.CompletionTime.Before(headerInfo.Time) {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("unbonding delegation is already processed")
	}

	// delegate back the unbonding delegation amount to the validator
	_, err = k.Keeper.Delegate(ctx, delegatorAddress, msg.Amount.Amount, types.Unbonding, validator, false)
	if err != nil {
		return nil, err
	}

	amount := unbondEntry.Balance.Sub(msg.Amount.Amount)
	if amount.IsZero() {
		ubd.RemoveEntry(unbondEntryIndex)
	} else {
		// update the unbondingDelegationEntryBalance and InitialBalance for ubd entry
		unbondEntry.Balance = amount
		unbondEntry.InitialBalance = unbondEntry.InitialBalance.Sub(msg.Amount.Amount)
		ubd.Entries[unbondEntryIndex] = unbondEntry
	}

	// set the unbonding delegation or remove it if there are no more entries
	if len(ubd.Entries) == 0 {
		err = k.RemoveUnbondingDelegation(ctx, ubd)
	} else {
		err = k.SetUnbondingDelegation(ctx, ubd)
	}

	if err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeCancelUnbondingDelegation,
		event.NewAttribute(sdk.AttributeKeyAmount, msg.Amount.String()),
		event.NewAttribute(types.AttributeKeyValidator, msg.ValidatorAddress),
		event.NewAttribute(types.AttributeKeyDelegator, msg.DelegatorAddress),
		event.NewAttribute(types.AttributeKeyCreationHeight, strconv.FormatInt(msg.CreationHeight, 10)),
	); err != nil {
		return nil, err
	}

	return &types.MsgCancelUnbondingDelegationResponse{}, nil
}

// UpdateParams defines a method to perform updation of params exist in x/staking module.
func (k msgServer) UpdateParams(ctx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if k.authority != msg.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	if err := msg.Params.Validate(); err != nil {
		return nil, err
	}

	// get previous staking params
	previousParams, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	// store params
	if err := k.Params.Set(ctx, msg.Params); err != nil {
		return nil, err
	}

	// when min commission rate is updated, we need to update the commission rate of all validators
	if !previousParams.MinCommissionRate.Equal(msg.Params.MinCommissionRate) {
		minRate := msg.Params.MinCommissionRate

		vals, err := k.GetAllValidators(ctx)
		if err != nil {
			return nil, err
		}

		for _, val := range vals {
			// set the commission rate to min rate
			if val.Commission.CommissionRates.Rate.LT(minRate) {
				val.Commission.CommissionRates.Rate = minRate
				// set the max rate to minRate if it is less than min rate
				if val.Commission.CommissionRates.MaxRate.LT(minRate) {
					val.Commission.CommissionRates.MaxRate = minRate
				}

				val.Commission.UpdateTime = k.HeaderService.HeaderInfo(ctx).Time
				if err := k.SetValidator(ctx, val); err != nil {
					return nil, fmt.Errorf("failed to set validator after MinCommissionRate param change: %w", err)
				}
			}
		}
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

// RotateConsPubKey handles the rotation of a validator's consensus public key.
// It validates the new key, checks for conflicts, and updates the necessary state.
// The function requires that the validator params are not nil for successful execution.
func (k msgServer) RotateConsPubKey(ctx context.Context, msg *types.MsgRotateConsPubKey) (res *types.MsgRotateConsPubKeyResponse, err error) {
	cv := msg.NewPubkey.GetCachedValue()
	if cv == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidType, "new public key is nil")
	}

	pk, ok := cv.(cryptotypes.PubKey)
	if !ok {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidType, "expecting cryptotypes.PubKey, got %T", cv)
	}

	pubkeyTypes, err := k.consensusKeeper.ValidatorPubKeyTypes(ctx)
	if err != nil {
		return nil, err
	}

	if pubkeyTypes == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "validator params are not set")
	}

	if err = validatePubKey(pk, pubkeyTypes); err != nil {
		return nil, err
	}

	err = k.checkConsKeyAlreadyUsed(ctx, pk)
	if err != nil {
		return nil, err
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(msg.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	validator, err := k.Keeper.GetValidator(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	if validator.GetOperator() == "" {
		return nil, types.ErrNoValidatorFound
	}

	if status := validator.GetStatus(); status != sdk.Bonded {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidType, "validator status is not bonded, got %x", status)
	}

	// Check if the validator is exceeding parameter MaxConsPubKeyRotations within the
	// unbonding period by iterating ConsPubKeyRotationHistory.
	err = k.ExceedsMaxRotations(ctx, valAddr)
	if err != nil {
		return nil, err
	}

	// Check if the signing account has enough balance to pay KeyRotationFee
	// KeyRotationFees are sent to the community fund.
	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	err = k.Keeper.bankKeeper.SendCoinsFromAccountToModule(ctx, sdk.AccAddress(valAddr), types.PoolModuleName, sdk.NewCoins(params.KeyRotationFee))
	if err != nil {
		return nil, err
	}

	// Add ConsPubKeyRotationHistory for tracking rotation
	err = k.setConsPubKeyRotationHistory(
		ctx,
		valAddr,
		validator.ConsensusPubkey,
		msg.NewPubkey,
		params.KeyRotationFee,
	)
	if err != nil {
		return nil, err
	}

	return res, nil
}

// checkConsKeyAlreadyUsed returns an error if the consensus public key is already used,
// in ConsAddrToValidatorIdentifierMap, OldToNewConsAddrMap, or in the current block (RotationHistory).
func (k msgServer) checkConsKeyAlreadyUsed(ctx context.Context, newConsPubKey cryptotypes.PubKey) error {
	newConsAddr := sdk.ConsAddress(newConsPubKey.Address())
	rotatedTo, err := k.ConsAddrToValidatorIdentifierMap.Get(ctx, newConsAddr)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return err
	}

	if rotatedTo != nil {
		return sdkerrors.ErrInvalidAddress.Wrap(
			"public key was already used")
	}

	// check in the current block
	rotationHistory, err := k.GetBlockConsPubKeyRotationHistory(ctx)
	if err != nil {
		return err
	}

	for _, rotation := range rotationHistory {
		cachedValue := rotation.NewConsPubkey.GetCachedValue()
		if cachedValue == nil {
			return sdkerrors.ErrInvalidAddress.Wrap("new public key is nil")
		}
		if bytes.Equal(cachedValue.(cryptotypes.PubKey).Address(), newConsAddr) {
			return sdkerrors.ErrInvalidAddress.Wrap("public key was already used")
		}
	}

	// checks if NewPubKey is not duplicated on ValidatorsByConsAddr
	_, err = k.Keeper.ValidatorByConsAddr(ctx, newConsAddr)
	if err == nil {
		return types.ErrValidatorPubKeyExists
	}

	return nil
}

func validatePubKey(pk cryptotypes.PubKey, knownPubKeyTypes []string) error {
	pkType := pk.Type()
	if !slices.Contains(knownPubKeyTypes, pkType) {
		return errorsmod.Wrapf(
			types.ErrValidatorPubKeyTypeNotSupported,
			"got: %s, expected: %s", pk.Type(), knownPubKeyTypes,
		)
	}

	if pkType == sdk.PubKeyEd25519Type {
		if len(pk.Bytes()) != ed25519.PubKeySize {
			return errorsmod.Wrapf(
				types.ErrConsensusPubKeyLenInvalid,
				"invalid Ed25519 pubkey size: got %d, expected %d", len(pk.Bytes()), ed25519.PubKeySize,
			)
		}
	}

	return nil
}
