package types

import (
	"encoding/hex"
	"encoding/json"
	"math"
	"strings"

	abci "github.com/cometbft/cometbft/abci/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
)

var cdc = codec.NewLegacyAmino()

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
		raw, err := cdc.MarshalJSON(logs)
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
	return &SearchTxsResult{
		TotalCount: totalCount,
		Count:      count,
		PageNumber: page,
		PageTotal:  uint64(math.Ceil(float64(totalCount) / float64(limit))),
		Limit:      limit,
		Txs:        txs,
	}
}

// ParseABCILogs attempts to parse a stringified ABCI tx log into a slice of
// ABCIMessageLog types. It returns an error upon JSON decoding failure.
func ParseABCILogs(logs string) (res ABCIMessageLogs, err error) {
	err = json.Unmarshal([]byte(logs), &res)
	return res, err
}

var _, _ codectypes.UnpackInterfacesMessage = SearchTxsResult{}, TxResponse{}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
//
// types.UnpackInterfaces needs to be called for each nested Tx because
// there are generally interfaces to unpack in Tx's
func (s SearchTxsResult) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	for _, tx := range s.Txs {
		err := codectypes.UnpackInterfaces(tx, unpacker)
		if err != nil {
			return err
		}
	}
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (r TxResponse) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	if r.Tx != nil {
		var tx Tx
		return unpacker.UnpackAny(r.Tx, &tx)
	}
	return nil
}

// GetTx unpacks the Tx from within a TxResponse and returns it
func (r TxResponse) GetTx() Tx {
	if tx, ok := r.Tx.GetCachedValue().(Tx); ok {
		return tx
	}
	return nil
}

// WrapServiceResult wraps a result from a protobuf RPC service method call (res proto.Message, err error)
// in a Result object or error. This method takes care of marshaling the res param to
// protobuf and attaching any events on the ctx.EventManager() to the Result.
func WrapServiceResult(ctx Context, res proto.Message, err error) (*Result, error) {
	if err != nil {
		return nil, err
	}

	any, err := codectypes.NewAnyWithValue(res)
	if err != nil {
		return nil, err
	}

	var data []byte
	if res != nil {
		data, err = proto.Marshal(res)
		if err != nil {
			return nil, err
		}
	}

	var events []abci.Event
	if evtMgr := ctx.EventManager(); evtMgr != nil {
		events = evtMgr.ABCIEvents()
	}

	return &Result{
		Data:         data,
		Events:       events,
		MsgResponses: []*codectypes.Any{any},
	}, nil
}
