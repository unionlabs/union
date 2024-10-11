package runtime

import (
	"context"

	"cosmossdk.io/core/transaction"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ transaction.Service = TransactionService{}

type TransactionService struct{}

// ExecMode implements transaction.Service.
func (t TransactionService) ExecMode(ctx context.Context) transaction.ExecMode {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	return transaction.ExecMode(sdkCtx.ExecMode())
}
