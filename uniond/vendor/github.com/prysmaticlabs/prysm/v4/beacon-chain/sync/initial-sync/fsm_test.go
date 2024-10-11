package initialsync

import (
	"errors"
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStateMachineManager_String(t *testing.T) {
	tests := []struct {
		name     string
		machines map[primitives.Slot]*stateMachine
		want     string
	}{
		{
			"empty epoch state list",
			map[primitives.Slot]*stateMachine{},
			"map[]",
		},
		{
			"newly created state machine",
			map[primitives.Slot]*stateMachine{
				0:   {start: 64 * 0, state: stateNew},
				64:  {start: 64 * 1, state: stateScheduled},
				128: {start: 64 * 2, state: stateDataParsed},
				196: {start: 64 * 3, state: stateSkipped},
				256: {start: 64 * 4, state: stateSent},
			},
			"map[0:{0:new} 64:{2:scheduled} 128:{4:dataParsed} 196:{6:skipped} 256:{8:sent}]",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			smm := &stateMachineManager{
				machines: tt.machines,
			}
			assert.Equal(t, tt.want, smm.String())
		})
	}
}

func TestStateMachine_StateIDString(t *testing.T) {
	stateIDs := []stateID{stateNew, stateScheduled, stateDataParsed, stateSkipped, stateSent}
	assert.Equal(t, "[new scheduled dataParsed skipped sent]", fmt.Sprintf("%v", stateIDs))
	assert.Equal(t, "stateUnknown", stateID(15).String())
}

func TestStateMachine_EventIDString(t *testing.T) {
	eventIDs := []eventID{eventTick, eventDataReceived}
	assert.Equal(t, "[tick dataReceived]", fmt.Sprintf("%v", eventIDs))
	assert.Equal(t, "eventUnknown", eventID(15).String())
}

func TestStateMachineManager_addEventHandler(t *testing.T) {
	smm := newStateMachineManager()

	smm.addEventHandler(eventTick, stateNew, func(m *stateMachine, i interface{}) (id stateID, err error) {
		return stateScheduled, nil
	})
	assert.Equal(t, 1, len(smm.handlers[stateNew]), "Unexpected size")
	state, err := smm.handlers[stateNew][eventTick](nil, nil)
	assert.NoError(t, err)
	assert.Equal(t, stateScheduled, state, "Unexpected state")

	// Add second handler to the same event
	smm.addEventHandler(eventTick, stateSent, func(m *stateMachine, i interface{}) (id stateID, err error) {
		return stateDataParsed, nil
	})
	assert.Equal(t, 1, len(smm.handlers[stateSent]), "Unexpected size")
	state, err = smm.handlers[stateSent][eventTick](nil, nil)
	assert.NoError(t, err)
	assert.Equal(t, stateDataParsed, state, "Unexpected state")

	// Add another handler to existing event/state pair. Should have no effect.
	smm.addEventHandler(eventTick, stateSent, func(m *stateMachine, i interface{}) (id stateID, err error) {
		return stateSkipped, nil
	})
	assert.Equal(t, 1, len(smm.handlers[stateSent]), "Unexpected size")
	state, err = smm.handlers[stateSent][eventTick](nil, nil)
	assert.NoError(t, err)
	// No effect, previous handler worked.
	assert.Equal(t, stateDataParsed, state, "Unexpected state")
}

func TestStateMachine_trigger(t *testing.T) {
	type event struct {
		state       stateID
		event       eventID
		returnState stateID
		err         bool
	}
	type args struct {
		name        eventID
		returnState stateID
		epoch       primitives.Epoch
		data        interface{}
	}
	tests := []struct {
		name   string
		events []event
		epochs []primitives.Epoch
		args   args
		err    error
	}{
		{
			name:   "event not found",
			events: []event{},
			epochs: []primitives.Epoch{12, 13},
			args:   args{name: eventTick, epoch: 12, data: nil, returnState: stateNew},
			err:    errors.New("no event handlers registered for event: tick, state: new"),
		},
		{
			name: "single action",
			events: []event{
				{stateNew, eventTick, stateScheduled, false},
			},
			epochs: []primitives.Epoch{12, 13},
			args:   args{name: eventTick, epoch: 12, data: nil, returnState: stateScheduled},
			err:    nil,
		},
		{
			name: "multiple actions, has error",
			events: []event{
				{stateNew, eventTick, stateScheduled, false},
				{stateScheduled, eventTick, stateSent, true},
				{stateSent, eventTick, stateSkipped, false},
			},
			epochs: []primitives.Epoch{12, 13},
			args:   args{name: eventTick, epoch: 12, data: nil, returnState: stateScheduled},
			err:    nil,
		},
		{
			name: "multiple actions, no error, can cascade",
			events: []event{
				{stateNew, eventTick, stateScheduled, false},
				{stateScheduled, eventTick, stateSent, false},
				{stateSent, eventTick, stateSkipped, false},
			},
			epochs: []primitives.Epoch{12, 13},
			args:   args{name: eventTick, epoch: 12, data: nil, returnState: stateScheduled},
			err:    nil,
		},
		{
			name: "multiple actions, no error, no cascade",
			events: []event{
				{stateNew, eventTick, stateScheduled, false},
				{stateScheduled, eventTick, stateSent, false},
				{stateNew, eventTick, stateSkipped, false},
			},
			epochs: []primitives.Epoch{12, 13},
			args:   args{name: eventTick, epoch: 12, data: nil, returnState: stateScheduled},
			err:    nil,
		},
	}
	fn := func(e event) eventHandlerFn {
		return func(m *stateMachine, in interface{}) (stateID, error) {
			if e.err {
				return m.state, errors.New("invalid")
			}
			return e.returnState, nil
		}
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			smm := newStateMachineManager()
			expectHandlerError := false
			for _, event := range tt.events {
				smm.addEventHandler(event.event, event.state, fn(event))
				if event.err {
					expectHandlerError = true
				}
			}
			for _, epoch := range tt.epochs {
				smm.addStateMachine(primitives.Slot(epoch * 32))
			}
			state := smm.machines[primitives.Slot(tt.args.epoch*32)]
			err := state.trigger(tt.args.name, tt.args.data)
			if tt.err != nil && (err == nil || tt.err.Error() != err.Error()) {
				t.Errorf("unexpected error = '%v', want '%v'", err, tt.err)
			}
			if tt.err == nil {
				if err != nil && !expectHandlerError {
					t.Error(err)
				}
				ind := primitives.Slot(tt.args.epoch * 32)
				if smm.machines[ind].state != tt.args.returnState {
					t.Errorf("unexpected final state: %v, want: %v (%v)",
						smm.machines[ind].state, tt.args.returnState, smm.machines)
				}
			}
		})
	}
}

func TestStateMachineManager_QueueLoop(t *testing.T) {
	smm := newStateMachineManager()
	smm.addEventHandler(eventTick, stateNew, func(m *stateMachine, data interface{}) (stateID, error) {
		return stateScheduled, nil
	})
	smm.addEventHandler(eventTick, stateScheduled, func(m *stateMachine, data interface{}) (stateID, error) {
		if m.start < 256 {
			return stateDataParsed, nil
		}
		return stateSkipped, nil
	})
	smm.addEventHandler(eventTick, stateDataParsed, func(m *stateMachine, data interface{}) (stateID, error) {
		return stateSent, nil
	})
	smm.addEventHandler(eventTick, stateSkipped, func(m *stateMachine, data interface{}) (stateID, error) {
		dataParsed, ok := data.(int)
		if !ok {
			return m.state, errors.New("invalid data type")
		}
		if dataParsed > 41 {
			return stateNew, nil
		}

		return stateScheduled, nil
	})
	assert.Equal(t, 4, len(smm.handlers), "Unexpected number of state events")
	smm.addStateMachine(64)
	smm.addStateMachine(512)

	assertState := func(startSlot primitives.Slot, state stateID) {
		fsm, ok := smm.findStateMachine(startSlot)
		require.Equal(t, true, ok, "State machine not found")
		assert.Equal(t, state, fsm.state, "Unexpected state machine state")
	}

	triggerTickEvent := func() {
		for _, fsm := range smm.machines {
			data := 42
			assert.NoError(t, fsm.trigger(eventTick, data))
		}
	}

	assertState(64, stateNew)
	assertState(512, stateNew)

	triggerTickEvent()
	assertState(64, stateScheduled)
	assertState(512, stateScheduled)

	triggerTickEvent()
	assertState(64, stateDataParsed)
	assertState(512, stateSkipped)

	triggerTickEvent()
	assertState(64, stateSent)
	assertState(512, stateNew)
}

func TestStateMachineManager_removeStateMachine(t *testing.T) {
	smm := newStateMachineManager()
	if _, ok := smm.findStateMachine(64); ok {
		t.Error("unexpected machine found")
	}
	smm.addStateMachine(64)
	if _, ok := smm.findStateMachine(64); !ok {
		t.Error("expected machine not found")
	}
	expectedError := fmt.Sprintf("state for machine %v is not found", 65)
	assert.ErrorContains(t, expectedError, smm.removeStateMachine(65))
	assert.NoError(t, smm.removeStateMachine(64))
	if _, ok := smm.findStateMachine(64); ok {
		t.Error("unexpected machine found")
	}
}

func TestStateMachineManager_removeAllStateMachines(t *testing.T) {
	smm := newStateMachineManager()
	smm.addStateMachine(64)
	smm.addStateMachine(128)
	smm.addStateMachine(196)
	keys := []primitives.Slot{64, 128, 196}
	assert.DeepEqual(t, smm.keys, keys, "Keys not sorted")
	assert.Equal(t, 3, len(smm.machines), "Unexpected list size")
	assert.NoError(t, smm.removeAllStateMachines())

	keys = []primitives.Slot{}
	assert.DeepEqual(t, smm.keys, keys, "Unexpected keys")
	assert.Equal(t, 0, len(smm.machines), "Expected empty list")
}

func TestStateMachineManager_findStateMachine(t *testing.T) {
	smm := newStateMachineManager()
	if _, ok := smm.findStateMachine(64); ok {
		t.Errorf("unexpected returned value: want: %v, got: %v", false, ok)
	}
	smm.addStateMachine(64)
	if fsm, ok := smm.findStateMachine(64); !ok || fsm == nil {
		t.Errorf("unexpected returned value: want: %v, got: %v", true, ok)
	}
	smm.addStateMachine(512)
	smm.addStateMachine(196)
	smm.addStateMachine(256)
	smm.addStateMachine(128)
	if fsm, ok := smm.findStateMachine(128); !ok || fsm.start != 128 {
		t.Errorf("unexpected start slot: %v, want: %v", fsm.start, 122)
	}
	if fsm, ok := smm.findStateMachine(512); !ok || fsm.start != 512 {
		t.Errorf("unexpected start slot: %v, want: %v", fsm.start, 512)
	}
	keys := []primitives.Slot{64, 128, 196, 256, 512}
	assert.DeepEqual(t, smm.keys, keys, "Keys not sorted")
}

func TestStateMachineManager_highestStartSlot(t *testing.T) {
	smm := newStateMachineManager()
	_, err := smm.highestStartSlot()
	assert.ErrorContains(t, "no state machine exist", err)
	smm.addStateMachine(64)
	smm.addStateMachine(128)
	smm.addStateMachine(196)
	start, err := smm.highestStartSlot()
	assert.NoError(t, err)
	assert.Equal(t, primitives.Slot(196), start, "Incorrect highest start slot")
	assert.NoError(t, smm.removeStateMachine(196))
	start, err = smm.highestStartSlot()
	assert.NoError(t, err)
	assert.Equal(t, primitives.Slot(128), start, "Incorrect highest start slot")
}

func TestStateMachineManager_allMachinesInState(t *testing.T) {
	tests := []struct {
		name             string
		smmGen           func() *stateMachineManager
		expectedStates   []stateID
		unexpectedStates []stateID
	}{
		{
			name:             "empty manager",
			smmGen:           newStateMachineManager,
			expectedStates:   []stateID{},
			unexpectedStates: []stateID{stateNew, stateScheduled, stateDataParsed, stateSkipped, stateSent},
		},
		{
			name: "single machine default state",
			smmGen: func() *stateMachineManager {
				smm := newStateMachineManager()
				smm.addStateMachine(64)
				return smm
			},
			expectedStates:   []stateID{stateNew},
			unexpectedStates: []stateID{stateScheduled, stateDataParsed, stateSkipped, stateSent},
		},
		{
			name: "single machine updated state",
			smmGen: func() *stateMachineManager {
				smm := newStateMachineManager()
				m1 := smm.addStateMachine(64)
				m1.setState(stateSkipped)
				return smm
			},
			expectedStates:   []stateID{stateSkipped},
			unexpectedStates: []stateID{stateNew, stateScheduled, stateDataParsed, stateSent},
		},
		{
			name: "multiple machines false",
			smmGen: func() *stateMachineManager {
				smm := newStateMachineManager()
				smm.addStateMachine(64)
				smm.addStateMachine(128)
				smm.addStateMachine(196)
				for _, fsm := range smm.machines {
					fsm.setState(stateSkipped)
				}
				smm.addStateMachine(256)
				return smm
			},
			expectedStates:   []stateID{},
			unexpectedStates: []stateID{stateNew, stateScheduled, stateDataParsed, stateSkipped, stateSent},
		},
		{
			name: "multiple machines true",
			smmGen: func() *stateMachineManager {
				smm := newStateMachineManager()
				smm.addStateMachine(64)
				smm.addStateMachine(128)
				smm.addStateMachine(196)
				for _, fsm := range smm.machines {
					fsm.setState(stateSkipped)
				}
				return smm
			},
			expectedStates:   []stateID{stateSkipped},
			unexpectedStates: []stateID{stateNew, stateScheduled, stateDataParsed, stateSent},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			smm := tt.smmGen()
			for _, state := range tt.expectedStates {
				if !smm.allMachinesInState(state) {
					t.Errorf("expected all machines be in state: %v", state)
				}
			}
			for _, state := range tt.unexpectedStates {
				if smm.allMachinesInState(state) {
					t.Errorf("unexpected state: %v", state)
				}
			}
		})
	}
}

func TestStateMachine_isFirstLast(t *testing.T) {
	checkFirst := func(m *stateMachine, want bool) {
		assert.Equal(t, want, m.isFirst(), "isFirst() returned unexpected value")
	}
	checkLast := func(m *stateMachine, want bool) {
		assert.Equal(t, want, m.isLast(), "isLast() returned unexpected value")
	}
	smm := newStateMachineManager()
	m1 := smm.addStateMachine(64)
	checkFirst(m1, true)
	checkLast(m1, true)

	m2 := smm.addStateMachine(128)
	checkFirst(m1, true)
	checkLast(m1, false)
	checkFirst(m2, false)
	checkLast(m2, true)

	m3 := smm.addStateMachine(512)
	checkFirst(m1, true)
	checkLast(m1, false)
	checkFirst(m2, false)
	checkLast(m2, false)
	checkFirst(m3, false)
	checkLast(m3, true)

	// Add machine with lower start slot - shouldn't be marked as last.
	m4 := smm.addStateMachine(196)
	checkFirst(m1, true)
	checkLast(m1, false)
	checkFirst(m2, false)
	checkLast(m2, false)
	checkFirst(m3, false)
	checkLast(m3, true)
	checkFirst(m4, false)
	checkLast(m4, false)

	// Add machine with lowest start slot - should be marked as first.
	m5 := smm.addStateMachine(32)
	checkFirst(m1, false)
	checkLast(m1, false)
	checkFirst(m2, false)
	checkLast(m2, false)
	checkFirst(m3, false)
	checkLast(m3, true)
	checkFirst(m4, false)
	checkLast(m4, false)
	checkFirst(m5, true)
	checkLast(m5, false)

	keys := []primitives.Slot{32, 64, 128, 196, 512}
	assert.DeepEqual(t, smm.keys, keys, "Keys not sorted")
}
