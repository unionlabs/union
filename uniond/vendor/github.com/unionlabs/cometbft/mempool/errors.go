package mempool

import (
	"errors"
	"fmt"
)

// ErrTxNotFound is returned to the client if tx is not found in mempool.
var ErrTxNotFound = errors.New("transaction not found in mempool")

// ErrTxInCache is returned to the client if we saw tx earlier.
var ErrTxInCache = errors.New("tx already exists in cache")

// ErrRecheckFull is returned when checking if the mempool is full and
// rechecking is still in progress after a new block was committed.
var ErrRecheckFull = errors.New("mempool is still rechecking after a new committed block, so it is considered as full")

// ErrTxTooLarge defines an error when a transaction is too big to be sent in a
// message to other peers.
type ErrTxTooLarge struct {
	Max    int
	Actual int
}

func (e ErrTxTooLarge) Error() string {
	return fmt.Sprintf("Tx too large. Max size is %d, but got %d", e.Max, e.Actual)
}

// ErrMempoolIsFull defines an error where CometBFT and the application cannot
// handle that much load.
type ErrMempoolIsFull struct {
	NumTxs      int
	MaxTxs      int
	TxsBytes    int64
	MaxTxsBytes int64
	RecheckFull bool
}

func (e ErrMempoolIsFull) Error() string {
	return fmt.Sprintf(
		"mempool is full: number of txs %d (max: %d), total txs bytes %d (max: %d)",
		e.NumTxs,
		e.MaxTxs,
		e.TxsBytes,
		e.MaxTxsBytes,
	)
}

// ErrPreCheck defines an error where a transaction fails a pre-check.
type ErrPreCheck struct {
	Err error
}

func (e ErrPreCheck) Error() string {
	return fmt.Sprintf("tx pre check: %v", e.Err)
}

func (e ErrPreCheck) Unwrap() error {
	return e.Err
}

// IsPreCheckError returns true if err is due to pre check failure.
func IsPreCheckError(err error) bool {
	return errors.As(err, &ErrPreCheck{})
}

type ErrAppConnMempool struct {
	Err error
}

func (e ErrAppConnMempool) Error() string {
	return fmt.Sprintf("appConn mempool: %v", e.Err)
}

func (e ErrAppConnMempool) Unwrap() error {
	return e.Err
}

type ErrFlushAppConn struct {
	Err error
}

func (e ErrFlushAppConn) Error() string {
	return fmt.Sprintf("flush appConn mempool: %v", e.Err)
}

func (e ErrFlushAppConn) Unwrap() error {
	return e.Err
}
