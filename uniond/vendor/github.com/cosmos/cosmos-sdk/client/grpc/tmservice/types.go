package tmservice

import (
	abci "github.com/cometbft/cometbft/abci/types"
)

// ToABCIRequestQuery converts a gRPC ABCIQueryRequest type to an ABCI
// RequestQuery type.
func (req *ABCIQueryRequest) ToABCIRequestQuery() abci.RequestQuery {
	return abci.RequestQuery{
		Data:   req.Data,
		Path:   req.Path,
		Height: req.Height,
		Prove:  req.Prove,
	}
}

// FromABCIResponseQuery converts an ABCI ResponseQuery type to a gRPC
// ABCIQueryResponse type.
func FromABCIResponseQuery(res abci.ResponseQuery) *ABCIQueryResponse {
	var proofOps *ProofOps

	if res.ProofOps != nil {
		proofOps = &ProofOps{
			Ops: make([]ProofOp, len(res.ProofOps.Ops)),
		}
		for i, proofOp := range res.ProofOps.Ops {
			proofOps.Ops[i] = ProofOp{
				Type: proofOp.Type,
				Key:  proofOp.Key,
				Data: proofOp.Data,
			}
		}
	}

	return &ABCIQueryResponse{
		Code:      res.Code,
		Log:       res.Log,
		Info:      res.Info,
		Index:     res.Index,
		Key:       res.Key,
		Value:     res.Value,
		ProofOps:  proofOps,
		Height:    res.Height,
		Codespace: res.Codespace,
	}
}
