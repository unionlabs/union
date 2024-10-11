package types

import (
	"github.com/cosmos/cosmos-sdk/types/query"
)

// Querier path constants
const (
	QueryBalance     = "balance"
	QueryAllBalances = "all_balances"
	QueryTotalSupply = "total_supply"
	QuerySupplyOf    = "supply_of"
)

// NewQueryBalanceRequest creates a new instance of QueryBalanceRequest.
func NewQueryBalanceRequest(addr, denom string) *QueryBalanceRequest {
	return &QueryBalanceRequest{Address: addr, Denom: denom}
}

// NewQueryAllBalancesRequest creates a new instance of QueryAllBalancesRequest.
func NewQueryAllBalancesRequest(addr string, req *query.PageRequest, resolveDenom bool) *QueryAllBalancesRequest {
	return &QueryAllBalancesRequest{Address: addr, Pagination: req, ResolveDenom: resolveDenom}
}

// NewQuerySpendableBalancesRequest creates a new instance of a
// QuerySpendableBalancesRequest.
func NewQuerySpendableBalancesRequest(addr string, req *query.PageRequest) *QuerySpendableBalancesRequest {
	return &QuerySpendableBalancesRequest{Address: addr, Pagination: req}
}

// NewQuerySpendableBalanceByDenomRequest creates a new instance of a
// QuerySpendableBalanceByDenomRequest.
func NewQuerySpendableBalanceByDenomRequest(addr, denom string) *QuerySpendableBalanceByDenomRequest {
	return &QuerySpendableBalanceByDenomRequest{Address: addr, Denom: denom}
}

// QueryTotalSupplyParams defines the params for the following queries:
// - 'custom/bank/totalSupply'
type QueryTotalSupplyParams struct {
	Page, Limit int
}

// NewQueryTotalSupplyParams creates a new instance to query the total supply
func NewQueryTotalSupplyParams(page, limit int) QueryTotalSupplyParams {
	return QueryTotalSupplyParams{page, limit}
}

// QuerySupplyOfParams defines the params for the following queries:
// - 'custom/bank/totalSupplyOf'
type QuerySupplyOfParams struct {
	Denom string
}

// NewQuerySupplyOfParams creates a new instance to query the total supply
// of a given denomination
func NewQuerySupplyOfParams(denom string) QuerySupplyOfParams {
	return QuerySupplyOfParams{denom}
}
