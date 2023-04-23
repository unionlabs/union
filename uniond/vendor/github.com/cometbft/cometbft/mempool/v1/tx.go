// Deprecated: Priority mempool will be removed in the next major release.
package v1

import (
	"sync"
	"time"

	"github.com/cometbft/cometbft/types"
)

// WrappedTx defines a wrapper around a raw transaction with additional metadata
// that is used for indexing.
type WrappedTx struct {
	tx        types.Tx    // the original transaction data
	hash      types.TxKey // the transaction hash
	height    int64       // height when this transaction was initially checked (for expiry)
	timestamp time.Time   // time when transaction was entered (for TTL)

	mtx       sync.Mutex
	gasWanted int64           // app: gas required to execute this transaction
	priority  int64           // app: priority value for this transaction
	sender    string          // app: assigned sender label
	peers     map[uint16]bool // peer IDs who have sent us this transaction
}

// Size reports the size of the raw transaction in bytes.
func (w *WrappedTx) Size() int64 { return int64(len(w.tx)) }

// SetPeer adds the specified peer ID as a sender of w.
func (w *WrappedTx) SetPeer(id uint16) {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	if w.peers == nil {
		w.peers = map[uint16]bool{id: true}
	} else {
		w.peers[id] = true
	}
}

// HasPeer reports whether the specified peer ID is a sender of w.
func (w *WrappedTx) HasPeer(id uint16) bool {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	_, ok := w.peers[id]
	return ok
}

// SetGasWanted sets the application-assigned gas requirement of w.
func (w *WrappedTx) SetGasWanted(gas int64) {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	w.gasWanted = gas
}

// GasWanted reports the application-assigned gas requirement of w.
func (w *WrappedTx) GasWanted() int64 {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	return w.gasWanted
}

// SetSender sets the application-assigned sender of w.
func (w *WrappedTx) SetSender(sender string) {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	w.sender = sender
}

// Sender reports the application-assigned sender of w.
func (w *WrappedTx) Sender() string {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	return w.sender
}

// SetPriority sets the application-assigned priority of w.
func (w *WrappedTx) SetPriority(p int64) {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	w.priority = p
}

// Priority reports the application-assigned priority of w.
func (w *WrappedTx) Priority() int64 {
	w.mtx.Lock()
	defer w.mtx.Unlock()
	return w.priority
}
