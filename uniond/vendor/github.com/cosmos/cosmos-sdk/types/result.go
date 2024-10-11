package types

import (
	"encoding/hex"
	"encoding/json"
	"strings"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
)

func (gi GasInfo) String() string {
	bz, _ := codec.MarshalYAML(codec.NewProtoCodec(nil), &gi)
	return string(bz)
}

func (r Result) String() string {
	bz, _ := codec.MarshalYAML(codec.NewProtoCodec(nil), &r)
	return string(bz)
}

func (r Result) GetEvents() Events {
	events := make(Events, len(r.Events))
	for i, e := range r.Events {
		events[i] = Event(e)
	}

	return events
}

// ABCIMessageLogs represents a slice of ABCIMessageLog.
type ABCIMessageLogs []ABCIMessageLog

func NewABCIMessageLog(i uint32, log string, events Events) ABCIMessageLog {
	return ABCIMessageLog{
		MsgIndex: i,
		Log:      log,
		Events:   StringifyEvents(events.ToABCIEvents()),
	}
}

// String implements the fmt.Stringer interface for the ABCIMessageLogs type.
func (logs ABCIMessageLogs) String() (str string) {
	if logs != nil {
		raw, err := json.Marshal(logs)
		if err == nil {
			str = string(raw)
		}
	}

	return str
}

// NewResponseResultTx returns a TxResponse given a ResultTx from tendermint
func NewResponseResultTx(res *coretypes.ResultTx, anyTx *codectypes.Any, timestamp string) *TxResponse {
	if res == nil {
		return nil
	}

	parsedLogs, _ := ParseABCILogs(res.TxResult.Log)

	return &TxResponse{
		TxHash:    res.Hash.String(),
		Height:    res.Height,
		Codespace: res.TxResult.Codespace,
		Code:      res.TxResult.Code,
		Data:      strings.ToUpper(hex.EncodeToString(res.TxResult.Data)),
		RawLog:    res.TxResult.Log,
		Logs:      parsedLogs,
		Info:      res.TxResult.Info,
		GasWanted: res.TxResult.GasWanted,
		GasUsed:   res.TxResult.GasUsed,
		Tx:        anyTx,
		Timestamp: timestamp,
		Events:    res.TxResult.Events,
	}
}

// NewResponseResultBlock returns a BlockResponse given a ResultBlock from CometBFT
func NewResponseResultBlock(res *coretypes.ResultBlock, timestamp string) *cmtproto.Block {
	if res == nil {
		return nil
	}

	blk, err := res.Block.ToProto()
	if err != nil {
		return nil
	}

	return &cmtproto.Block{
		Header:     blk.Header,
		Data:       blk.Data,
		Evidence:   blk.Evidence,
		LastCommit: blk.LastCommit,
	}
}

// NewResponseFormatBroadcastTx returns a TxResponse given a ResultBroadcastTx from tendermint
func NewResponseFormatBroadcastTx(res *coretypes.ResultBroadcastTx) *TxResponse {
	if res == nil {
		return nil
	}

	parsedLogs, _ := ParseABCILogs(res.Log)

	return &TxResponse{
		Code:      res.Code,
		Codespace: res.Codespace,
		Data:      res.Data.String(),
		RawLog:    res.Log,
		Logs:      parsedLogs,
		TxHash:    res.Hash.String(),
	}
}

func (r TxResponse) String() string {
	bz, _ := codec.MarshalYAML(codec.NewProtoCodec(nil), &r)
	return string(bz)
}

// Empty returns true if the response is empty
func (r TxResponse) Empty() bool {
	return r.TxHash == "" && r.Logs == nil
}

func NewSearchTxsResult(totalCount, count, page, limit uint64, txs []*TxResponse) *SearchTxsResult {
	totalPages := calcTotalPages(int64(totalCount), int64(limit))

	return &SearchTxsResult{
		TotalCount: totalCount,
		Count:      count,
		PageNumber: page,
		PageTotal:  uint64(totalPages),
		Limit:      limit,
		Txs:        txs,
	}
}

func NewSearchBlocksResult(totalCount, count, page, limit int64, blocks []*cmtproto.Block) *SearchBlocksResult {
	totalPages := calcTotalPages(totalCount, limit)

	return &SearchBlocksResult{
		TotalCount: totalCount,
		Count:      count,
		PageNumber: page,
		PageTotal:  totalPages,
		Limit:      limit,
		Blocks:     blocks,
	}
}

// ParseABCILogs attempts to parse a stringified ABCI tx log into a slice of
// ABCIMessageLog types. It returns an error upon JSON decoding failure.
func ParseABCILogs(logs string) (res ABCIMessageLogs, err error) {
	err = json.Unmarshal([]byte(logs), &res)
	return res, err
}

var _, _ gogoprotoany.UnpackInterfacesMessage = SearchTxsResult{}, TxResponse{}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
//
// types.UnpackInterfaces needs to be called for each nested Tx because
// there are generally interfaces to unpack in Tx's
func (s SearchTxsResult) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	for _, tx := range s.Txs {
		err := codectypes.UnpackInterfaces(tx, unpacker)
		if err != nil {
			return err
		}
	}
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (r TxResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	if r.Tx != nil {
		var tx HasMsgs
		return unpacker.UnpackAny(r.Tx, &tx)
	}
	return nil
}

// GetTx unpacks the Tx from within a TxResponse and returns it
func (r TxResponse) GetTx() HasMsgs {
	if tx, ok := r.Tx.GetCachedValue().(HasMsgs); ok {
		return tx
	}
	return nil
}

// calculate total pages in an overflow safe manner
func calcTotalPages(totalCount, limit int64) int64 {
	totalPages := int64(0)
	if totalCount != 0 && limit != 0 {
		if totalCount%limit > 0 {
			totalPages = totalCount/limit + 1
		} else {
			totalPages = totalCount / limit
		}
	}
	return totalPages
}
