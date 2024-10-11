package mempool

import (
	"errors"

	abcicli "github.com/cometbft/cometbft/abci/client"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/service"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/types"
)

// NopMempool is a mempool that does nothing.
//
// The ABCI app is responsible for storing, disseminating, and proposing transactions.
// See [ADR-111](../docs/architecture/adr-111-nop-mempool.md).
type NopMempool struct{}

// errNotAllowed indicates that the operation is not allowed with `nop` mempool.
var errNotAllowed = errors.New("not allowed with `nop` mempool")

var _ Mempool = &NopMempool{}

// CheckTx always returns an error.
func (*NopMempool) CheckTx(types.Tx, p2p.ID) (*abcicli.ReqRes, error) {
	return nil, errNotAllowed
}

// RemoveTxByKey always returns an error.
func (*NopMempool) RemoveTxByKey(types.TxKey) error { return errNotAllowed }

// ReapMaxBytesMaxGas always returns nil.
func (*NopMempool) ReapMaxBytesMaxGas(int64, int64) types.Txs { return nil }

// ReapMaxTxs always returns nil.
func (*NopMempool) ReapMaxTxs(int) types.Txs { return nil }

// Lock does nothing.
func (*NopMempool) Lock() {}

// Unlock does nothing.
func (*NopMempool) Unlock() {}

func (*NopMempool) PreUpdate() {}

// Update does nothing.
func (*NopMempool) Update(
	int64,
	types.Txs,
	[]*abci.ExecTxResult,
	PreCheckFunc,
	PostCheckFunc,
) error {
	return nil
}

// FlushAppConn does nothing.
func (*NopMempool) FlushAppConn() error { return nil }

// Flush does nothing.
func (*NopMempool) Flush() {}

// TxsAvailable always returns nil.
func (*NopMempool) TxsAvailable() <-chan struct{} {
	return nil
}

// EnableTxsAvailable does nothing.
func (*NopMempool) EnableTxsAvailable() {}

// Size always returns 0.
func (*NopMempool) Size() int { return 0 }

// SizeBytes always returns 0.
func (*NopMempool) SizeBytes() int64 { return 0 }

// NopMempoolReactor is a mempool reactor that does nothing.
type NopMempoolReactor struct {
	service.BaseService
}

// NewNopMempoolReactor returns a new `nop` reactor.
//
// To be used only in RPC.
func NewNopMempoolReactor() *NopMempoolReactor {
	return &NopMempoolReactor{*service.NewBaseService(nil, "NopMempoolReactor", nil)}
}

var _ p2p.Reactor = &NopMempoolReactor{}

// WaitSync always returns false.
func (*NopMempoolReactor) WaitSync() bool { return false }

// GetChannels always returns nil.
func (*NopMempoolReactor) GetChannels() []*p2p.ChannelDescriptor { return nil }

// AddPeer does nothing.
func (*NopMempoolReactor) AddPeer(p2p.Peer) {}

// InitPeer always returns nil.
func (*NopMempoolReactor) InitPeer(p2p.Peer) p2p.Peer { return nil }

// RemovePeer does nothing.
func (*NopMempoolReactor) RemovePeer(p2p.Peer, any) {}

// Receive does nothing.
func (*NopMempoolReactor) Receive(p2p.Envelope) {}

// SetSwitch does nothing.
func (*NopMempoolReactor) SetSwitch(*p2p.Switch) {}
