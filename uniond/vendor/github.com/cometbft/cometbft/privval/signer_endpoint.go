package privval

import (
	"fmt"
	"net"
	"time"

	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	privvalproto "github.com/cometbft/cometbft/proto/tendermint/privval"
)

const (
	defaultTimeoutReadWriteSeconds = 5
)

type signerEndpoint struct {
	service.BaseService

	connMtx cmtsync.Mutex
	conn    net.Conn

	timeoutReadWrite time.Duration
}

// Close closes the underlying net.Conn.
func (se *signerEndpoint) Close() error {
	se.DropConnection()
	return nil
}

// IsConnected indicates if there is an active connection
func (se *signerEndpoint) IsConnected() bool {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()
	return se.isConnected()
}

// TryGetConnection retrieves a connection if it is already available
func (se *signerEndpoint) GetAvailableConnection(connectionAvailableCh chan net.Conn) bool {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()

	// Is there a connection ready?
	select {
	case se.conn = <-connectionAvailableCh:
		return true
	default:
	}
	return false
}

// TryGetConnection retrieves a connection if it is already available
func (se *signerEndpoint) WaitConnection(connectionAvailableCh chan net.Conn, maxWait time.Duration) error {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()

	select {
	case se.conn = <-connectionAvailableCh:
	case <-time.After(maxWait):
		return ErrConnectionTimeout
	}

	return nil
}

// SetConnection replaces the current connection object
func (se *signerEndpoint) SetConnection(newConnection net.Conn) {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()
	se.conn = newConnection
}

// IsConnected indicates if there is an active connection
func (se *signerEndpoint) DropConnection() {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()
	se.dropConnection()
}

// ReadMessage reads a message from the endpoint
func (se *signerEndpoint) ReadMessage() (msg privvalproto.Message, err error) {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()

	if !se.isConnected() {
		return msg, fmt.Errorf("endpoint is not connected: %w", ErrNoConnection)
	}
	// Reset read deadline
	deadline := time.Now().Add(se.timeoutReadWrite)

	err = se.conn.SetReadDeadline(deadline)
	if err != nil {
		return
	}
	const maxRemoteSignerMsgSize = 1024 * 10
	protoReader := protoio.NewDelimitedReader(se.conn, maxRemoteSignerMsgSize)
	_, err = protoReader.ReadMsg(&msg)
	if _, ok := err.(timeoutError); ok {
		if err != nil {
			err = fmt.Errorf("%v: %w", err, ErrReadTimeout)
		} else {
			err = fmt.Errorf("empty error: %w", ErrReadTimeout)
		}

		se.Logger.Debug("Dropping [read]", "obj", se)
		se.dropConnection()
	}

	return
}

// WriteMessage writes a message from the endpoint
func (se *signerEndpoint) WriteMessage(msg privvalproto.Message) (err error) {
	se.connMtx.Lock()
	defer se.connMtx.Unlock()

	if !se.isConnected() {
		return fmt.Errorf("endpoint is not connected: %w", ErrNoConnection)
	}

	protoWriter := protoio.NewDelimitedWriter(se.conn)

	// Reset read deadline
	deadline := time.Now().Add(se.timeoutReadWrite)
	err = se.conn.SetWriteDeadline(deadline)
	if err != nil {
		return
	}

	_, err = protoWriter.WriteMsg(&msg)
	if _, ok := err.(timeoutError); ok {
		if err != nil {
			err = fmt.Errorf("%v: %w", err, ErrWriteTimeout)
		} else {
			err = fmt.Errorf("empty error: %w", ErrWriteTimeout)
		}
		se.dropConnection()
	}

	return
}

func (se *signerEndpoint) isConnected() bool {
	return se.conn != nil
}

func (se *signerEndpoint) dropConnection() {
	if se.conn != nil {
		if err := se.conn.Close(); err != nil {
			se.Logger.Error("signerEndpoint::dropConnection", "err", err)
		}
		se.conn = nil
	}
}
