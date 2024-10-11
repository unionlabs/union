package mempool

import (
	memprotos "github.com/cometbft/cometbft/api/cometbft/mempool/v1"
	"github.com/cometbft/cometbft/types"
)

var (
	_ types.Wrapper   = &memprotos.Txs{}
	_ types.Unwrapper = &memprotos.Message{}
)
