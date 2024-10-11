// Package types includes important structs used by end to end tests, such
// as a configuration type, an evaluator type, and more.
package types

import (
	"context"
	"os"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"google.golang.org/grpc"
)

type E2EConfigOpt func(*E2EConfig)

func WithEpochs(e uint64) E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.EpochsToRun = e
	}
}

func WithRemoteSigner() E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.UseWeb3RemoteSigner = true
	}
}

func WithCheckpointSync() E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.TestCheckpointSync = true
	}
}

func WithValidatorCrossClient() E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.UseValidatorCrossClient = true
	}
}

func WithValidatorRESTApi() E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.UseBeaconRestApi = true
	}
}

func WithBuilder() E2EConfigOpt {
	return func(cfg *E2EConfig) {
		cfg.UseBuilder = true
	}
}

// E2EConfig defines the struct for all configurations needed for E2E testing.
type E2EConfig struct {
	TestCheckpointSync      bool
	TestSync                bool
	TestFeature             bool
	UsePrysmShValidator     bool
	UsePprof                bool
	UseWeb3RemoteSigner     bool
	TestDeposits            bool
	UseFixedPeerIDs         bool
	UseValidatorCrossClient bool
	UseBeaconRestApi        bool
	UseBuilder              bool
	EpochsToRun             uint64
	Seed                    int64
	TracingSinkEndpoint     string
	Evaluators              []Evaluator
	EvalInterceptor         func(*EvaluationContext, uint64, []*grpc.ClientConn) bool
	BeaconFlags             []string
	ValidatorFlags          []string
	PeerIDs                 []string
	ExtraEpochs             uint64
}

func GenesisFork() int {
	cfg := params.BeaconConfig()
	if cfg.CapellaForkEpoch == 0 {
		return version.Capella
	}
	if cfg.BellatrixForkEpoch == 0 {
		return version.Bellatrix
	}
	if cfg.AltairForkEpoch == 0 {
		return version.Altair
	}
	return version.Phase0
}

// Evaluator defines the structure of the evaluators used to
// conduct the current beacon state during the E2E.
type Evaluator struct {
	Name   string
	Policy func(currentEpoch primitives.Epoch) bool
	// Evaluation accepts one or many/all conns, depending on what is needed by the set of evaluators.
	Evaluation func(ec *EvaluationContext, conn ...*grpc.ClientConn) error
}

// DepositBatch represents a group of deposits that are sent together during an e2e run.
type DepositBatch int

const (
	// reserved zero value
	_ DepositBatch = iota
	// GenesisDepositBatch deposits are sent to populate the initial set of validators for genesis.
	GenesisDepositBatch
	// PostGenesisDepositBatch deposits are sent to test that deposits appear in blocks as expected
	// and validators become active.
	PostGenesisDepositBatch
)

// DepositBalancer represents a type that can sum, by validator, all deposits made in E2E prior to the function call.
type DepositBalancer interface {
	Balances(DepositBatch) map[[48]byte]uint64
}

// EvaluationContext allows for additional data to be provided to evaluators that need extra state.
type EvaluationContext struct {
	DepositBalancer
	ExitedVals           map[[48]byte]bool
	SeenVotes            map[primitives.Slot][]byte
	ExpectedEth1DataVote []byte
}

// NewEvaluationContext handles initializing internal datastructures (like maps) provided by the EvaluationContext.
func NewEvaluationContext(d DepositBalancer) *EvaluationContext {
	return &EvaluationContext{
		DepositBalancer: d,
		ExitedVals:      make(map[[48]byte]bool),
		SeenVotes:       make(map[primitives.Slot][]byte),
	}
}

// ComponentRunner defines an interface via which E2E component's configuration, execution and termination is managed.
type ComponentRunner interface {
	// Start starts a component.
	Start(ctx context.Context) error
	// Started checks whether an underlying component is started and ready to be queried.
	Started() <-chan struct{}
	// Pause pauses a component.
	Pause() error
	// Resume resumes a component.
	Resume() error
	// Stop stops a component.
	Stop() error
	// UnderlyingProcess is the underlying process, once started.
	UnderlyingProcess() *os.Process
}

type MultipleComponentRunners interface {
	ComponentRunner
	// ComponentAtIndex returns the component at index
	ComponentAtIndex(i int) (ComponentRunner, error)
	// PauseAtIndex pauses the grouped component element at the desired index.
	PauseAtIndex(i int) error
	// ResumeAtIndex resumes the grouped component element at the desired index.
	ResumeAtIndex(i int) error
	// StopAtIndex stops the grouped component element at the desired index.
	StopAtIndex(i int) error
}

type EngineProxy interface {
	ComponentRunner
	// AddRequestInterceptor adds in a json-rpc request interceptor.
	AddRequestInterceptor(rpcMethodName string, responseGen func() interface{}, trigger func() bool)
	// RemoveRequestInterceptor removes the request interceptor for the provided method.
	RemoveRequestInterceptor(rpcMethodName string)
	// ReleaseBackedUpRequests releases backed up http requests.
	ReleaseBackedUpRequests(rpcMethodName string)
}

// BeaconNodeSet defines an interface for an object that fulfills the duties
// of a group of beacon nodes.
type BeaconNodeSet interface {
	ComponentRunner
	// SetENR provides the relevant bootnode's enr to the beacon nodes.
	SetENR(enr string)
}
