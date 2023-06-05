package types

import (
	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DefaultMaxQueryStackSize maximum size of the stack of contract instances doing queries
const DefaultMaxQueryStackSize uint32 = 10

// WasmerEngine defines the WASM contract runtime engine.
type WasmerEngine interface {
	// Create will compile the wasm code, and store the resulting pre-compile
	// as well as the original code. Both can be referenced later via CodeID
	// This must be done one time for given code, after which it can be
	// instatitated many times, and each instance called many times.
	//
	// For example, the code for all ERC-20 contracts should be the same.
	// This function stores the code for that contract only once, but it can
	// be instantiated with custom inputs in the future.
	Create(code wasmvm.WasmCode) (wasmvm.Checksum, error)

	// AnalyzeCode will statically analyze the code.
	// Currently just reports if it exposes all IBC entry points.
	AnalyzeCode(checksum wasmvm.Checksum) (*wasmvmtypes.AnalysisReport, error)

	// Instantiate will create a new contract based on the given codeID.
	// We can set the initMsg (contract "genesis") here, and it then receives
	// an account and address and can be invoked (Execute) many times.
	//
	// Storage should be set with a PrefixedKVStore that this code can safely access.
	//
	// Under the hood, we may recompile the wasm, use a cached native compile, or even use a cached instance
	// for performance.
	Instantiate(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		info wasmvmtypes.MessageInfo,
		initMsg []byte,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.Response, uint64, error)

	// Execute calls a given contract. Since the only difference between contracts with the same CodeID is the
	// data in their local storage, and their address in the outside world, we need no ContractID here.
	// (That is a detail for the external, sdk-facing, side).
	//
	// The caller is responsible for passing the correct `store` (which must have been initialized exactly once),
	// and setting the env with relevant info on this instance (address, balance, etc)
	Execute(
		code wasmvm.Checksum,
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

	// Query allows a client to execute a contract-specific query. If the result is not empty, it should be
	// valid json-encoded data to return to the client.
	// The meaning of path and data can be determined by the code. Path is the suffix of the abci.QueryRequest.Path
	Query(
		code wasmvm.Checksum,
		env wasmvmtypes.Env,
		queryMsg []byte,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) ([]byte, uint64, error)

	// Migrate will migrate an existing contract to a new code binary.
	// This takes storage of the data from the original contract and the CodeID of the new contract that should
	// replace it. This allows it to run a migration step if needed, or return an error if unable to migrate
	// the given data.
	//
	// MigrateMsg has some data on how to perform the migration.
	Migrate(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		migrateMsg []byte,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.Response, uint64, error)

	// Sudo runs an existing contract in read/write mode (like Execute), but is never exposed to external callers
	// (either transactions or government proposals), but can only be called by other native Go modules directly.
	//
	// This allows a contract to expose custom "super user" functions or priviledged operations that can be
	// deeply integrated with native modules.
	Sudo(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		sudoMsg []byte,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.Response, uint64, error)

	// Reply is called on the original dispatching contract after running a submessage
	Reply(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		reply wasmvmtypes.Reply,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.Response, uint64, error)

	// GetCode will load the original wasm code for the given code id.
	// This will only succeed if that code id was previously returned from
	// a call to Create.
	//
	// This can be used so that the (short) code id (hash) is stored in the iavl tree
	// and the larger binary blobs (wasm and pre-compiles) are all managed by the
	// rust library
	GetCode(code wasmvm.Checksum) (wasmvm.WasmCode, error)

	// Cleanup should be called when no longer using this to free resources on the rust-side
	Cleanup()

	// IBCChannelOpen is available on IBC-enabled contracts and is a hook to call into
	// during the handshake phase
	IBCChannelOpen(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelOpenMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBC3ChannelOpenResponse, uint64, error)

	// IBCChannelConnect is available on IBC-enabled contracts and is a hook to call into
	// during the handshake phase
	IBCChannelConnect(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelConnectMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	// IBCChannelClose is available on IBC-enabled contracts and is a hook to call into
	// at the end of the channel lifetime
	IBCChannelClose(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		channel wasmvmtypes.IBCChannelCloseMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	// IBCPacketReceive is available on IBC-enabled contracts and is called when an incoming
	// packet is received on a channel belonging to this contract
	IBCPacketReceive(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		packet wasmvmtypes.IBCPacketReceiveMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCReceiveResult, uint64, error)

	// IBCPacketAck is available on IBC-enabled contracts and is called when an
	// the response for an outgoing packet (previously sent by this contract)
	// is received
	IBCPacketAck(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		ack wasmvmtypes.IBCPacketAckMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	// IBCPacketTimeout is available on IBC-enabled contracts and is called when an
	// outgoing packet (previously sent by this contract) will probably never be executed.
	// Usually handled like ack returning an error
	IBCPacketTimeout(
		checksum wasmvm.Checksum,
		env wasmvmtypes.Env,
		packet wasmvmtypes.IBCPacketTimeoutMsg,
		store wasmvm.KVStore,
		goapi wasmvm.GoAPI,
		querier wasmvm.Querier,
		gasMeter wasmvm.GasMeter,
		gasLimit uint64,
		deserCost wasmvmtypes.UFraction,
	) (*wasmvmtypes.IBCBasicResponse, uint64, error)

	// Pin pins a code to an in-memory cache, such that is
	// always loaded quickly when executed.
	// Pin is idempotent.
	Pin(checksum wasmvm.Checksum) error

	// Unpin removes the guarantee of a contract to be pinned (see Pin).
	// After calling this, the code may or may not remain in memory depending on
	// the implementor's choice.
	// Unpin is idempotent.
	Unpin(checksum wasmvm.Checksum) error

	// GetMetrics some internal metrics for monitoring purposes.
	GetMetrics() (*wasmvmtypes.Metrics, error)
}

var _ wasmvm.KVStore = &StoreAdapter{}

// StoreAdapter adapter to bridge SDK store impl to wasmvm
type StoreAdapter struct {
	parent sdk.KVStore
}

// NewStoreAdapter constructor
func NewStoreAdapter(s sdk.KVStore) *StoreAdapter {
	if s == nil {
		panic("store must not be nil")
	}
	return &StoreAdapter{parent: s}
}

func (s StoreAdapter) Get(key []byte) []byte {
	return s.parent.Get(key)
}

func (s StoreAdapter) Set(key, value []byte) {
	s.parent.Set(key, value)
}

func (s StoreAdapter) Delete(key []byte) {
	s.parent.Delete(key)
}

func (s StoreAdapter) Iterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.Iterator(start, end)
}

func (s StoreAdapter) ReverseIterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.ReverseIterator(start, end)
}
