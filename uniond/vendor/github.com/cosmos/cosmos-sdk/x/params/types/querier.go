package types

// Querier path constants
const (
	QueryParams = "params"
)

// QuerySubspaceParams defines the params for querying module params by a given
// subspace and key.
type QuerySubspaceParams struct {
	Subspace string
	Key      string
}

// SubspaceParamsResponse defines the response for quering parameters by subspace.
type SubspaceParamsResponse struct {
	Subspace string
	Key      string
	Value    string
}

func NewQuerySubspaceParams(ss, key string) QuerySubspaceParams {
	return QuerySubspaceParams{
		Subspace: ss,
		Key:      key,
	}
}

func NewSubspaceParamsResponse(ss, key, value string) SubspaceParamsResponse {
	return SubspaceParamsResponse{
		Subspace: ss,
		Key:      key,
		Value:    value,
	}
}
