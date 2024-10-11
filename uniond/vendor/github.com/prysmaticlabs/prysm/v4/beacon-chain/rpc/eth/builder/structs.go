package builder

type ExpectedWithdrawalsResponse struct {
	Data                []*ExpectedWithdrawal `json:"data"`
	ExecutionOptimistic bool                  `json:"execution_optimistic"`
	Finalized           bool                  `json:"finalized"`
}

type ExpectedWithdrawal struct {
	Address        string `json:"address" hex:"true"`
	Amount         string `json:"amount"`
	Index          string `json:"index"`
	ValidatorIndex string `json:"validator_index"`
}
