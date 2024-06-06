package types

// See https://github.com/CosmWasm/token-bindings/blob/main/packages/bindings/src/query.rs
type DeferredAckQuery struct {
	/// Given a subdenom minted by a contract via `TokenFactoryMsg::MintTokens`,
	/// returns the full denom as used by `BankMsg::Send`.
	Params *GetParams `json:"params,omitempty"`
}

// query types

type GetParams struct{}

// responses

type ParamsResponse struct {
	Params Params `json:"params"`
}
