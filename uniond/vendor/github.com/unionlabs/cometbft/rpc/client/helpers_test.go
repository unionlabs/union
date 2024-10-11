package client_test

import (
	"errors"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/rpc/client"
	"github.com/cometbft/cometbft/rpc/client/mock"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
)

func TestWaitForHeight(t *testing.T) {
	assert, require := assert.New(t), require.New(t)

	// test with error result - immediate failure
	m := &mock.StatusMock{
		Call: mock.Call{
			Error: errors.New("bye"),
		},
	}
	r := mock.NewStatusRecorder(m)

	// connection failure always leads to error
	err := client.WaitForHeight(r, 8, nil)
	require.Error(err)
	require.Equal("bye", err.Error())
	// we called status once to check
	require.Len(r.Calls, 1)

	// now set current block height to 10
	m.Call = mock.Call{
		Response: &ctypes.ResultStatus{SyncInfo: ctypes.SyncInfo{LatestBlockHeight: 10}},
	}

	// we will not wait for more than 10 blocks
	err = client.WaitForHeight(r, 40, nil)
	require.Error(err)
	require.True(strings.Contains(err.Error(), "aborting"))
	// we called status once more to check
	require.Len(r.Calls, 2)

	// waiting for the past returns immediately
	err = client.WaitForHeight(r, 5, nil)
	require.NoError(err)
	// we called status once more to check
	require.Len(r.Calls, 3)

	// since we can't update in a background goroutine (test --race)
	// we use the callback to update the status height
	myWaiter := func(delta int64) error {
		// update the height for the next call
		m.Call.Response = &ctypes.ResultStatus{SyncInfo: ctypes.SyncInfo{LatestBlockHeight: 15}}
		return client.DefaultWaitStrategy(delta)
	}

	// we wait for a few blocks
	err = client.WaitForHeight(r, 12, myWaiter)
	require.NoError(err)
	// we called status once to check
	require.Len(r.Calls, 5)

	pre := r.Calls[3]
	require.NoError(pre.Error)
	prer, ok := pre.Response.(*ctypes.ResultStatus)
	require.True(ok)
	assert.Equal(int64(10), prer.SyncInfo.LatestBlockHeight)

	post := r.Calls[4]
	require.NoError(post.Error)
	postr, ok := post.Response.(*ctypes.ResultStatus)
	require.True(ok)
	assert.Equal(int64(15), postr.SyncInfo.LatestBlockHeight)
}
