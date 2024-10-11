package authz

import (
	"time"

	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	errorsmod "cosmossdk.io/errors"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// NewGrant returns new Grant. Expiration is optional and noop if null.
// It returns an error if the expiration is before the current block time,
// which is passed into the `blockTime` arg.
func NewGrant(blockTime time.Time, a Authorization, expiration *time.Time) (Grant, error) {
	if expiration != nil && !expiration.After(blockTime) {
		return Grant{}, errorsmod.Wrapf(ErrInvalidExpirationTime, "expiration must be after the current block time (%v), got %v", blockTime.Format(time.RFC3339), expiration.Format(time.RFC3339))
	}
	msg, ok := a.(proto.Message)
	if !ok {
		return Grant{}, sdkerrors.ErrPackAny.Wrapf("cannot proto marshal %T", a)
	}
	any, err := gogoprotoany.NewAnyWithCacheWithValue(msg)
	if err != nil {
		return Grant{}, err
	}
	return Grant{
		Expiration:    expiration,
		Authorization: any,
	}, nil
}

var _ gogoprotoany.UnpackInterfacesMessage = &Grant{}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (g Grant) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var authorization Authorization
	return unpacker.UnpackAny(g.Authorization, &authorization)
}

// GetAuthorization returns the cached value from the Grant.Authorization if present.
func (g Grant) GetAuthorization() (Authorization, error) {
	if g.Authorization == nil {
		return nil, sdkerrors.ErrInvalidType.Wrap("authorization is nil")
	}
	av := g.Authorization.GetCachedValue()
	a, ok := av.(Authorization)
	if !ok {
		return nil, sdkerrors.ErrInvalidType.Wrapf("expected %T, got %T", (Authorization)(nil), av)
	}
	return a, nil
}

func (g Grant) ValidateBasic() error {
	if g.Authorization == nil {
		return sdkerrors.ErrInvalidType.Wrap("authorization is nil")
	}

	av := g.Authorization.GetCachedValue()
	a, ok := av.(Authorization)
	if !ok {
		return sdkerrors.ErrInvalidType.Wrapf("expected %T, got %T", (Authorization)(nil), av)
	}
	return a.ValidateBasic()
}
