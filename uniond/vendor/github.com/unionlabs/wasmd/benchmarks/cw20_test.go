package benchmarks

type cw20InitMsg struct {
	Name            string    `json:"name"`
	Symbol          string    `json:"symbol"`
	Decimals        uint8     `json:"decimals"`
	InitialBalances []balance `json:"initial_balances"`
}

type balance struct {
	Address string `json:"address"`
	Amount  uint64 `json:"amount,string"`
}

type cw20ExecMsg struct {
	Transfer *transferMsg `json:"transfer,omitempty"`
}

type transferMsg struct {
	Recipient string `json:"recipient"`
	Amount    uint64 `json:"amount,string"`
}
