package types

import (
	"context"
	"errors"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	corecontext "cosmossdk.io/core/context"
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/authz"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// TODO: Revisit this once we have proper gas fee framework.
// Tracking issues https://github.com/cosmos/cosmos-sdk/issues/9054, https://github.com/cosmos/cosmos-sdk/discussions/9072
const gasCostPerIteration = uint64(10)

// NewStakeAuthorization creates a new StakeAuthorization object.
func NewStakeAuthorization(allowed, denied []sdk.ValAddress, authzType AuthorizationType, amount *sdk.Coin, valAddressCodec address.Codec) (*StakeAuthorization, error) {
	allowedValidators, deniedValidators, err := validateAllowAndDenyValidators(allowed, denied, valAddressCodec)
	if err != nil {
		return nil, err
	}

	a := StakeAuthorization{}
	if allowedValidators != nil {
		a.Validators = &StakeAuthorization_AllowList{
			AllowList: &StakeAuthorization_Validators{
				Address: allowedValidators,
			},
		}
	} else {
		a.Validators = &StakeAuthorization_DenyList{
			DenyList: &StakeAuthorization_Validators{
				Address: deniedValidators,
			},
		}
	}

	if amount != nil {
		a.MaxTokens = amount
	}

	a.AuthorizationType = authzType

	return &a, nil
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a StakeAuthorization) MsgTypeURL() string {
	authzType, err := normalizeAuthzType(a.AuthorizationType)
	if err != nil {
		panic(err)
	}

	return authzType
}

// ValidateBasic performs a stateless validation of the fields.
// It fails if MaxTokens is either undefined or negative or if the authorization
// is unspecified.
func (a StakeAuthorization) ValidateBasic() error {
	if a.MaxTokens != nil && a.MaxTokens.IsNegative() {
		return errorsmod.Wrapf(errors.New("max tokens should be positive"),
			"negative coin amount: %v", a.MaxTokens)
	}

	if a.AuthorizationType == AuthorizationType_AUTHORIZATION_TYPE_UNSPECIFIED {
		return errors.New("unknown authorization type")
	}

	return nil
}

// Accept implements Authorization.Accept. It checks, that the validator is not in the denied list,
// and, should the allowed list not be empty, if the validator is in the allowed list.
// If these conditions are met, the authorization amount is validated and if successful, the
// corresponding AcceptResponse is returned.
func (a StakeAuthorization) Accept(ctx context.Context, msg sdk.Msg) (authz.AcceptResponse, error) {
	var (
		validatorAddress string
		amount           sdk.Coin
	)

	switch msg := msg.(type) {
	case *MsgDelegate:
		validatorAddress = msg.ValidatorAddress
		amount = msg.Amount
	case *MsgUndelegate:
		validatorAddress = msg.ValidatorAddress
		amount = msg.Amount
	case *MsgBeginRedelegate:
		validatorAddress = msg.ValidatorDstAddress
		amount = msg.Amount
	case *MsgCancelUnbondingDelegation:
		validatorAddress = msg.ValidatorAddress
		amount = msg.Amount
	default:
		return authz.AcceptResponse{}, sdkerrors.ErrInvalidRequest.Wrap("unknown msg type")
	}

	authzEnv, ok := ctx.Value(corecontext.EnvironmentContextKey).(appmodule.Environment)
	if !ok {
		return authz.AcceptResponse{}, sdkerrors.ErrUnauthorized.Wrap("environment not set")
	}
	isValidatorExists := false
	allowedList := a.GetAllowList().GetAddress()
	for _, validator := range allowedList {
		if err := authzEnv.GasService.GasMeter(ctx).Consume(gasCostPerIteration, "stake authorization"); err != nil {
			return authz.AcceptResponse{}, err
		}

		if validator == validatorAddress {
			isValidatorExists = true
			break
		}
	}

	denyList := a.GetDenyList().GetAddress()
	for _, validator := range denyList {
		if err := authzEnv.GasService.GasMeter(ctx).Consume(gasCostPerIteration, "stake authorization"); err != nil {
			return authz.AcceptResponse{}, err
		}

		if validator == validatorAddress {
			return authz.AcceptResponse{}, sdkerrors.ErrUnauthorized.Wrapf("cannot delegate/undelegate to %s validator", validator)
		}
	}

	if len(allowedList) > 0 && !isValidatorExists {
		return authz.AcceptResponse{}, sdkerrors.ErrUnauthorized.Wrapf("cannot delegate/undelegate to %s validator", validatorAddress)
	}

	if a.MaxTokens == nil {
		return authz.AcceptResponse{
			Accept: true,
			Delete: false,
			Updated: &StakeAuthorization{
				Validators:        a.GetValidators(),
				AuthorizationType: a.GetAuthorizationType(),
			},
		}, nil
	}

	limitLeft, err := a.MaxTokens.SafeSub(amount)
	if err != nil {
		return authz.AcceptResponse{}, err
	}

	if limitLeft.IsZero() {
		return authz.AcceptResponse{Accept: true, Delete: true}, nil
	}

	return authz.AcceptResponse{
		Accept: true,
		Delete: false,
		Updated: &StakeAuthorization{
			Validators:        a.GetValidators(),
			AuthorizationType: a.GetAuthorizationType(),
			MaxTokens:         &limitLeft,
		},
	}, nil
}

func validateAllowAndDenyValidators(allowed, denied []sdk.ValAddress, valAddressCodec address.Codec) ([]string, []string, error) {
	if len(allowed) == 0 && len(denied) == 0 {
		return nil, nil, sdkerrors.ErrInvalidRequest.Wrap("both allowed & deny list cannot be empty")
	}

	if len(allowed) > 0 && len(denied) > 0 {
		return nil, nil, sdkerrors.ErrInvalidRequest.Wrap("cannot set both allowed & deny list")
	}

	allowedValidators := make([]string, len(allowed))
	if len(allowed) > 0 {
		foundAllowedValidators := make(map[string]bool, len(allowed))
		for i, validator := range allowed {
			valAddr, err := valAddressCodec.BytesToString(validator)
			if err != nil {
				return nil, nil, sdkerrors.ErrInvalidRequest.Wrap("could not convert validator address")
			}
			if foundAllowedValidators[valAddr] {
				return nil, nil, sdkerrors.ErrInvalidRequest.Wrapf("duplicate allowed validator address: %s", valAddr)
			}
			foundAllowedValidators[valAddr] = true
			allowedValidators[i] = valAddr
		}
		return allowedValidators, nil, nil
	}

	deniedValidators := make([]string, len(denied))
	foundDeniedValidators := make(map[string]bool, len(denied))
	for i, validator := range denied {
		valAddr, err := valAddressCodec.BytesToString(validator)
		if err != nil {
			return nil, nil, sdkerrors.ErrInvalidRequest.Wrap("could not convert validator address")
		}
		if foundDeniedValidators[valAddr] {
			return nil, nil, sdkerrors.ErrInvalidRequest.Wrapf("duplicate denied validator address: %s", valAddr)
		}
		foundDeniedValidators[valAddr] = true
		deniedValidators[i] = valAddr
	}

	return nil, deniedValidators, nil
}

// Normalized Msg type URLs
func normalizeAuthzType(authzType AuthorizationType) (string, error) {
	switch authzType {
	case AuthorizationType_AUTHORIZATION_TYPE_DELEGATE:
		return sdk.MsgTypeURL(&MsgDelegate{}), nil
	case AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE:
		return sdk.MsgTypeURL(&MsgUndelegate{}), nil
	case AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE:
		return sdk.MsgTypeURL(&MsgBeginRedelegate{}), nil
	case AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION:
		return sdk.MsgTypeURL(&MsgCancelUnbondingDelegation{}), nil
	default:
		return "", errorsmod.Wrapf(errors.New("unknown authorization type"),
			"cannot normalize authz type with %T", authzType)
	}
}
