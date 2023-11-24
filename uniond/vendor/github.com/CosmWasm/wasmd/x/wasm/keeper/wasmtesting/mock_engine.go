package wasmtesting

import (
	"bytes"

	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/cometbft/cometbft/libs/rand"

	errorsmod "cosmossdk.io/errors"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ types.WasmEngine = &MockWasmEngine{}

// MockWasmEngine implements types.WasmEngine for testing purpose. One or multiple messages can be stubbed.
// Without a stub function a panic is thrown.
type MockWasmEngine struct {
	StoreCodeFn          func(codeID wasmvm.WasmCode) (wasmvm.Checksum, error)
	StoreCodeUncheckedFn func(codeID wasmvm.WasmCode) (wasmvm.Checksum, error)
	AnalyzeCodeFn        func(codeID wasmvm.Checksum) (*wasmvmtypes.AnalysisReport, error)
	InstantiateFn        func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
	ExecuteFn            func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
	QueryFn              func(codeID wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) ([]byte, uint64, error)
	MigrateFn            func(codeID wasmvm.Checksum, env wasmvmtypes.Env, migrateMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
	SudoFn               func(codeID wasmvm.Checksum, env wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
	ReplyFn              func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
	GetCodeFn            func(codeID wasmvm.Checksum) (wasmvm.WasmCode, error)
	CleanupFn            func()
	IBCChannelOpenFn     func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelOpenMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error)
	IBCChannelConnectFn  func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelConnectMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCChannelCloseFn    func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelCloseMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCPacketReceiveFn   func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error)
	IBCPacketAckFn       func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketAckMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCPacketTimeoutFn   func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketTimeoutMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	PinFn                func(checksum wasmvm.Checksum) error
	UnpinFn              func(checksum wasmvm.Checksum) error
	GetMetricsFn         func() (*wasmvmtypes.Metrics, error)
}

func (m *MockWasmEngine) IBCChannelOpen(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelOpenMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error) {
	if m.IBCChannelOpenFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCChannelOpenFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) IBCChannelConnect(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelConnectMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCChannelConnectFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCChannelConnectFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) IBCChannelClose(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelCloseMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCChannelCloseFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCChannelCloseFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) IBCPacketReceive(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	if m.IBCPacketReceiveFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCPacketReceiveFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) IBCPacketAck(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketAckMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCPacketAckFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCPacketAckFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) IBCPacketTimeout(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketTimeoutMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCPacketTimeoutFn == nil {
		panic("not supposed to be called!")
	}
	return m.IBCPacketTimeoutFn(codeID, env, msg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

// Deprecated: use StoreCode instead.
func (m *MockWasmEngine) Create(codeID wasmvm.WasmCode) (wasmvm.Checksum, error) {
	panic("Deprecated: use StoreCode instead")
}

func (m *MockWasmEngine) StoreCode(codeID wasmvm.WasmCode) (wasmvm.Checksum, error) {
	if m.StoreCodeFn == nil {
		panic("not supposed to be called!")
	}
	return m.StoreCodeFn(codeID)
}

func (m *MockWasmEngine) StoreCodeUnchecked(codeID wasmvm.WasmCode) (wasmvm.Checksum, error) {
	if m.StoreCodeUncheckedFn == nil {
		panic("not supposed to be called!")
	}
	return m.StoreCodeUncheckedFn(codeID)
}

func (m *MockWasmEngine) AnalyzeCode(codeID wasmvm.Checksum) (*wasmvmtypes.AnalysisReport, error) {
	if m.AnalyzeCodeFn == nil {
		panic("not supposed to be called!")
	}
	return m.AnalyzeCodeFn(codeID)
}

func (m *MockWasmEngine) Instantiate(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	if m.InstantiateFn == nil {
		panic("not supposed to be called!")
	}
	return m.InstantiateFn(codeID, env, info, initMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) Execute(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	if m.ExecuteFn == nil {
		panic("not supposed to be called!")
	}
	return m.ExecuteFn(codeID, env, info, executeMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) Query(codeID wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) ([]byte, uint64, error) {
	if m.QueryFn == nil {
		panic("not supposed to be called!")
	}
	return m.QueryFn(codeID, env, queryMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) Migrate(codeID wasmvm.Checksum, env wasmvmtypes.Env, migrateMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	if m.MigrateFn == nil {
		panic("not supposed to be called!")
	}
	return m.MigrateFn(codeID, env, migrateMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) Sudo(codeID wasmvm.Checksum, env wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	if m.SudoFn == nil {
		panic("not supposed to be called!")
	}
	return m.SudoFn(codeID, env, sudoMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) Reply(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	if m.ReplyFn == nil {
		panic("not supposed to be called!")
	}
	return m.ReplyFn(codeID, env, reply, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m *MockWasmEngine) GetCode(codeID wasmvm.Checksum) (wasmvm.WasmCode, error) {
	if m.GetCodeFn == nil {
		panic("not supposed to be called!")
	}
	return m.GetCodeFn(codeID)
}

func (m *MockWasmEngine) Cleanup() {
	if m.CleanupFn == nil {
		panic("not supposed to be called!")
	}
	m.CleanupFn()
}

func (m *MockWasmEngine) Pin(checksum wasmvm.Checksum) error {
	if m.PinFn == nil {
		panic("not supposed to be called!")
	}
	return m.PinFn(checksum)
}

func (m *MockWasmEngine) Unpin(checksum wasmvm.Checksum) error {
	if m.UnpinFn == nil {
		panic("not supposed to be called!")
	}
	return m.UnpinFn(checksum)
}

func (m *MockWasmEngine) GetMetrics() (*wasmvmtypes.Metrics, error) {
	if m.GetMetricsFn == nil {
		panic("not expected to be called")
	}
	return m.GetMetricsFn()
}

var AlwaysPanicMockWasmEngine = &MockWasmEngine{}

// SelfCallingInstMockWasmEngine prepares a WasmEngine mock that calls itself on instantiation.
func SelfCallingInstMockWasmEngine(executeCalled *bool) *MockWasmEngine {
	return &MockWasmEngine{
		StoreCodeFn: func(code wasmvm.WasmCode) (wasmvm.Checksum, error) {
			anyCodeID := bytes.Repeat([]byte{0x1}, 32)
			return anyCodeID, nil
		},
		InstantiateFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
			return &wasmvmtypes.Response{
				Messages: []wasmvmtypes.SubMsg{
					{Msg: wasmvmtypes.CosmosMsg{
						Wasm: &wasmvmtypes.WasmMsg{Execute: &wasmvmtypes.ExecuteMsg{ContractAddr: env.Contract.Address, Msg: []byte(`{}`)}},
					}},
				},
			}, 1, nil
		},
		AnalyzeCodeFn: WithoutIBCAnalyzeFn,
		ExecuteFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
			*executeCalled = true
			return &wasmvmtypes.Response{}, 1, nil
		},
	}
}

// IBCContractCallbacks defines the methods from wasmvm to interact with the wasm contract.
// A mock contract would implement the interface to fully simulate a wasm contract's behavior.
type IBCContractCallbacks interface {
	IBCChannelOpen(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelOpenMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error)

	IBCChannelConnect(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelConnectMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	IBCChannelClose(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelCloseMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	IBCPacketReceive(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		packet wasmvmtypes.IBCPacketReceiveMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCReceiveResult, uint64, error)

	IBCPacketAck(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		ack wasmvmtypes.IBCPacketAckMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	IBCPacketTimeout(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		packet wasmvmtypes.IBCPacketTimeoutMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)
}

type contractExecutable interface {
	Execute(
		codeID wasmvm.Checksum,
		env wasmvmtypes.Env,
		info wasmvmtypes.MessageInfo,
		executeMsg []byte,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.Response, uint64, error)
}

// MakeInstantiable adds some noop functions to not fail when contract is used for instantiation
func MakeInstantiable(m *MockWasmEngine) {
	m.StoreCodeFn = HashOnlyStoreCodeFn
	m.InstantiateFn = NoOpInstantiateFn
	m.AnalyzeCodeFn = WithoutIBCAnalyzeFn
}

// MakeIBCInstantiable adds some noop functions to not fail when contract is used for instantiation
func MakeIBCInstantiable(m *MockWasmEngine) {
	MakeInstantiable(m)
	m.AnalyzeCodeFn = HasIBCAnalyzeFn
}

// NewIBCContractMockWasmEngine prepares a mocked wasm_engine for testing with an IBC contract test type.
// It is safe to use the mock with store code and instantiate functions in keeper as is also prepared
// with stubs. Execute is optional. When implemented by the Go test contract then it can be used with
// the mock.
func NewIBCContractMockWasmEngine(c IBCContractCallbacks) *MockWasmEngine {
	m := &MockWasmEngine{
		IBCChannelOpenFn:    c.IBCChannelOpen,
		IBCChannelConnectFn: c.IBCChannelConnect,
		IBCChannelCloseFn:   c.IBCChannelClose,
		IBCPacketReceiveFn:  c.IBCPacketReceive,
		IBCPacketAckFn:      c.IBCPacketAck,
		IBCPacketTimeoutFn:  c.IBCPacketTimeout,
	}
	MakeIBCInstantiable(m)
	if e, ok := c.(contractExecutable); ok { // optional function
		m.ExecuteFn = e.Execute
	}
	return m
}

func HashOnlyStoreCodeFn(code wasmvm.WasmCode) (wasmvm.Checksum, error) {
	if code == nil {
		return nil, errorsmod.Wrap(types.ErrInvalid, "wasm code must not be nil")
	}
	return wasmvm.CreateChecksum(code)
}

func NoOpInstantiateFn(wasmvm.Checksum, wasmvmtypes.Env, wasmvmtypes.MessageInfo, []byte, wasmvm.KVStore, wasmvm.GoAPI, wasmvm.Querier, wasmvm.GasMeter, uint64, wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
	return &wasmvmtypes.Response{}, 0, nil
}

func NoOpStoreCodeFn(_ wasmvm.WasmCode) (wasmvm.Checksum, error) {
	return rand.Bytes(32), nil
}

func HasIBCAnalyzeFn(wasmvm.Checksum) (*wasmvmtypes.AnalysisReport, error) {
	return &wasmvmtypes.AnalysisReport{
		HasIBCEntryPoints: true,
	}, nil
}

func WithoutIBCAnalyzeFn(wasmvm.Checksum) (*wasmvmtypes.AnalysisReport, error) {
	return &wasmvmtypes.AnalysisReport{}, nil
}

var _ IBCContractCallbacks = &MockIBCContractCallbacks{}

type MockIBCContractCallbacks struct {
	IBCChannelOpenFn    func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelOpenMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error)
	IBCChannelConnectFn func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelConnectMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCChannelCloseFn   func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCChannelCloseMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCPacketReceiveFn  func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error)
	IBCPacketAckFn      func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketAckMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
	IBCPacketTimeoutFn  func(codeID wasmvm.Checksum, env wasmvmtypes.Env, msg wasmvmtypes.IBCPacketTimeoutMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error)
}

func (m MockIBCContractCallbacks) IBCChannelOpen(codeID wasmvm.Checksum, env wasmvmtypes.Env, channel wasmvmtypes.IBCChannelOpenMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error) {
	if m.IBCChannelOpenFn == nil {
		panic("not expected to be called")
	}
	return m.IBCChannelOpenFn(codeID, env, channel, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m MockIBCContractCallbacks) IBCChannelConnect(codeID wasmvm.Checksum, env wasmvmtypes.Env, channel wasmvmtypes.IBCChannelConnectMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCChannelConnectFn == nil {
		panic("not expected to be called")
	}
	return m.IBCChannelConnectFn(codeID, env, channel, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m MockIBCContractCallbacks) IBCChannelClose(codeID wasmvm.Checksum, env wasmvmtypes.Env, channel wasmvmtypes.IBCChannelCloseMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCChannelCloseFn == nil {
		panic("not expected to be called")
	}
	return m.IBCChannelCloseFn(codeID, env, channel, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m MockIBCContractCallbacks) IBCPacketReceive(codeID wasmvm.Checksum, env wasmvmtypes.Env, packet wasmvmtypes.IBCPacketReceiveMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	if m.IBCPacketReceiveFn == nil {
		panic("not expected to be called")
	}
	return m.IBCPacketReceiveFn(codeID, env, packet, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m MockIBCContractCallbacks) IBCPacketAck(codeID wasmvm.Checksum, env wasmvmtypes.Env, ack wasmvmtypes.IBCPacketAckMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCPacketAckFn == nil {
		panic("not expected to be called")
	}
	return m.IBCPacketAckFn(codeID, env, ack, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

func (m MockIBCContractCallbacks) IBCPacketTimeout(codeID wasmvm.Checksum, env wasmvmtypes.Env, packet wasmvmtypes.IBCPacketTimeoutMsg, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResponse, uint64, error) {
	if m.IBCPacketTimeoutFn == nil {
		panic("not expected to be called")
	}
	return m.IBCPacketTimeoutFn(codeID, env, packet, store, goapi, querier, gasMeter, gasLimit, deserCost)
}
