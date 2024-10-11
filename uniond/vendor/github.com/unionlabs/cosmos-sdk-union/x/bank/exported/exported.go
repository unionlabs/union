package exported

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// GenesisBalance defines a genesis balance interface that allows for account
// address and balance retrieval.
type GenesisBalance interface {
	GetAddress() string
	GetCoins() sdk.Coins
}
