package authz

import (
	"context"

	"github.com/cosmos/gogoproto/proto"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/authz"
)

// Authorization represents the interface of various Authorization types implemented
// by other modules.
type Authorization interface {
	proto.Message

	// MsgTypeURL returns the fully-qualified Msg service method URL (as described in ADR 031),
	// which will process and accept or reject a request.
	MsgTypeURL() string

	// Accept determines whether this grant permits the provided sdk.Msg to be performed,
	// and if so provides an upgraded authorization instance.
	Accept(ctx context.Context, msg sdk.Msg) (authz.AcceptResponse, error)

	// ValidateBasic does a simple validation check that
	// doesn't require access to any other information.
	ValidateBasic() error
}
