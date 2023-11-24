package authz

import (
	"context"
	"errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ Authorization = &GenericAuthorization{}

// NewGenericAuthorization creates a new GenericAuthorization object.
func NewGenericAuthorization(msgTypeURL string) *GenericAuthorization {
	return &GenericAuthorization{
		Msg: msgTypeURL,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a GenericAuthorization) MsgTypeURL() string {
	return a.Msg
}

// Accept implements Authorization.Accept.
func (a GenericAuthorization) Accept(ctx context.Context, msg sdk.Msg) (AcceptResponse, error) {
	return AcceptResponse{Accept: true}, nil
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a GenericAuthorization) ValidateBasic() error {
	if a.Msg == "" {
		return errors.New("msg type cannot be empty")
	}
	return nil
}
