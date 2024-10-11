package mock_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/rpc/client/mock"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
)

func TestStatus(t *testing.T) {
	assert, require := assert.New(t), require.New(t)

	m := &mock.StatusMock{
		Call: mock.Call{
			Response: &ctypes.ResultStatus{
				SyncInfo: ctypes.SyncInfo{
					LatestBlockHash:   bytes.HexBytes("block"),
					LatestAppHash:     bytes.HexBytes("app"),
					LatestBlockHeight: 10,
				},
			},
		},
	}

	r := mock.NewStatusRecorder(m)
	require.Empty(r.Calls)

	// make sure response works proper
	status, err := r.Status(context.Background())
	require.NoError(err, "%+v", err)
	assert.EqualValues("block", status.SyncInfo.LatestBlockHash)
	assert.EqualValues(10, status.SyncInfo.LatestBlockHeight)

	// make sure recorder works properly
	require.Len(r.Calls, 1)
	rs := r.Calls[0]
	assert.Equal("status", rs.Name)
	assert.Nil(rs.Args)
	require.NoError(rs.Error)
	require.NotNil(rs.Response)
	st, ok := rs.Response.(*ctypes.ResultStatus)
	require.True(ok)
	assert.EqualValues("block", st.SyncInfo.LatestBlockHash)
	assert.EqualValues(10, st.SyncInfo.LatestBlockHeight)
}
