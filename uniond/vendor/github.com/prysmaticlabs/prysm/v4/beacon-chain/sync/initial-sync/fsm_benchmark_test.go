package initialsync

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func BenchmarkStateMachine_trigger(b *testing.B) {
	sm := newStateMachineManager()

	handlerFn := func(m *stateMachine, in interface{}) (id stateID, err error) {
		response, ok := in.(*fetchRequestParams)
		if !ok {
			return 0, errInputNotFetchRequestParams
		}
		_ = response.count
		return stateScheduled, nil
	}

	sm.addEventHandler(eventTick, stateNew, handlerFn)
	sm.addEventHandler(eventTick, stateScheduled, handlerFn)
	sm.addEventHandler(eventTick, stateDataParsed, handlerFn)
	sm.addEventHandler(eventTick, stateSkipped, handlerFn)
	sm.addEventHandler(eventTick, stateSent, handlerFn)
	sm.addStateMachine(64)

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		data := &fetchRequestParams{
			start: 23,
			count: 32,
		}
		err := sm.machines[64].trigger(eventTick, data)
		require.NoError(b, err)
	}
}
