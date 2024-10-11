package types_test

import (
	"encoding/hex"
	"encoding/json"
	"strings"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtt "github.com/cometbft/cometbft/api/cometbft/types/v1"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cmt "github.com/cometbft/cometbft/types"
	"github.com/stretchr/testify/suite"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type resultTestSuite struct {
	suite.Suite
}

func TestResultTestSuite(t *testing.T) {
	suite.Run(t, new(resultTestSuite))
}

func (s *resultTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *resultTestSuite) TestParseABCILog() {
	logs := `[{"log":"","msg_index":1,"success":true}]`
	res, err := sdk.ParseABCILogs(logs)

	s.Require().NoError(err)
	s.Require().Len(res, 1)
	s.Require().Equal(res[0].Log, "")
	s.Require().Equal(res[0].MsgIndex, uint32(1))
}

func (s *resultTestSuite) TestABCIMessageLog() {
	events := sdk.Events{
		sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo")),
		sdk.NewEvent("transfer", sdk.NewAttribute("sender", "bar")),
	}
	msgLog := sdk.NewABCIMessageLog(0, "", events)
	msgLogs := sdk.ABCIMessageLogs{msgLog}
	bz, err := json.Marshal(msgLogs)

	s.Require().NoError(err)
	s.Require().Equal(string(bz), msgLogs.String())
	s.Require().Equal(`[{"msg_index":0,"events":[{"type":"transfer","attributes":[{"key":"sender","value":"foo"}]},{"type":"transfer","attributes":[{"key":"sender","value":"bar"}]}]}]`, msgLogs.String())
}

func (s *resultTestSuite) TestNewSearchTxsResult() {
	got := sdk.NewSearchTxsResult(150, 20, 2, 20, []*sdk.TxResponse{})
	s.Require().Equal(&sdk.SearchTxsResult{
		TotalCount: 150,
		Count:      20,
		PageNumber: 2,
		PageTotal:  8,
		Limit:      20,
		Txs:        []*sdk.TxResponse{},
	}, got)
}

func (s *resultTestSuite) TestResponseResultTx() {
	deliverTxResult := abci.ExecTxResult{
		Codespace: "codespace",
		Code:      1,
		Data:      []byte("data"),
		Log:       `[]`,
		Info:      "info",
		GasWanted: 100,
		GasUsed:   90,
	}
	resultTx := &coretypes.ResultTx{
		Hash:     []byte("test"),
		Height:   10,
		TxResult: deliverTxResult,
	}
	logs, err := sdk.ParseABCILogs(`[]`)

	s.Require().NoError(err)

	want := &sdk.TxResponse{
		TxHash:    "74657374",
		Height:    10,
		Codespace: "codespace",
		Code:      1,
		Data:      strings.ToUpper(hex.EncodeToString([]byte("data"))),
		RawLog:    `[]`,
		Logs:      logs,
		Info:      "info",
		GasWanted: 100,
		GasUsed:   90,
		Tx:        nil,
		Timestamp: "timestamp",
	}

	s.Require().Equal(want, sdk.NewResponseResultTx(resultTx, nil, "timestamp"))
	s.Require().Equal((*sdk.TxResponse)(nil), sdk.NewResponseResultTx(nil, nil, "timestamp"))
	s.Require().Equal(`code: 1
codespace: codespace
data: "64617461"
events: []
gas_used: "90"
gas_wanted: "100"
height: "10"
info: info
logs: []
raw_log: '[]'
timestamp: timestamp
tx: null
txhash: "74657374"
`, sdk.NewResponseResultTx(resultTx, nil, "timestamp").String())
	s.Require().True(sdk.TxResponse{}.Empty())
	s.Require().False(want.Empty())

	resultBroadcastTx := &coretypes.ResultBroadcastTx{
		Code:      1,
		Codespace: "codespace",
		Data:      []byte("data"),
		Log:       `[]`,
		Hash:      []byte("test"),
	}

	s.Require().Equal(&sdk.TxResponse{
		Code:      1,
		Codespace: "codespace",
		Data:      "64617461",
		RawLog:    `[]`,
		Logs:      logs,
		TxHash:    "74657374",
	}, sdk.NewResponseFormatBroadcastTx(resultBroadcastTx))
	s.Require().Equal((*sdk.TxResponse)(nil), sdk.NewResponseFormatBroadcastTx(nil))
}

func (s *resultTestSuite) TestNewSearchBlocksResult() {
	got := sdk.NewSearchBlocksResult(150, 20, 2, 20, []*cmtt.Block{})
	s.Require().Equal(&sdk.SearchBlocksResult{
		TotalCount: 150,
		Count:      20,
		PageNumber: 2,
		PageTotal:  8,
		Limit:      20,
		Blocks:     []*cmtt.Block{},
	}, got)
}

func (s *resultTestSuite) TestResponseResultBlock() {
	timestamp := time.Now()
	timestampStr := timestamp.UTC().Format(time.RFC3339)

	//  create a block
	resultBlock := &coretypes.ResultBlock{Block: &cmt.Block{
		Header: cmt.Header{
			Height: 10,
			Time:   timestamp,
		},
		Evidence: cmt.EvidenceData{
			Evidence: make(cmt.EvidenceList, 0),
		},
	}}

	blk, err := resultBlock.Block.ToProto()
	s.Require().NoError(err)

	want := &cmtt.Block{
		Header:   blk.Header,
		Evidence: blk.Evidence,
	}

	s.Require().Equal(want, sdk.NewResponseResultBlock(resultBlock, timestampStr))
}
