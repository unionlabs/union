package privval

import (
	"errors"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtnet "github.com/cometbft/cometbft/internal/net"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/types"
)

var (
	testTimeoutAccept = defaultTimeoutAcceptSeconds * time.Second

	testTimeoutReadWrite    = 100 * time.Millisecond
	testTimeoutReadWrite2o3 = 60 * time.Millisecond // 2/3 of the other one
)

type dialerTestCase struct {
	addr   string
	dialer SocketDialer
}

// TestSignerRemoteRetryTCPOnly will test connection retry attempts over TCP. We
// don't need this for Unix sockets because the OS instantly knows the state of
// both ends of the socket connection. This basically causes the
// SignerDialerEndpoint.dialer() call inside SignerDialerEndpoint.acceptNewConnection() to return
// successfully immediately, putting an instant stop to any retry attempts.
func TestSignerRemoteRetryTCPOnly(t *testing.T) {
	var (
		attemptCh = make(chan int)
		retries   = 10
	)

	ln, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)

	// Continuously Accept connection and close {attempts} times
	go func(ln net.Listener, attemptCh chan<- int) {
		attempts := 0
		for {
			conn, err := ln.Accept()
			require.NoError(t, err)

			err = conn.Close()
			require.NoError(t, err)

			attempts++

			if attempts == retries {
				attemptCh <- attempts
				break
			}
		}
	}(ln, attemptCh)

	dialerEndpoint := NewSignerDialerEndpoint(
		log.TestingLogger(),
		DialTCPFn(ln.Addr().String(), testTimeoutReadWrite, ed25519.GenPrivKey()),
	)
	SignerDialerEndpointTimeoutReadWrite(time.Millisecond)(dialerEndpoint)
	SignerDialerEndpointConnRetries(retries)(dialerEndpoint)

	chainID := cmtrand.Str(12)
	mockPV := types.NewMockPV()
	signerServer := NewSignerServer(dialerEndpoint, chainID, mockPV)

	err = signerServer.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := signerServer.Stop(); err != nil {
			t.Error(err)
		}
	})

	select {
	case attempts := <-attemptCh:
		assert.Equal(t, retries, attempts)
	case <-time.After(1500 * time.Millisecond):
		t.Error("expected remote to observe connection attempts")
	}
}

func TestRetryConnToRemoteSigner(t *testing.T) {
	for _, tc := range getDialerTestCases(t) {
		var (
			logger           = log.TestingLogger()
			chainID          = cmtrand.Str(12)
			mockPV           = types.NewMockPV()
			endpointIsOpenCh = make(chan struct{})
			thisConnTimeout  = testTimeoutReadWrite
			listenerEndpoint = newSignerListenerEndpoint(logger, tc.addr, thisConnTimeout)
		)

		dialerEndpoint := NewSignerDialerEndpoint(
			logger,
			tc.dialer,
		)
		SignerDialerEndpointTimeoutReadWrite(testTimeoutReadWrite)(dialerEndpoint)
		SignerDialerEndpointConnRetries(10)(dialerEndpoint)

		signerServer := NewSignerServer(dialerEndpoint, chainID, mockPV)

		startListenerEndpointAsync(t, listenerEndpoint, endpointIsOpenCh)
		t.Cleanup(func() {
			if err := listenerEndpoint.Stop(); err != nil {
				t.Error(err)
			}
		})

		require.NoError(t, signerServer.Start())
		assert.True(t, signerServer.IsRunning())
		<-endpointIsOpenCh
		if err := signerServer.Stop(); err != nil {
			t.Error(err)
		}

		dialerEndpoint2 := NewSignerDialerEndpoint(
			logger,
			tc.dialer,
		)
		signerServer2 := NewSignerServer(dialerEndpoint2, chainID, mockPV)

		// let some pings pass
		require.NoError(t, signerServer2.Start())
		assert.True(t, signerServer2.IsRunning())
		t.Cleanup(func() {
			if err := signerServer2.Stop(); err != nil {
				t.Error(err)
			}
		})

		// give the client some time to re-establish the conn to the remote signer
		// should see sth like this in the logs:
		//
		// E[10016-01-10|17:12:46.128] Ping                                         err="remote signer timed out"
		// I[10016-01-10|17:16:42.447] Re-created connection to remote signer       impl=SocketVal
		time.Sleep(testTimeoutReadWrite * 2)
	}
}

func TestDuplicateListenReject(t *testing.T) {
	for _, tc := range getDialerTestCases(t) {
		var (
			logger           = log.TestingLogger()
			chainID          = cmtrand.Str(12)
			mockPV           = types.NewMockPV()
			endpointIsOpenCh = make(chan struct{})
			thisConnTimeout  = testTimeoutReadWrite
			listenerEndpoint = newSignerListenerEndpoint(logger, tc.addr, thisConnTimeout)
		)
		listenerEndpoint.timeoutAccept = defaultTimeoutAcceptSeconds / 2 * time.Second

		dialerEndpoint := NewSignerDialerEndpoint(
			logger,
			tc.dialer,
		)
		SignerDialerEndpointTimeoutReadWrite(testTimeoutReadWrite)(dialerEndpoint)
		SignerDialerEndpointConnRetries(10)(dialerEndpoint)

		signerServer := NewSignerServer(dialerEndpoint, chainID, mockPV)

		startListenerEndpointAsync(t, listenerEndpoint, endpointIsOpenCh)
		t.Cleanup(func() {
			if err := listenerEndpoint.Stop(); err != nil {
				t.Error(err)
			}
		})

		require.NoError(t, signerServer.Start())
		assert.True(t, signerServer.IsRunning())

		<-endpointIsOpenCh
		if err := signerServer.Stop(); err != nil {
			t.Error(err)
		}

		dialerEndpoint2 := NewSignerDialerEndpoint(
			logger,
			tc.dialer,
		)
		signerServer2 := NewSignerServer(dialerEndpoint2, chainID, mockPV)

		// let some pings pass
		require.NoError(t, signerServer2.Start())
		assert.True(t, signerServer2.IsRunning())

		// wait for successful connection
		for {
			if listenerEndpoint.IsConnected() {
				break
			}
		}

		// simulate ensureConnection, bypass triggerConnect default drop with multiple messages
		time.Sleep(100 * time.Millisecond)
		listenerEndpoint.triggerConnect()
		time.Sleep(100 * time.Millisecond)
		listenerEndpoint.triggerConnect()
		time.Sleep(100 * time.Millisecond)
		listenerEndpoint.triggerConnect()

		// simulate validator node running long enough for privval listen timeout multiple times
		// up to 1 timeout error is possible due to timing differences
		// Run 3 times longer than timeout to generate at least 2 accept errors
		time.Sleep(3 * defaultTimeoutAcceptSeconds * time.Second)
		t.Cleanup(func() {
			if err := signerServer2.Stop(); err != nil {
				t.Error(err)
			}
		})

		// after connect, there should not be more than 1 accept fail
		assert.LessOrEqual(t, listenerEndpoint.acceptFailCount.Load(), uint32(1))

		// give the client some time to re-establish the conn to the remote signer
		// should see sth like this in the logs:
		//
		// E[10016-01-10|17:12:46.128] Ping                                         err="remote signer timed out"
		// I[10016-01-10|17:16:42.447] Re-created connection to remote signer       impl=SocketVal
		time.Sleep(testTimeoutReadWrite * 2)
	}
}

func newSignerListenerEndpoint(logger log.Logger, addr string, timeoutReadWrite time.Duration) *SignerListenerEndpoint {
	proto, address := cmtnet.ProtocolAndAddress(addr)

	ln, err := net.Listen(proto, address)
	logger.Info("SignerListener: Listening", "proto", proto, "address", address)
	if err != nil {
		panic(err)
	}

	var listener net.Listener

	if proto == "unix" {
		unixLn := NewUnixListener(ln)
		UnixListenerTimeoutAccept(testTimeoutAccept)(unixLn)
		UnixListenerTimeoutReadWrite(timeoutReadWrite)(unixLn)
		listener = unixLn
	} else {
		tcpLn := NewTCPListener(ln, ed25519.GenPrivKey())
		TCPListenerTimeoutAccept(testTimeoutAccept)(tcpLn)
		TCPListenerTimeoutReadWrite(timeoutReadWrite)(tcpLn)
		listener = tcpLn
	}

	return NewSignerListenerEndpoint(
		logger,
		listener,
		SignerListenerEndpointTimeoutReadWrite(testTimeoutReadWrite),
	)
}

func startListenerEndpointAsync(t *testing.T, sle *SignerListenerEndpoint, endpointIsOpenCh chan struct{}) {
	t.Helper()
	go func(sle *SignerListenerEndpoint) {
		require.NoError(t, sle.Start())
		assert.True(t, sle.IsRunning())
		close(endpointIsOpenCh)
	}(sle)
}

func getMockEndpoints(
	t *testing.T,
	addr string,
	socketDialer SocketDialer,
) (*SignerListenerEndpoint, *SignerDialerEndpoint) {
	t.Helper()
	var (
		logger           = log.TestingLogger()
		endpointIsOpenCh = make(chan struct{})

		dialerEndpoint = NewSignerDialerEndpoint(
			logger,
			socketDialer,
		)

		listenerEndpoint = newSignerListenerEndpoint(logger, addr, testTimeoutReadWrite)
	)

	SignerDialerEndpointTimeoutReadWrite(testTimeoutReadWrite)(dialerEndpoint)
	SignerDialerEndpointConnRetries(1e6)(dialerEndpoint)

	startListenerEndpointAsync(t, listenerEndpoint, endpointIsOpenCh)

	require.NoError(t, dialerEndpoint.Start())
	assert.True(t, dialerEndpoint.IsRunning())

	<-endpointIsOpenCh

	return listenerEndpoint, dialerEndpoint
}

func TestSignerListenerEndpointServiceLoop(t *testing.T) {
	listenerEndpoint := NewSignerListenerEndpoint(
		log.TestingLogger(),
		&testListener{initialErrs: 5},
	)

	require.NoError(t, listenerEndpoint.Start())
	require.NoError(t, listenerEndpoint.WaitForConnection(time.Second))
}

type testListener struct {
	net.Listener
	initialErrs int
}

func (l *testListener) Accept() (net.Conn, error) {
	if l.initialErrs > 0 {
		l.initialErrs--

		return nil, errors.New("accept error")
	}

	return nil, nil // Note this doesn't actually return a valid connection, it just doesn't error.
}
