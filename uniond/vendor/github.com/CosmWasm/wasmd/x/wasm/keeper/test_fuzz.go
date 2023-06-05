package keeper

import (
	"encoding/json"

	tmBytes "github.com/cometbft/cometbft/libs/bytes"
	sdk "github.com/cosmos/cosmos-sdk/types"
	fuzz "github.com/google/gofuzz"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var ModelFuzzers = []interface{}{FuzzAddr, FuzzAddrString, FuzzAbsoluteTxPosition, FuzzContractInfo, FuzzStateModel, FuzzAccessType, FuzzAccessConfig, FuzzContractCodeHistory}

func FuzzAddr(m *sdk.AccAddress, c fuzz.Continue) {
	*m = make([]byte, 20)
	c.Read(*m)
}

func FuzzAddrString(m *string, c fuzz.Continue) {
	var x sdk.AccAddress
	FuzzAddr(&x, c)
	*m = x.String()
}

func FuzzAbsoluteTxPosition(m *types.AbsoluteTxPosition, c fuzz.Continue) {
	m.BlockHeight = c.RandUint64()
	m.TxIndex = c.RandUint64()
}

func FuzzContractInfo(m *types.ContractInfo, c fuzz.Continue) {
	m.CodeID = c.RandUint64()
	FuzzAddrString(&m.Creator, c)
	FuzzAddrString(&m.Admin, c)
	m.Label = c.RandString()
	c.Fuzz(&m.Created)
}

func FuzzContractCodeHistory(m *types.ContractCodeHistoryEntry, c fuzz.Continue) {
	const maxMsgSize = 128
	m.CodeID = c.RandUint64()
	msg := make([]byte, c.RandUint64()%maxMsgSize)
	c.Read(msg)
	var err error
	if m.Msg, err = json.Marshal(msg); err != nil {
		panic(err)
	}
	c.Fuzz(&m.Updated)
	m.Operation = types.AllCodeHistoryTypes[c.Int()%len(types.AllCodeHistoryTypes)]
}

func FuzzStateModel(m *types.Model, c fuzz.Continue) {
	m.Key = tmBytes.HexBytes(c.RandString())
	if len(m.Key) != 0 {
		c.Fuzz(&m.Value)
		return
	}
	// try again, keys must not be empty
	FuzzStateModel(m, c)
}

func FuzzAccessType(m *types.AccessType, c fuzz.Continue) {
	pos := c.Int() % len(types.AllAccessTypes)
	for _, v := range types.AllAccessTypes {
		if pos == 0 {
			*m = v
			return
		}
		pos--
	}
}

func FuzzAccessConfig(m *types.AccessConfig, c fuzz.Continue) {
	FuzzAccessType(&m.Permission, c)
	var add sdk.AccAddress
	FuzzAddr(&add, c)
	*m = m.Permission.With(add)
}
