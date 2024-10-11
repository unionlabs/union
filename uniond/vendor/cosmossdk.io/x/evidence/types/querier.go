package types

import (
	"github.com/cosmos/cosmos-sdk/types/query"
)

// Querier routes for the evidence module
const (
	QueryEvidence    = "evidence"
	QueryAllEvidence = "all_evidence"
)

// NewQueryEvidenceRequest creates a new instance of QueryEvidenceRequest.
func NewQueryEvidenceRequest(hash string) *QueryEvidenceRequest {
	return &QueryEvidenceRequest{Hash: hash}
}

// NewQueryAllEvidenceRequest creates a new instance of QueryAllEvidenceRequest.
func NewQueryAllEvidenceRequest(pageReq *query.PageRequest) *QueryAllEvidenceRequest {
	return &QueryAllEvidenceRequest{Pagination: pageReq}
}

// QueryAllEvidenceParams defines the parameters necessary for querying for all Evidence.
type QueryAllEvidenceParams struct {
	Page  int `json:"page" yaml:"page"`
	Limit int `json:"limit" yaml:"limit"`
}

// NewQueryAllEvidenceParams creates a new instance to query all evidence params.
func NewQueryAllEvidenceParams(page, limit int) QueryAllEvidenceParams {
	return QueryAllEvidenceParams{Page: page, Limit: limit}
}
