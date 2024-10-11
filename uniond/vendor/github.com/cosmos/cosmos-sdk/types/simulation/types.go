package simulation

import (
	"context"
	"encoding/json"
	"math/rand"
	"time"

	"cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

// AppEntrypoint defines the method for delivering simulation TX to the app. This is implemented by *Baseapp
type AppEntrypoint interface {
	SimDeliver(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error)
}

var _ AppEntrypoint = SimDeliverFn(nil)

type (
	AppEntrypointFn = SimDeliverFn
	SimDeliverFn    func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error)
)

func (m SimDeliverFn) SimDeliver(txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
	return m(txEncoder, tx)
}

// Deprecated: Use WeightedProposalMsg instead.
type WeightedProposalContent interface {
	AppParamsKey() string                   // key used to retrieve the value of the weight from the simulation application params
	DefaultWeight() int                     // default weight
	ContentSimulatorFn() ContentSimulatorFn // content simulator function
}

// Deprecated: Use MsgSimulatorFn instead.
type ContentSimulatorFn func(r *rand.Rand, ctx context.Context, accs []Account) Content

// Deprecated: Use MsgSimulatorFn instead.
type Content interface {
	GetTitle() string
	GetDescription() string
	ProposalRoute() string
	ProposalType() string
	ValidateBasic() error
	String() string
}

type WeightedProposalMsg interface {
	AppParamsKey() string            // key used to retrieve the value of the weight from the simulation application params
	DefaultWeight() int              // default weight
	MsgSimulatorFn() MsgSimulatorFnX // msg simulator function
}

type (
	// Deprecated: use MsgSimulatorFnX
	MsgSimulatorFn  func(r *rand.Rand, accs []Account, cdc address.Codec) (sdk.Msg, error)
	MsgSimulatorFnX func(ctx context.Context, r *rand.Rand, accs []Account, cdc address.Codec) (sdk.Msg, error)
)

type SimValFn func(r *rand.Rand) string

type LegacyParamChange interface {
	Subspace() string
	Key() string
	SimValue() SimValFn
	ComposedKey() string
}

type WeightedOperation interface {
	Weight() int
	Op() Operation
}

// Operation runs a state machine transition, and ensures the transition
// happened as expected.  The operation could be running and testing a fuzzed
// transaction, or doing the same for a message.
//
// For ease of debugging, an operation returns a descriptive message "action",
// which details what this fuzzed state machine transition actually did.
//
// Operations can optionally provide a list of "FutureOperations" to run later
// These will be ran at the beginning of the corresponding block.
type Operation func(r *rand.Rand, app AppEntrypoint,
	ctx sdk.Context, accounts []Account, chainID string) (
	OperationMsg OperationMsg, futureOps []FutureOperation, err error)

// OperationMsg - structure for operation output
type OperationMsg struct {
	Route   string `json:"route" yaml:"route"`     // msg route (i.e module name)
	Name    string `json:"name" yaml:"name"`       // operation name (msg Type or "no-operation")
	Comment string `json:"comment" yaml:"comment"` // additional comment
	OK      bool   `json:"ok" yaml:"ok"`           // success
}

// NewOperationMsgBasic creates a new operation message from raw input.
func NewOperationMsgBasic(moduleName, msgType, comment string, ok bool) OperationMsg {
	return OperationMsg{
		Route:   moduleName,
		Name:    msgType,
		Comment: comment,
		OK:      ok,
	}
}

// NewOperationMsg - create a new operation message from sdk.Msg
func NewOperationMsg(msg sdk.Msg, ok bool, comment string) OperationMsg {
	msgType := sdk.MsgTypeURL(msg)
	moduleName := sdk.GetModuleNameFromTypeURL(msgType)
	if moduleName == "" {
		moduleName = msgType
	}
	return NewOperationMsgBasic(moduleName, msgType, comment, ok)
}

// NoOpMsg - create a no-operation message
func NoOpMsg(moduleName, msgType, comment string) OperationMsg {
	return NewOperationMsgBasic(moduleName, msgType, comment, false)
}

// log entry text for this operation msg
func (om OperationMsg) String() string {
	out, err := json.Marshal(om)
	if err != nil {
		panic(err)
	}

	return string(out)
}

// MustMarshal Marshals the operation msg, panic on error
func (om OperationMsg) MustMarshal() json.RawMessage {
	out, err := json.Marshal(om)
	if err != nil {
		panic(err)
	}

	return out
}

// LogEvent adds an event for the events stats
func (om OperationMsg) LogEvent(eventLogger func(route, op, evResult string)) {
	pass := "ok"
	if !om.OK {
		pass = "failure"
	}

	eventLogger(om.Route, om.Name, pass)
}

// FutureOperation is an operation which will be ran at the beginning of the
// provided BlockHeight. If both a BlockHeight and BlockTime are specified, it
// will use the BlockHeight. In the (likely) event that multiple operations
// are queued at the same block height, they will execute in a FIFO pattern.
type FutureOperation struct {
	BlockHeight int
	BlockTime   time.Time
	Op          Operation
}

// AppParams defines a flat JSON of key/values for all possible configurable
// simulation parameters. It might contain: operation weights, simulation parameters
// and flattened module state parameters (i.e not stored under it's respective module name).
type AppParams map[string]json.RawMessage

// GetOrGenerate attempts to get a given parameter by key from the AppParams
// object. If it exists, it'll be decoded and returned. Otherwise, the provided
// ParamSimulator is used to generate a random value or default value (eg: in the
// case of operation weights where Rand is not used).
func (sp AppParams) GetOrGenerate(key string, ptr interface{}, r *rand.Rand, ps ParamSimulator) {
	if v, ok := sp[key]; ok && v != nil {
		err := json.Unmarshal(v, ptr)
		if err != nil {
			panic(err)
		}
		return
	}

	ps(r)
}

type ParamSimulator func(r *rand.Rand)

type SelectOpFn func(r *rand.Rand) Operation

// AppStateFn returns the app state json bytes and the genesis accounts
type AppStateFn func(r *rand.Rand, accs []Account, config Config) (
	appState json.RawMessage, accounts []Account, chainId string, genesisTimestamp time.Time,
)

// RandomAccountFn returns a slice of n random simulation accounts
type RandomAccountFn func(r *rand.Rand, n int) []Account

type Params interface {
	PastEvidenceFraction() float64
	NumKeys() int
	EvidenceFraction() float64
	InitialLivenessWeightings() []int
	LivenessTransitionMatrix() TransitionMatrix
	BlockSizeTransitionMatrix() TransitionMatrix
}

// StoreDecoderRegistry defines each of the modules store decoders. Used for ImportExport
// simulation.
type StoreDecoderRegistry map[string]func(kvA, kvB kv.Pair) string
