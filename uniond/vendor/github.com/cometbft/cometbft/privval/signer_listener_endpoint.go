package privval

import (
	"fmt"
	"net"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	privvalproto "github.com/cometbft/cometbft/proto/tendermint/privval"
)

// SignerListenerEndpointOption sets an optional parameter on the SignerListenerEndpoint.
type SignerListenerEndpointOption func(*SignerListenerEndpoint)

// SignerListenerEndpointTimeoutReadWrite sets the read and write timeout for
// connections from external signing processes.
//
// Default: 5s
func SignerListenerEndpointTimeoutReadWrite(timeout time.Duration) SignerListenerEndpointOption {
	return func(sl *SignerListenerEndpoint) { sl.signerEndpoint.timeoutReadWrite = timeout }
}

// SignerListenerEndpoint listens for an external process to dial in and keeps
// the connection alive by dropping and reconnecting.
//
// The process will send pings every ~3s (read/write timeout * 2/3) to keep the
// connection alive.
type SignerListenerEndpoint struct {
	signerEndpoint

	listener              net.Listener
	connectRequestCh      chan struct{}
	connectionAvailableCh chan net.Conn

	timeoutAccept time.Duration
	pingTimer     *time.Ticker
	pingInterval  time.Duration

	instanceMtx cmtsync.Mutex // Ensures instance public methods access, i.e. SendRequest
}

// NewSignerListenerEndpoint returns an instance of SignerListenerEndpoint.
func NewSignerListenerEndpoint(
	logger log.Logger,
	listener net.Listener,
	options ...SignerListenerEndpointOption,
) *SignerListenerEndpoint {
	sl := &SignerListenerEndpoint{
		listener:      listener,
		timeoutAccept: defaultTimeoutAcceptSeconds * time.Second,
	}

	sl.BaseService = *service.NewBaseService(logger, "SignerListenerEndpoint", sl)
	sl.signerEndpoint.timeoutReadWrite = defaultTimeoutReadWriteSeconds * time.Second

	for _, optionFunc := range options {
		optionFunc(sl)
	}

	return sl
}

// OnStart implements service.Service.
func (sl *SignerListenerEndpoint) OnStart() error {
	sl.connectRequestCh = make(chan struct{})
	sl.connectionAvailableCh = make(chan net.Conn)

	// NOTE: ping timeout must be less than read/write timeout
	sl.pingInterval = time.Duration(sl.signerEndpoint.timeoutReadWrite.Milliseconds()*2/3) * time.Millisecond
	sl.pingTimer = time.NewTicker(sl.pingInterval)

	go sl.serviceLoop()
	go sl.pingLoop()

	sl.connectRequestCh <- struct{}{}

	return nil
}

// OnStop implements service.Service
func (sl *SignerListenerEndpoint) OnStop() {
	sl.instanceMtx.Lock()
	defer sl.instanceMtx.Unlock()
	_ = sl.Close()

	// Stop listening
	if sl.listener != nil {
		if err := sl.listener.Close(); err != nil {
			sl.Logger.Error("Closing Listener", "err", err)
			sl.listener = nil
		}
	}

	sl.pingTimer.Stop()
}

// WaitForConnection waits maxWait for a connection or returns a timeout error
func (sl *SignerListenerEndpoint) WaitForConnection(maxWait time.Duration) error {
	sl.instanceMtx.Lock()
	defer sl.instanceMtx.Unlock()
	return sl.ensureConnection(maxWait)
}

// SendRequest ensures there is a connection, sends a request and waits for a response
func (sl *SignerListenerEndpoint) SendRequest(request privvalproto.Message) (*privvalproto.Message, error) {
	sl.instanceMtx.Lock()
	defer sl.instanceMtx.Unlock()

	err := sl.ensureConnection(sl.timeoutAccept)
	if err != nil {
		return nil, err
	}

	err = sl.WriteMessage(request)
	if err != nil {
		return nil, err
	}

	res, err := sl.ReadMessage()
	if err != nil {
		return nil, err
	}

	// Reset pingTimer to avoid sending unnecessary pings.
	sl.pingTimer.Reset(sl.pingInterval)

	return &res, nil
}

func (sl *SignerListenerEndpoint) ensureConnection(maxWait time.Duration) error {
	if sl.IsConnected() {
		return nil
	}

	// Is there a connection ready? then use it
	if sl.GetAvailableConnection(sl.connectionAvailableCh) {
		return nil
	}

	// block until connected or timeout
	sl.Logger.Info("SignerListener: Blocking for connection")
	sl.triggerConnect()
	err := sl.WaitConnection(sl.connectionAvailableCh, maxWait)
	if err != nil {
		return err
	}

	return nil
}

func (sl *SignerListenerEndpoint) acceptNewConnection() (net.Conn, error) {
	if !sl.IsRunning() || sl.listener == nil {
		return nil, fmt.Errorf("endpoint is closing")
	}

	// wait for a new conn
	sl.Logger.Info("SignerListener: Listening for new connection")
	conn, err := sl.listener.Accept()
	if err != nil {
		return nil, err
	}

	return conn, nil
}

func (sl *SignerListenerEndpoint) triggerConnect() {
	select {
	case sl.connectRequestCh <- struct{}{}:
	default:
	}
}

func (sl *SignerListenerEndpoint) triggerReconnect() {
	sl.DropConnection()
	sl.triggerConnect()
}

func (sl *SignerListenerEndpoint) serviceLoop() {
	for {
		select {
		case <-sl.connectRequestCh:
			{
				conn, err := sl.acceptNewConnection()
				if err == nil {
					sl.Logger.Info("SignerListener: Connected")

					// We have a good connection, wait for someone that needs one otherwise cancellation
					select {
					case sl.connectionAvailableCh <- conn:
					case <-sl.Quit():
						return
					}
				}

				select {
				case sl.connectRequestCh <- struct{}{}:
				default:
				}
			}
		case <-sl.Quit():
			return
		}
	}
}

func (sl *SignerListenerEndpoint) pingLoop() {
	for {
		select {
		case <-sl.pingTimer.C:
			{
				_, err := sl.SendRequest(mustWrapMsg(&privvalproto.PingRequest{}))
				if err != nil {
					sl.Logger.Error("SignerListener: Ping timeout")
					sl.triggerReconnect()
				}
			}
		case <-sl.Quit():
			return
		}
	}
}
