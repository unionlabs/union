package ante

import (
	"context"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// AccountKeeper defines the contract needed for AccountKeeper related APIs.
// Interface provides support to use non-sdk AccountKeeper for AnteHandler's decorators.
type AccountKeeper interface {
	GetParams(ctx context.Context) (params types.Params)
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	SetAccount(ctx context.Context, acc sdk.AccountI)
	GetModuleAddress(moduleName string) sdk.AccAddress
	NewAccountWithAddress(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	AddressCodec() address.Codec
	GetEnvironment() appmodule.Environment
}

// FeegrantKeeper defines the expected feegrant keeper.
type FeegrantKeeper interface {
	UseGrantedFees(ctx context.Context, granter, grantee sdk.AccAddress, fee sdk.Coins, msgs []sdk.Msg) error
}

type ConsensusKeeper interface {
	BlockParams(context.Context) (uint64, uint64, error)
}
