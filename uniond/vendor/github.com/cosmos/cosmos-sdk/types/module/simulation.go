package module

import (
	"encoding/json"
	"math/rand"
	"sort"
	"time"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

// AppModuleSimulation defines the standard functions that every module should expose
// for the SDK blockchain simulator
type AppModuleSimulation interface {
	// randomized genesis states
	GenerateGenesisState(input *SimulationState)

	// register a func to decode the each module's defined types from their corresponding store key
	RegisterStoreDecoder(simulation.StoreDecoderRegistry)

	// simulation operations (i.e msgs) with their respective weight
	WeightedOperations(simState SimulationState) []simulation.WeightedOperation
}

// HasProposalMsgs defines the messages that can be used to simulate governance (v1) proposals
type HasProposalMsgs interface {
	// msg functions used to simulate governance proposals
	ProposalMsgs(simState SimulationState) []simulation.WeightedProposalMsg
}

// HasProposalContents defines the contents that can be used to simulate legacy governance (v1beta1) proposals
type HasProposalContents interface {
	// content functions used to simulate governance proposals
	ProposalContents(simState SimulationState) []simulation.WeightedProposalContent //nolint:staticcheck // legacy v1beta1 governance
}

// SimulationManager defines a simulation manager that provides the high level utility
// for managing and executing simulation functionalities for a group of modules
type SimulationManager struct {
	Modules       []AppModuleSimulation           // array of app modules; we use an array for deterministic simulation tests
	StoreDecoders simulation.StoreDecoderRegistry // functions to decode the key-value pairs from each module's store
}

// NewSimulationManager creates a new SimulationManager object
//
// CONTRACT: All the modules provided must be also registered on the module Manager
func NewSimulationManager(modules ...AppModuleSimulation) *SimulationManager {
	return &SimulationManager{
		Modules:       modules,
		StoreDecoders: make(simulation.StoreDecoderRegistry),
	}
}

// NewSimulationManagerFromAppModules creates a new SimulationManager object.
//
// First it sets any SimulationModule provided by overrideModules, and ignores any AppModule
// with the same moduleName.
// Then it attempts to cast every provided AppModule into an AppModuleSimulation.
// If the cast succeeds, its included, otherwise it is excluded.
func NewSimulationManagerFromAppModules(modules map[string]interface{}, overrideModules map[string]AppModuleSimulation) *SimulationManager {
	simModules := []AppModuleSimulation{}
	appModuleNamesSorted := make([]string, 0, len(modules))
	for moduleName := range modules {
		appModuleNamesSorted = append(appModuleNamesSorted, moduleName)
	}

	sort.Strings(appModuleNamesSorted)

	for _, moduleName := range appModuleNamesSorted {
		// for every module, see if we override it. If so, use override.
		// Else, if we can cast the app module into a simulation module add it.
		// otherwise no simulation module.
		if simModule, ok := overrideModules[moduleName]; ok {
			simModules = append(simModules, simModule)
		} else {
			appModule := modules[moduleName]
			if simModule, ok := appModule.(AppModuleSimulation); ok {
				simModules = append(simModules, simModule)
			}
			// cannot cast, so we continue
		}
	}
	return NewSimulationManager(simModules...)
}

// Deprecated: Use GetProposalMsgs instead.
// GetProposalContents returns each module's proposal content generator function
// with their default operation weight and key.
func (sm *SimulationManager) GetProposalContents(simState SimulationState) []simulation.WeightedProposalContent {
	wContents := make([]simulation.WeightedProposalContent, 0, len(sm.Modules))
	for _, module := range sm.Modules {
		if module, ok := module.(HasProposalContents); ok {
			wContents = append(wContents, module.ProposalContents(simState)...)
		}
	}

	return wContents
}

// GetProposalMsgs returns each module's proposal msg generator function
// with their default operation weight and key.
func (sm *SimulationManager) GetProposalMsgs(simState SimulationState) []simulation.WeightedProposalMsg {
	wContents := make([]simulation.WeightedProposalMsg, 0, len(sm.Modules))
	for _, module := range sm.Modules {
		if module, ok := module.(HasProposalMsgs); ok {
			wContents = append(wContents, module.ProposalMsgs(simState)...)
		}
	}

	return wContents
}

// RegisterStoreDecoders registers each of the modules' store decoders into a map
func (sm *SimulationManager) RegisterStoreDecoders() {
	for _, module := range sm.Modules {
		module.RegisterStoreDecoder(sm.StoreDecoders)
	}
}

// GenerateGenesisStates generates a randomized GenesisState for each of the
// registered modules
func (sm *SimulationManager) GenerateGenesisStates(simState *SimulationState) {
	for _, module := range sm.Modules {
		module.GenerateGenesisState(simState)
	}
}

// WeightedOperations returns all the modules' weighted operations of an application
func (sm *SimulationManager) WeightedOperations(simState SimulationState) []simulation.WeightedOperation {
	wOps := make([]simulation.WeightedOperation, 0, len(sm.Modules))
	for _, module := range sm.Modules {
		wOps = append(wOps, module.WeightedOperations(simState)...)
	}

	return wOps
}

// SimulationState is the input parameters used on each of the module's randomized
// GenesisState generator function
type SimulationState struct {
	AppParams         simulation.AppParams
	Cdc               codec.JSONCodec                // application codec
	TxConfig          client.TxConfig                // Shared TxConfig; this is expensive to create and stateless, so create it once up front.
	Rand              *rand.Rand                     // random number
	GenState          map[string]json.RawMessage     // genesis state
	Accounts          []simulation.Account           // simulation accounts
	InitialStake      sdkmath.Int                    // initial coins per account
	NumBonded         int64                          // number of initially bonded accounts
	BondDenom         string                         // denom to be used as default
	GenTimestamp      time.Time                      // genesis timestamp
	UnbondTime        time.Duration                  // staking unbond time stored to use it as the slashing maximum evidence duration
	LegacyParamChange []simulation.LegacyParamChange // simulated parameter changes from modules
	//nolint:staticcheck //	legacy used for testing
	LegacyProposalContents []simulation.WeightedProposalContent // proposal content generator functions with their default weight and app sim key
	ProposalMsgs           []simulation.WeightedProposalMsg     // proposal msg generator functions with their default weight and app sim key
}
