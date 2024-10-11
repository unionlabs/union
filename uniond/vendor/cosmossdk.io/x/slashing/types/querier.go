package types

// Query endpoints supported by the slashing querier
const (
	QueryParameters   = "parameters"
	QuerySigningInfo  = "signingInfo"
	QuerySigningInfos = "signingInfos"
)

// QuerySigningInfosParams defines the params for the following queries:
// - 'custom/slashing/signingInfos'
type QuerySigningInfosParams struct {
	Page, Limit int
}

// NewQuerySigningInfosParams creates a new QuerySigningInfosParams instance
func NewQuerySigningInfosParams(page, limit int) QuerySigningInfosParams {
	return QuerySigningInfosParams{page, limit}
}
