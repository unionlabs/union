package privval

import (
	"time"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
)

const (
	defaultMaxDialRetries        = 10
	defaultRetryWaitMilliseconds = 100
)

// SignerServiceEndpointOption sets an optional parameter on the SignerDialerEndpoint.
type SignerServiceEndpointOption func(*SignerDialerEndpoint)

// SignerDialerEndpointTimeoutReadWrite sets the read and write timeout for
// connections from client processes.
func SignerDialerEndpointTimeoutReadWrite(timeout time.Duration) SignerServiceEndpointOption {
	return func(ss *SignerDialerEndpoint) { ss.timeoutReadWrite = timeout }
}

// SignerDialerEndpointConnRetries sets the amount of attempted retries to
// acceptNewConnection.
func SignerDialerEndpointConnRetries(retries int) SignerServiceEndpointOption {
	return func(ss *SignerDialerEndpoint) { ss.maxConnRetries = retries }
}

// SignerDialerEndpointRetryWaitInterval sets the retry wait interval to a
// custom value.
func SignerDialerEndpointRetryWaitInterval(interval time.Duration) SignerServiceEndpointOption {
	return func(ss *SignerDialerEndpoint) { ss.retryWait = interval }
}

// SignerDialerEndpoint dials using its dialer and responds to any signature
// requests using its privVal.
type SignerDialerEndpoint struct {
	signerEndpoint

	dialer SocketDialer

	retryWait      time.Duration
	maxConnRetries int
}

// NewSignerDialerEndpoint returns a SignerDialerEndpoint that will dial using the given
// dialer and respond to any signature requests over the connection
// using the given privVal.
func NewSignerDialerEndpoint(
	logger log.Logger,
	dialer SocketDialer,
	options ...SignerServiceEndpointOption,
) *SignerDialerEndpoint {

	sd := &SignerDialerEndpoint{
		dialer:         dialer,
		retryWait:      defaultRetryWaitMilliseconds * time.Millisecond,
		maxConnRetries: defaultMaxDialRetries,
	}

	sd.BaseService = *service.NewBaseService(logger, "SignerDialerEndpoint", sd)
	sd.signerEndpoint.timeoutReadWrite = defaultTimeoutReadWriteSeconds * time.Second

	for _, optionFunc := range options {
		optionFunc(sd)
	}

	return sd
}

func (sd *SignerDialerEndpoint) ensureConnection() error {
	if sd.IsConnected() {
		return nil
	}

	retries := 0
	for retries < sd.maxConnRetries {
		conn, err := sd.dialer()

		if err != nil {
			retries++
			sd.Logger.Debug("SignerDialer: Reconnection failed", "retries", retries, "max", sd.maxConnRetries, "err", err)
			// Wait between retries
			time.Sleep(sd.retryWait)
		} else {
			sd.SetConnection(conn)
			sd.Logger.Debug("SignerDialer: Connection Ready")
			return nil
		}
	}

	sd.Logger.Debug("SignerDialer: Max retries exceeded", "retries", retries, "max", sd.maxConnRetries)

	return ErrNoConnection
}
