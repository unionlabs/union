package mempool

import (
	"sync"
	"sync/atomic"

	"github.com/cometbft/cometbft/types"
)

// mempoolTx is an entry in the mempool
type mempoolTx struct {
	height    int64    // height that this tx had been validated in
	gasWanted int64    // amount of gas this tx states it will require
	tx        types.Tx // validated by the application

	// ids of peers who've sent us this tx (as a map for quick lookups).
	// senders: PeerID -> bool
	senders sync.Map
}

// Height returns the height for this transaction
func (memTx *mempoolTx) Height() int64 {
	return atomic.LoadInt64(&memTx.height)
}

func (memTx *mempoolTx) isSender(peerID uint16) bool {
	_, ok := memTx.senders.Load(peerID)
	return ok
}

func (memTx *mempoolTx) addSender(senderID uint16) bool {
	_, added := memTx.senders.LoadOrStore(senderID, true)
	return added
}
