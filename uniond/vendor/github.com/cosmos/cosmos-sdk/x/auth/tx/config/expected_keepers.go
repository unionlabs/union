package tx

import (
	"context"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
)

// BankKeeper defines the contract needed for tx-related APIs
type BankKeeper interface {
	DenomMetadataV2(c context.Context, req *bankv1beta1.QueryDenomMetadataRequest) (*bankv1beta1.QueryDenomMetadataResponse, error)
}
