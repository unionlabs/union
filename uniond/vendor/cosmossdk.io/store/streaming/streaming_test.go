package streaming

import (
	"context"
	"fmt"
	"os"
	"runtime"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
)

type PluginTestSuite struct {
	suite.Suite

	loggerCtx MockContext

	workDir string

	finalizeBlockReq abci.FinalizeBlockRequest
	finalizeBlockRes abci.FinalizeBlockResponse
	commitRes        abci.CommitResponse

	changeSet []*storetypes.StoreKVPair
}

func (s *PluginTestSuite) SetupTest() {
	if runtime.GOOS != "linux" {
		s.T().Skip("only run on linux")
	}

	path, err := os.Getwd()
	if err != nil {
		s.T().Fail()
	}
	s.workDir = path

	pluginVersion := "abci"
	// to write data to files, replace stdout/stdout => file/file
	pluginPath := fmt.Sprintf("%s/abci/examples/stdout/stdout", s.workDir)
	if err := os.Setenv(GetPluginEnvKey(pluginVersion), pluginPath); err != nil {
		s.T().Fail()
	}

	raw, err := NewStreamingPlugin(pluginVersion, "trace")
	require.NoError(s.T(), err, "load", "streaming", "unexpected error")

	abciListener, ok := raw.(storetypes.ABCIListener)
	require.True(s.T(), ok, "should pass type check")

	header := cmtproto.Header{Height: 1, Time: time.Now()}
	logger := log.NewNopLogger()
	streamingService := storetypes.StreamingManager{
		ABCIListeners: []storetypes.ABCIListener{abciListener},
		StopNodeOnErr: true,
	}
	s.loggerCtx = NewMockContext(header, logger, streamingService)

	// test abci message types

	s.finalizeBlockReq = abci.FinalizeBlockRequest{
		Height:            s.loggerCtx.BlockHeight(),
		Txs:               [][]byte{{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}},
		Misbehavior:       []abci.Misbehavior{},
		Hash:              []byte{1, 2, 3, 4, 5, 6, 7, 8, 9},
		DecidedLastCommit: abci.CommitInfo{},
	}
	s.finalizeBlockRes = abci.FinalizeBlockResponse{
		Events:                []abci.Event{},
		ConsensusParamUpdates: &cmtproto.ConsensusParams{},
		ValidatorUpdates:      []abci.ValidatorUpdate{},
		TxResults: []*abci.ExecTxResult{{
			Events:    []abci.Event{},
			Code:      1,
			Codespace: "mockCodeSpace",
			Data:      []byte{5, 6, 7, 8},
			GasUsed:   2,
			GasWanted: 3,
			Info:      "mockInfo",
			Log:       "mockLog",
		}},
	}
	s.commitRes = abci.CommitResponse{}

	// test store kv pair types
	for range [2000]int{} {
		s.changeSet = append(s.changeSet, &storetypes.StoreKVPair{
			StoreKey: "mockStore",
			Delete:   false,
			Key:      []byte{1, 2, 3},
			Value:    []byte{3, 2, 1},
		})
	}
}

func TestPluginTestSuite(t *testing.T) {
	suite.Run(t, new(PluginTestSuite))
}

func (s *PluginTestSuite) TestABCIGRPCPlugin() {
	s.T().Run("Should successfully load streaming", func(t *testing.T) {
		abciListeners := s.loggerCtx.StreamingManager().ABCIListeners
		for _, abciListener := range abciListeners {
			for i := range [50]int{} {

				err := abciListener.ListenFinalizeBlock(s.loggerCtx, s.finalizeBlockReq, s.finalizeBlockRes)
				assert.NoError(t, err, "ListenEndBlock")

				err = abciListener.ListenCommit(s.loggerCtx, s.commitRes, s.changeSet)
				assert.NoError(t, err, "ListenCommit")

				s.updateHeight(int64(i + 1))
			}
		}
	})
}

func (s *PluginTestSuite) updateHeight(n int64) {
	header := s.loggerCtx.BlockHeader()
	header.Height = n
	s.loggerCtx = NewMockContext(header, s.loggerCtx.Logger(), s.loggerCtx.StreamingManager())
}

var (
	_ context.Context    = MockContext{}
	_ storetypes.Context = MockContext{}
)

type MockContext struct {
	baseCtx          context.Context
	header           cmtproto.Header
	logger           log.Logger
	streamingManager storetypes.StreamingManager
}

func (m MockContext) BlockHeight() int64                            { return m.header.Height }
func (m MockContext) Logger() log.Logger                            { return m.logger }
func (m MockContext) StreamingManager() storetypes.StreamingManager { return m.streamingManager }

func (m MockContext) BlockHeader() cmtproto.Header {
	msg := proto.Clone(&m.header).(*cmtproto.Header)
	return *msg
}

func NewMockContext(header cmtproto.Header, logger log.Logger, sm storetypes.StreamingManager) MockContext {
	header.Time = header.Time.UTC()
	return MockContext{
		baseCtx:          context.Background(),
		header:           header,
		logger:           logger,
		streamingManager: sm,
	}
}

func (m MockContext) Deadline() (deadline time.Time, ok bool) {
	return m.baseCtx.Deadline()
}

func (m MockContext) Done() <-chan struct{} {
	return m.baseCtx.Done()
}

func (m MockContext) Err() error {
	return m.baseCtx.Err()
}

func (m MockContext) Value(key any) any {
	return m.baseCtx.Value(key)
}
