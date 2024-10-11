package feegrant

import (
	"context"
	"errors"
	"time"

	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/core/appmodule"
	corecontext "cosmossdk.io/core/context"
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// TODO: Revisit this once we have proper gas fee framework.
// Tracking issues https://github.com/cosmos/cosmos-sdk/issues/9054, https://github.com/cosmos/cosmos-sdk/discussions/9072
const (
	gasCostPerIteration = uint64(10)
)

var (
	_ FeeAllowanceI                        = (*AllowedMsgAllowance)(nil)
	_ gogoprotoany.UnpackInterfacesMessage = (*AllowedMsgAllowance)(nil)
)

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (a *AllowedMsgAllowance) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var allowance FeeAllowanceI
	return unpacker.UnpackAny(a.Allowance, &allowance)
}

// NewAllowedMsgAllowance creates new filtered fee allowance.
func NewAllowedMsgAllowance(allowance FeeAllowanceI, allowedMsgs []string) (*AllowedMsgAllowance, error) {
	msg, ok := allowance.(proto.Message)
	if !ok {
		return nil, errorsmod.Wrapf(sdkerrors.ErrPackAny, "cannot proto marshal %T", msg)
	}
	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, err
	}

	return &AllowedMsgAllowance{
		Allowance:       any,
		AllowedMessages: allowedMsgs,
	}, nil
}

// GetAllowance returns allowed fee allowance.
func (a *AllowedMsgAllowance) GetAllowance() (FeeAllowanceI, error) {
	allowance, ok := a.Allowance.GetCachedValue().(FeeAllowanceI)
	if !ok {
		return nil, errorsmod.Wrap(ErrNoAllowance, "failed to get allowance")
	}

	return allowance, nil
}

// SetAllowance sets allowed fee allowance.
func (a *AllowedMsgAllowance) SetAllowance(allowance FeeAllowanceI) error {
	newAllowance, err := types.NewAnyWithValue(allowance.(proto.Message))
	if err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrPackAny, "cannot proto marshal %T", allowance)
	}

	a.Allowance = newAllowance

	return nil
}

// Accept method checks for the filtered messages has valid expiry
func (a *AllowedMsgAllowance) Accept(ctx context.Context, fee sdk.Coins, msgs []sdk.Msg) (bool, error) {
	allowed, err := a.allMsgTypesAllowed(ctx, msgs)
	if err != nil {
		return false, err
	}
	if !allowed {
		return false, errorsmod.Wrap(ErrMessageNotAllowed, "message does not exist in allowed messages")
	}

	allowance, err := a.GetAllowance()
	if err != nil {
		return false, err
	}

	remove, err := allowance.Accept(ctx, fee, msgs)
	if err == nil && !remove {
		if err = a.SetAllowance(allowance); err != nil {
			return false, err
		}
	}
	return remove, err
}

func (a *AllowedMsgAllowance) allowedMsgsToMap(ctx context.Context) (map[string]struct{}, error) {
	msgsMap := make(map[string]struct{}, len(a.AllowedMessages))
	environment, ok := ctx.Value(corecontext.EnvironmentContextKey).(appmodule.Environment)
	if !ok {
		return nil, errors.New("environment not set")
	}
	gasMeter := environment.GasService.GasMeter(ctx)
	for _, msg := range a.AllowedMessages {
		if err := gasMeter.Consume(gasCostPerIteration, "check msg"); err != nil {
			return nil, err
		}
		msgsMap[msg] = struct{}{}
	}

	return msgsMap, nil
}

func (a *AllowedMsgAllowance) allMsgTypesAllowed(ctx context.Context, msgs []sdk.Msg) (bool, error) {
	msgsMap, err := a.allowedMsgsToMap(ctx)
	if err != nil {
		return false, err
	}
	environment, ok := ctx.Value(corecontext.EnvironmentContextKey).(appmodule.Environment)
	if !ok {
		return false, errors.New("environment not set")
	}
	gasMeter := environment.GasService.GasMeter(ctx)
	for _, msg := range msgs {
		if err := gasMeter.Consume(gasCostPerIteration, "check msg"); err != nil {
			return false, err
		}
		if _, allowed := msgsMap[sdk.MsgTypeURL(msg)]; !allowed {
			return false, nil
		}
	}

	return true, nil
}

// ValidateBasic implements FeeAllowance and enforces basic sanity checks
func (a *AllowedMsgAllowance) ValidateBasic() error {
	if a.Allowance == nil {
		return errorsmod.Wrap(ErrNoAllowance, "allowance should not be empty")
	}
	if len(a.AllowedMessages) == 0 {
		return errorsmod.Wrap(ErrNoMessages, "allowed messages shouldn't be empty")
	}

	allowance, err := a.GetAllowance()
	if err != nil {
		return err
	}

	return allowance.ValidateBasic()
}

// ExpiresAt returns the expiry time of the AllowedMsgAllowance.
func (a *AllowedMsgAllowance) ExpiresAt() (*time.Time, error) {
	allowance, err := a.GetAllowance()
	if err != nil {
		return nil, err
	}
	return allowance.ExpiresAt()
}

// UpdatePeriodReset update "PeriodReset" of the AllowedMsgAllowance.
func (a *AllowedMsgAllowance) UpdatePeriodReset(validTime time.Time) error {
	allowance, err := a.GetAllowance()
	if err != nil {
		return err
	}
	return allowance.UpdatePeriodReset(validTime)
}
