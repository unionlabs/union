package accounts

import (
	"context"
	"fmt"

	"cosmossdk.io/x/accounts/internal/implementation"
	v1 "cosmossdk.io/x/accounts/v1"
)

var _ v1.QueryServer = &queryServer{}

// NewQueryServer initializes a new instance of QueryServer.
// It precalculates and stores schemas for efficient schema retrieval.
func NewQueryServer(k Keeper) v1.QueryServer {
	// Pre-calculate schemas for efficient retrieval.
	schemas := v1.MakeAccountsSchemas(k.accounts)
	return &queryServer{
		k:       k,
		schemas: schemas, // Store precalculated schemas.
	}
}

type queryServer struct {
	k       Keeper
	schemas map[string]*v1.SchemaResponse // Stores precalculated schemas.
}

func (q *queryServer) AccountQuery(ctx context.Context, request *v1.AccountQueryRequest) (*v1.AccountQueryResponse, error) {
	// get target addr
	targetAddr, err := q.k.addressCodec.StringToBytes(request.Target)
	if err != nil {
		return nil, err
	}

	// decode req into boxed concrete type
	queryReq, err := implementation.UnpackAnyRaw(request.Request)
	if err != nil {
		return nil, err
	}
	// run query
	resp, err := q.k.Query(ctx, targetAddr, queryReq)
	if err != nil {
		return nil, err
	}

	// encode response
	respAny, err := implementation.PackAny(resp)
	if err != nil {
		return nil, err
	}

	return &v1.AccountQueryResponse{
		Response: respAny,
	}, nil
}

// Schema retrieves the schema for a given account type.
// It checks the precalculated schemas and returns an error if the schema is not found.
func (q *queryServer) Schema(_ context.Context, request *v1.SchemaRequest) (*v1.SchemaResponse, error) {
	// Fetch schema from precalculated schemas.
	schema, ok := q.schemas[request.AccountType]
	if !ok {
		return nil, fmt.Errorf("%w: %s", errAccountTypeNotFound, request.AccountType)
	}

	return schema, nil
}

func (q *queryServer) AccountType(ctx context.Context, request *v1.AccountTypeRequest) (*v1.AccountTypeResponse, error) {
	addr, err := q.k.addressCodec.StringToBytes(request.Address)
	if err != nil {
		return nil, err
	}
	accType, err := q.k.AccountsByType.Get(ctx, addr)
	if err != nil {
		return nil, err
	}
	return &v1.AccountTypeResponse{
		AccountType: accType,
	}, nil
}

func (q *queryServer) AccountNumber(ctx context.Context, request *v1.AccountNumberRequest) (*v1.AccountNumberResponse, error) {
	addr, err := q.k.addressCodec.StringToBytes(request.Address)
	if err != nil {
		return nil, err
	}
	number, err := q.k.AccountByNumber.Get(ctx, addr)
	if err != nil {
		return nil, err
	}
	return &v1.AccountNumberResponse{Number: number}, nil
}

const (
	// TODO(tip): evaluate if the following numbers should be parametrised over state, or over the node.
	SimulateAuthenticateGasLimit   = 1_000_000
	SimulateBundlerPaymentGasLimit = SimulateAuthenticateGasLimit
	ExecuteGasLimit                = SimulateAuthenticateGasLimit
)
