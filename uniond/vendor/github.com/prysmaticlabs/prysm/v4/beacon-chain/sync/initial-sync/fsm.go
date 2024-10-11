package initialsync

import (
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

const (
	stateNew stateID = iota
	stateScheduled
	stateDataParsed
	stateSkipped
	stateSent
)

const (
	eventTick eventID = iota
	eventDataReceived
)

// stateID is unique handle for a state.
type stateID uint8

// eventID is unique handle for an event.
type eventID uint8

// stateMachineManager is a collection of managed FSMs.
type stateMachineManager struct {
	keys     []primitives.Slot
	machines map[primitives.Slot]*stateMachine
	handlers map[stateID]map[eventID]eventHandlerFn
}

// stateMachine holds a state of a single block processing FSM.
// Each FSM allows deterministic state transitions: State(S) x Event(E) -> Actions (A), State(S').
type stateMachine struct {
	smm     *stateMachineManager
	start   primitives.Slot
	state   stateID
	pid     peer.ID
	bwb     []blocks.BlockWithROBlobs
	updated time.Time
}

// eventHandlerFn is an event handler function's signature.
type eventHandlerFn func(m *stateMachine, data interface{}) (newState stateID, err error)

// newStateMachineManager returns fully initialized state machine manager.
func newStateMachineManager() *stateMachineManager {
	return &stateMachineManager{
		keys:     make([]primitives.Slot, 0, lookaheadSteps),
		machines: make(map[primitives.Slot]*stateMachine, lookaheadSteps),
		handlers: make(map[stateID]map[eventID]eventHandlerFn),
	}
}

// addHandler attaches an event handler to a state event.
func (smm *stateMachineManager) addEventHandler(event eventID, state stateID, fn eventHandlerFn) {
	if _, ok := smm.handlers[state]; !ok {
		smm.handlers[state] = make(map[eventID]eventHandlerFn)
	}
	if _, ok := smm.handlers[state][event]; !ok {
		smm.handlers[state][event] = fn
	}
}

// addStateMachine allocates memory for new FSM.
func (smm *stateMachineManager) addStateMachine(startSlot primitives.Slot) *stateMachine {
	smm.machines[startSlot] = &stateMachine{
		smm:     smm,
		start:   startSlot,
		state:   stateNew,
		bwb:     []blocks.BlockWithROBlobs{},
		updated: prysmTime.Now(),
	}
	smm.recalculateMachineAttribs()
	return smm.machines[startSlot]
}

// removeStateMachine frees memory of a processed/finished FSM.
func (smm *stateMachineManager) removeStateMachine(startSlot primitives.Slot) error {
	if _, ok := smm.machines[startSlot]; !ok {
		return fmt.Errorf("state for machine %v is not found", startSlot)
	}
	smm.machines[startSlot].bwb = nil
	delete(smm.machines, startSlot)
	smm.recalculateMachineAttribs()
	return nil
}

// removeAllStateMachines removes all managed machines.
func (smm *stateMachineManager) removeAllStateMachines() error {
	for _, key := range smm.keys {
		if err := smm.removeStateMachine(key); err != nil {
			return err
		}
	}
	smm.recalculateMachineAttribs()
	return nil
}

// recalculateMachineAttribs updates cached attributes, which are used for efficiency.
func (smm *stateMachineManager) recalculateMachineAttribs() {
	keys := make([]primitives.Slot, 0, lookaheadSteps)
	for key := range smm.machines {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool {
		return keys[i] < keys[j]
	})
	smm.keys = keys
}

// findStateMachine returns a state machine for a given start slot (if exists).
func (smm *stateMachineManager) findStateMachine(startSlot primitives.Slot) (*stateMachine, bool) {
	fsm, ok := smm.machines[startSlot]
	return fsm, ok
}

// highestStartSlot returns the start slot for the latest known state machine.
func (smm *stateMachineManager) highestStartSlot() (primitives.Slot, error) {
	if len(smm.keys) == 0 {
		return 0, errors.New("no state machine exist")
	}
	key := smm.keys[len(smm.keys)-1]
	return smm.machines[key].start, nil
}

// allMachinesInState checks whether all registered state machines are in the same state.
func (smm *stateMachineManager) allMachinesInState(state stateID) bool {
	if len(smm.machines) == 0 {
		return false
	}
	for _, fsm := range smm.machines {
		if fsm.state != state {
			return false
		}
	}
	return true
}

// String returns human readable representation of a FSM collection.
func (smm *stateMachineManager) String() string {
	return fmt.Sprintf("%v", smm.machines)
}

// setState updates the current state of a given state machine.
func (m *stateMachine) setState(name stateID) {
	if m.state == name {
		return
	}
	m.state = name
	m.updated = prysmTime.Now()
}

// trigger invokes the event handler on a given state machine.
func (m *stateMachine) trigger(event eventID, data interface{}) error {
	handlers, ok := m.smm.handlers[m.state]
	if !ok {
		return fmt.Errorf("no event handlers registered for event: %v, state: %v", event, m.state)
	}
	if handlerFn, ok := handlers[event]; ok {
		state, err := handlerFn(m, data)
		if err != nil {
			return err
		}
		m.setState(state)
	}
	return nil
}

// isFirst checks whether a given machine has the lowest start slot.
func (m *stateMachine) isFirst() bool {
	return m.start == m.smm.keys[0]
}

// isLast checks whether a given machine has the highest start slot.
func (m *stateMachine) isLast() bool {
	return m.start == m.smm.keys[len(m.smm.keys)-1]
}

// String returns human-readable representation of a FSM state.
func (m *stateMachine) String() string {
	return fmt.Sprintf("{%d:%s}", slots.ToEpoch(m.start), m.state)
}

// String returns human-readable representation of a state.
func (s stateID) String() string {
	states := map[stateID]string{
		stateNew:        "new",
		stateScheduled:  "scheduled",
		stateDataParsed: "dataParsed",
		stateSkipped:    "skipped",
		stateSent:       "sent",
	}
	if _, ok := states[s]; !ok {
		return "stateUnknown"
	}
	return states[s]
}

// String returns human-readable representation of an event.
func (e eventID) String() string {
	events := map[eventID]string{
		eventTick:         "tick",
		eventDataReceived: "dataReceived",
	}
	if _, ok := events[e]; !ok {
		return "eventUnknown"
	}
	return events[e]
}
