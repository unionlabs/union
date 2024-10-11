package testing

import (
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"reflect"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

const DefaultGasUsed = uint64(1)

var (
	_ ibcwasm.WasmEngine = (*MockWasmEngine)(nil)

	// queryTypes contains all the possible query message types.
	queryTypes = [...]any{types.StatusMsg{}, types.ExportMetadataMsg{}, types.TimestampAtHeightMsg{}, types.VerifyClientMessageMsg{}, types.CheckForMisbehaviourMsg{}}

	// sudoTypes contains all the possible sudo message types.
	sudoTypes = [...]any{types.UpdateStateMsg{}, types.UpdateStateOnMisbehaviourMsg{}, types.VerifyUpgradeAndUpdateStateMsg{}, types.VerifyMembershipMsg{}, types.VerifyNonMembershipMsg{}, types.MigrateClientStoreMsg{}}
)

type (
	// queryFn is a callback function that is invoked when a specific query message type is received.
	queryFn func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error)

	// sudoFn is a callback function that is invoked when a specific sudo message type is received.
	sudoFn func(checksum wasmvm.Checksum, env wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error)
)

// MockWasmEngine implements types.WasmEngine for testing purposes. One or multiple messages can be stubbed.
// Without a stub function a panic is thrown.
// ref: https://github.com/CosmWasm/wasmd/blob/v0.42.0/x/wasm/keeper/wasmtesting/mock_engine.go#L19
type MockWasmEngine struct {
	StoreCodeFn          func(code wasmvm.WasmCode, gasLimit uint64) (wasmvmtypes.Checksum, uint64, error)
	StoreCodeUncheckedFn func(code wasmvm.WasmCode) (wasmvm.Checksum, error)
	InstantiateFn        func(checksum wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error)
	MigrateFn            func(checksum wasmvm.Checksum, env wasmvmtypes.Env, migrateMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error)
	GetCodeFn            func(checksum wasmvm.Checksum) (wasmvm.WasmCode, error)
	PinFn                func(checksum wasmvm.Checksum) error
	UnpinFn              func(checksum wasmvm.Checksum) error

	// queryCallbacks contains a mapping of queryMsg field type name to callback function.
	queryCallbacks map[string]queryFn
	sudoCallbacks  map[string]sudoFn

	// contracts contains a mapping of checksum to code.
	storedContracts map[uint32][]byte
}

// NewMockWasmEngine creates and returns a new instance of the mock wasmvm for testing purposes.
// Each callback method of the mock wasmvm can be overridden to assign specific functionality.
// Default functionality is assigned for StoreCode, StoreCodeUnchecked and GetCode. Both Pin and Unpin are implemented as no-op methods.
// All other callbacks stored in the query and sudo callback maps panic. Use RegisterQueryCallback and RegisterSudoCallback methods
// to assign expected behaviour for test cases.
func NewMockWasmEngine() *MockWasmEngine {
	m := &MockWasmEngine{
		queryCallbacks:  map[string]queryFn{},
		sudoCallbacks:   map[string]sudoFn{},
		storedContracts: map[uint32][]byte{},
	}

	for _, msgType := range queryTypes {
		typeName := reflect.TypeOf(msgType).Name()
		m.queryCallbacks[typeName] = func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
			panic(fmt.Errorf("no callback specified for type %s", typeName))
		}
	}

	for _, msgType := range sudoTypes {
		typeName := reflect.TypeOf(msgType).Name()
		m.sudoCallbacks[typeName] = func(checksum wasmvm.Checksum, env wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
			panic(fmt.Errorf("no callback specified for type %s", typeName))
		}
	}

	// Set up default behavior for Store/Pin/Get
	m.StoreCodeFn = func(code wasmvm.WasmCode, gasLimit uint64) (wasmvmtypes.Checksum, uint64, error) {
		checkSum, _ := types.CreateChecksum(code)

		m.storedContracts[binary.LittleEndian.Uint32(checkSum)] = code
		return checkSum, 0, nil
	}

	m.StoreCodeUncheckedFn = func(code wasmvm.WasmCode) (wasmvm.Checksum, error) {
		checkSum, _ := types.CreateChecksum(code)

		m.storedContracts[binary.LittleEndian.Uint32(checkSum)] = code
		return checkSum, nil
	}

	m.PinFn = func(checksum wasmvm.Checksum) error {
		return nil
	}

	m.UnpinFn = func(checksum wasmvm.Checksum) error {
		return nil
	}

	m.GetCodeFn = func(checksum wasmvm.Checksum) (wasmvm.WasmCode, error) {
		code, ok := m.storedContracts[binary.LittleEndian.Uint32(checksum)]
		if !ok {
			return nil, errors.New("code not found")
		}
		return code, nil
	}

	return m
}

// RegisterQueryCallback registers a callback for a specific message type.
func (m *MockWasmEngine) RegisterQueryCallback(queryMessage any, fn queryFn) {
	typeName := reflect.TypeOf(queryMessage).Name()
	if _, found := m.queryCallbacks[typeName]; !found {
		panic(fmt.Errorf("unexpected argument of type %s passed", typeName))
	}
	m.queryCallbacks[typeName] = fn
}

// RegisterSudoCallback registers a callback for a specific sudo message type.
func (m *MockWasmEngine) RegisterSudoCallback(sudoMessage any, fn sudoFn) {
	typeName := reflect.TypeOf(sudoMessage).Name()
	if _, found := m.sudoCallbacks[typeName]; !found {
		panic(fmt.Errorf("unexpected argument of type %s passed", typeName))
	}
	m.sudoCallbacks[typeName] = fn
}

// StoreCode implements the WasmEngine interface.
func (m *MockWasmEngine) StoreCode(code wasmvm.WasmCode, gasLimit uint64) (wasmvmtypes.Checksum, uint64, error) {
	if m.StoreCodeFn == nil {
		panic(errors.New("mock engine is not properly initialized: StoreCodeFn is nil"))
	}
	return m.StoreCodeFn(code, gasLimit)
}

// StoreCode implements the WasmEngine interface.
func (m *MockWasmEngine) StoreCodeUnchecked(code wasmvm.WasmCode) (wasmvm.Checksum, error) {
	if m.StoreCodeUncheckedFn == nil {
		panic(errors.New("mock engine is not properly initialized: StoreCodeUncheckedFn is nil"))
	}
	return m.StoreCodeUncheckedFn(code)
}

// Instantiate implements the WasmEngine interface.
func (m *MockWasmEngine) Instantiate(checksum wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	if m.InstantiateFn == nil {
		panic(errors.New("mock engine is not properly initialized: InstantiateFn is nil"))
	}
	return m.InstantiateFn(checksum, env, info, initMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

// Query implements the WasmEngine interface.
func (m *MockWasmEngine) Query(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
	msgTypeName := getQueryMsgPayloadTypeName(queryMsg)

	callbackFn, ok := m.queryCallbacks[msgTypeName]
	if !ok {
		panic(fmt.Errorf("mock engine is not properly initialized: no callback specified for %s", msgTypeName))
	}

	return callbackFn(checksum, env, queryMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

// Migrate implements the WasmEngine interface.
func (m *MockWasmEngine) Migrate(checksum wasmvm.Checksum, env wasmvmtypes.Env, migrateMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	if m.MigrateFn == nil {
		panic(errors.New("mock engine is not properly initialized: MigrateFn is nil"))
	}
	return m.MigrateFn(checksum, env, migrateMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

// Sudo implements the WasmEngine interface.
func (m *MockWasmEngine) Sudo(checksum wasmvm.Checksum, env wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	msgTypeName := getSudoMsgPayloadTypeName(sudoMsg)

	sudoFn, ok := m.sudoCallbacks[msgTypeName]
	if !ok {
		panic(fmt.Errorf("mock engine is not properly initialized: no callback specified for %s", msgTypeName))
	}

	return sudoFn(checksum, env, sudoMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
}

// GetCode implements the WasmEngine interface.
func (m *MockWasmEngine) GetCode(checksum wasmvm.Checksum) (wasmvm.WasmCode, error) {
	if m.GetCodeFn == nil {
		panic(errors.New("mock engine is not properly initialized: GetCodeFn is nil"))
	}
	return m.GetCodeFn(checksum)
}

// Pin implements the WasmEngine interface.
func (m *MockWasmEngine) Pin(checksum wasmvm.Checksum) error {
	if m.PinFn == nil {
		panic(errors.New("mock engine is not properly initialized: PinFn is nil"))
	}
	return m.PinFn(checksum)
}

// Unpin implements the WasmEngine interface.
func (m *MockWasmEngine) Unpin(checksum wasmvm.Checksum) error {
	if m.UnpinFn == nil {
		panic(errors.New("mock engine is not properly initialized: UnpinFn is nil"))
	}
	return m.UnpinFn(checksum)
}

// getQueryMsgPayloadTypeName extracts the name of the struct that is populated.
// this value is used as a key to map to a callback function to handle that message type.
func getQueryMsgPayloadTypeName(queryMsgBz []byte) string {
	payload := types.QueryMsg{}
	if err := json.Unmarshal(queryMsgBz, &payload); err != nil {
		panic(err)
	}

	var payloadField any
	if payload.Status != nil {
		payloadField = *payload.Status
	}

	if payload.CheckForMisbehaviour != nil {
		payloadField = *payload.CheckForMisbehaviour
	}

	if payload.ExportMetadata != nil {
		payloadField = *payload.ExportMetadata
	}

	if payload.TimestampAtHeight != nil {
		payloadField = *payload.TimestampAtHeight
	}

	if payload.VerifyClientMessage != nil {
		payloadField = *payload.VerifyClientMessage
	}

	if payloadField == nil {
		panic(fmt.Errorf("failed to extract valid query message from bytes: %s", string(queryMsgBz)))
	}

	return reflect.TypeOf(payloadField).Name()
}

// getSudoMsgPayloadTypeName extracts the name of the struct that is populated.
// this value is used as a key to map to a callback function to handle that message type.
func getSudoMsgPayloadTypeName(sudoMsgBz []byte) string {
	payload := types.SudoMsg{}
	if err := json.Unmarshal(sudoMsgBz, &payload); err != nil {
		panic(err)
	}

	var payloadField any
	if payload.UpdateState != nil {
		payloadField = *payload.UpdateState
	}

	if payload.UpdateStateOnMisbehaviour != nil {
		payloadField = *payload.UpdateStateOnMisbehaviour
	}

	if payload.VerifyUpgradeAndUpdateState != nil {
		payloadField = *payload.VerifyUpgradeAndUpdateState
	}

	if payload.VerifyMembership != nil {
		payloadField = *payload.VerifyMembership
	}

	if payload.VerifyNonMembership != nil {
		payloadField = *payload.VerifyNonMembership
	}

	if payload.MigrateClientStore != nil {
		payloadField = *payload.MigrateClientStore
	}

	if payloadField == nil {
		panic(fmt.Errorf("failed to extract valid sudo message from bytes: %s", string(sudoMsgBz)))
	}

	return reflect.TypeOf(payloadField).Name()
}
