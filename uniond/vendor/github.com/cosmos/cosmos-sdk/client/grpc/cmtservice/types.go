package cmtservice

import (
	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
)

// ToABCIRequestQuery converts a gRPC ABCIQueryRequest type to an ABCI
// RequestQuery type.
func (req *ABCIQueryRequest) ToABCIRequestQuery() *abci.QueryRequest {
	return &abci.QueryRequest{
		Data:   req.Data,
		Path:   req.Path,
		Height: req.Height,
		Prove:  req.Prove,
	}
}
