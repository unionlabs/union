package ibctesting

import (
	"bytes"
	"compress/gzip"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	abci "github.com/cometbft/cometbft/abci/types"
	"cosmossdk.io/math/unsafe"
	"github.com/cosmos/gogoproto/proto"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var wasmIdent = []byte("\x00\x61\x73\x6D")

// SeedNewContractInstance stores some wasm code and instantiates a new contract on this chain.
// This method can be called to prepare the store with some valid CodeInfo and ContractInfo. The returned
// Address is the contract address for this instance. Test should make use of this data and/or use NewIBCContractMockWasmEngine
// for using a contract mock in Go.
func (chain *TestChain) SeedNewContractInstance() sdk.AccAddress {
	pInstResp := chain.StoreCode(append(wasmIdent, rand.Bytes(10)...))
	codeID := pInstResp.CodeID

	anyAddressStr := chain.SenderAccount.GetAddress().String()
	initMsg := []byte(fmt.Sprintf(`{"verifier": %q, "beneficiary": %q}`, anyAddressStr, anyAddressStr))
	return chain.InstantiateContract(codeID, initMsg)
}

func (chain *TestChain) StoreCodeFile(filename string) types.MsgStoreCodeResponse {
	wasmCode, err := os.ReadFile(filename)
	require.NoError(chain.t, err)
	if strings.HasSuffix(filename, "wasm") { // compress for gas limit
		var buf bytes.Buffer
		gz := gzip.NewWriter(&buf)
		_, err := gz.Write(wasmCode)
		require.NoError(chain.t, err)
		err = gz.Close()
		require.NoError(chain.t, err)
		wasmCode = buf.Bytes()
	}
	return chain.StoreCode(wasmCode)
}

func (chain *TestChain) StoreCode(byteCode []byte) types.MsgStoreCodeResponse {
	storeMsg := &types.MsgStoreCode{
		Sender:       chain.SenderAccount.GetAddress().String(),
		WASMByteCode: byteCode,
	}
	r, err := chain.SendMsgs(storeMsg)
	require.NoError(chain.t, err)

	var pInstResp types.MsgStoreCodeResponse
	chain.UnwrapExecTXResult(r, &pInstResp)

	require.NotEmpty(chain.t, pInstResp.CodeID)
	require.NotEmpty(chain.t, pInstResp.Checksum)
	return pInstResp
}

// UnwrapExecTXResult is a helper to unpack execution result from proto any type
func (chain *TestChain) UnwrapExecTXResult(r *abci.ExecTxResult, target proto.Message) {
	var wrappedRsp sdk.TxMsgData
	require.NoError(chain.t, chain.Codec.Unmarshal(r.Data, &wrappedRsp))

	// unmarshal protobuf response from data
	require.Len(chain.t, wrappedRsp.MsgResponses, 1)
	require.NoError(chain.t, proto.Unmarshal(wrappedRsp.MsgResponses[0].Value, target))
}

func (chain *TestChain) InstantiateContract(codeID uint64, initMsg []byte) sdk.AccAddress {
	instantiateMsg := &types.MsgInstantiateContract{
		Sender: chain.SenderAccount.GetAddress().String(),
		Admin:  chain.SenderAccount.GetAddress().String(),
		CodeID: codeID,
		Label:  "ibc-test",
		Msg:    initMsg,
		Funds:  sdk.Coins{ibctesting.TestCoin},
	}

	r, err := chain.SendMsgs(instantiateMsg)
	require.NoError(chain.t, err)

	var pExecResp types.MsgInstantiateContractResponse
	chain.UnwrapExecTXResult(r, &pExecResp)

	a, err := sdk.AccAddressFromBech32(pExecResp.Address)
	require.NoError(chain.t, err)
	return a
}

func (chain *TestChain) RawQuery(contractAddr string, queryData []byte) ([]byte, error) {
	req := types.QueryRawContractStateRequest{
		Address:   contractAddr,
		QueryData: queryData,
	}
	reqBin, err := proto.Marshal(&req)
	if err != nil {
		return nil, err
	}

	res, err := chain.App.Query(context.TODO(), &abci.RequestQuery{
		Path: "/cosmwasm.wasm.v1.Query/RawContractState",
		Data: reqBin,
	})
	require.NoError(chain.t, err)

	if res.Code != 0 {
		return nil, fmt.Errorf("raw query failed: (%d) %s", res.Code, res.Log)
	}

	// unpack protobuf
	var resp types.QueryRawContractStateResponse
	err = proto.Unmarshal(res.Value, &resp)
	if err != nil {
		return nil, err
	}

	return resp.Data, nil
}

// SmartQuery This will serialize the query message and submit it to the contract.
// The response is parsed into the provided interface.
// Usage: SmartQuery(addr, QueryMsg{Foo: 1}, &response)
func (chain *TestChain) SmartQuery(contractAddr string, queryMsg, response interface{}) error {
	msg, err := json.Marshal(queryMsg)
	if err != nil {
		return err
	}

	req := types.QuerySmartContractStateRequest{
		Address:   contractAddr,
		QueryData: msg,
	}
	reqBin, err := proto.Marshal(&req)
	if err != nil {
		return err
	}

	res, err := chain.App.Query(context.TODO(), &abci.RequestQuery{
		Path: "/cosmwasm.wasm.v1.Query/SmartContractState",
		Data: reqBin,
	})
	require.NoError(chain.t, err)

	if res.Code != 0 {
		return fmt.Errorf("smart query failed: (%d) %s", res.Code, res.Log)
	}

	// unpack protobuf
	var resp types.QuerySmartContractStateResponse
	err = proto.Unmarshal(res.Value, &resp)
	if err != nil {
		return err
	}
	// unpack json content
	return json.Unmarshal(resp.Data, response)
}

// ContractInfo is a helper function to returns the ContractInfo for the given contract address
func (chain *TestChain) ContractInfo(contractAddr sdk.AccAddress) *types.ContractInfo {
	return chain.App.GetWasmKeeper().GetContractInfo(chain.GetContext(), contractAddr)
}
