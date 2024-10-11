package testutil

import (
	"math/big"
)

// MockExecutionChainInfoFetcher is a fake implementation of the powchain.ChainInfoFetcher
type MockExecutionChainInfoFetcher struct {
	CurrEndpoint string
	CurrError    error
}

func (*MockExecutionChainInfoFetcher) GenesisExecutionChainInfo() (uint64, *big.Int) {
	return uint64(0), &big.Int{}
}

func (*MockExecutionChainInfoFetcher) ExecutionClientConnected() bool {
	return true
}

func (m *MockExecutionChainInfoFetcher) ExecutionClientEndpoint() string {
	return m.CurrEndpoint
}

func (m *MockExecutionChainInfoFetcher) ExecutionClientConnectionErr() error {
	return m.CurrError
}
