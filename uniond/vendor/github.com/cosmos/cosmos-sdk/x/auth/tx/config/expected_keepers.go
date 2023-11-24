package tx

import (
	"context"

	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// BankKeeper defines the contract needed for tx-related APIs
type BankKeeper interface {
	DenomMetadata(c context.Context, req *types.QueryDenomMetadataRequest) (*types.QueryDenomMetadataResponse, error)
}
