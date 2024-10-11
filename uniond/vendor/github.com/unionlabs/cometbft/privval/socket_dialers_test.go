package privval

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/ed25519"
)

func getDialerTestCases(t *testing.T) []dialerTestCase {
	t.Helper()
	tcpAddr := GetFreeLocalhostAddrPort()
	unixFilePath, err := testUnixAddr()
	require.NoError(t, err)
	unixAddr := "unix://" + unixFilePath

	return []dialerTestCase{
		{
			addr:   tcpAddr,
			dialer: DialTCPFn(tcpAddr, testTimeoutReadWrite, ed25519.GenPrivKey()),
		},
		{
			addr:   unixAddr,
			dialer: DialUnixFn(unixFilePath),
		},
	}
}

func TestIsConnTimeoutForFundamentalTimeouts(t *testing.T) {
	// Generate a networking timeout
	tcpAddr := GetFreeLocalhostAddrPort()
	dialer := DialTCPFn(tcpAddr, time.Millisecond, ed25519.GenPrivKey())
	_, err := dialer()
	require.Error(t, err)
	assert.True(t, IsConnTimeout(err))
}

func TestIsConnTimeoutForWrappedConnTimeouts(t *testing.T) {
	tcpAddr := GetFreeLocalhostAddrPort()
	dialer := DialTCPFn(tcpAddr, time.Millisecond, ed25519.GenPrivKey())
	_, err := dialer()
	require.Error(t, err)
	err = fmt.Errorf("%v: %w", err, ErrConnectionTimeout)
	assert.True(t, IsConnTimeout(err))
}
