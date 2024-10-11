package simsx

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestSimulationReporterToLegacy(t *testing.T) {
	myErr := errors.New("my-error")
	myMsg := testdata.NewTestMsg([]byte{1})

	specs := map[string]struct {
		setup  func() SimulationReporter
		expOp  simtypes.OperationMsg
		expErr error
	}{
		"init only": {
			setup:  func() SimulationReporter { return NewBasicSimulationReporter() },
			expOp:  simtypes.NewOperationMsgBasic("", "", "", false),
			expErr: errors.New("operation aborted before msg was executed"),
		},
		"success result": {
			setup: func() SimulationReporter {
				r := NewBasicSimulationReporter().WithScope(myMsg)
				r.Success(myMsg, "testing")
				return r
			},
			expOp: simtypes.NewOperationMsgBasic("TestMsg", "/testpb.TestMsg", "testing", true),
		},
		"error result": {
			setup: func() SimulationReporter {
				r := NewBasicSimulationReporter().WithScope(myMsg)
				r.Fail(myErr, "testing")
				return r
			},
			expOp:  simtypes.NewOperationMsgBasic("TestMsg", "/testpb.TestMsg", "testing", false),
			expErr: myErr,
		},
		"last error wins": {
			setup: func() SimulationReporter {
				r := NewBasicSimulationReporter().WithScope(myMsg)
				r.Fail(errors.New("other-err"), "testing1")
				r.Fail(myErr, "testing2")
				return r
			},
			expOp:  simtypes.NewOperationMsgBasic("TestMsg", "/testpb.TestMsg", "testing1, testing2", false),
			expErr: myErr,
		},
		"skipped ": {
			setup: func() SimulationReporter {
				r := NewBasicSimulationReporter().WithScope(myMsg)
				r.Skip("testing")
				return r
			},
			expOp: simtypes.NoOpMsg("TestMsg", "/testpb.TestMsg", "testing"),
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			r := spec.setup()
			assert.Equal(t, spec.expOp, r.ToLegacyOperationMsg())
			require.Equal(t, spec.expErr, r.Close())
		})
	}
}

func TestSimulationReporterTransitions(t *testing.T) {
	specs := map[string]struct {
		setup     func(r SimulationReporter)
		expStatus ReporterStatus
		expPanic  bool
	}{
		"undefined->skipped": {
			setup: func(r SimulationReporter) {
				r.Skip("testing")
			},
			expStatus: skipped,
		},
		"skipped->skipped": {
			setup: func(r SimulationReporter) {
				r.Skip("testing1")
				r.Skip("testing2")
			},
			expStatus: skipped,
		},
		"skipped->completed": {
			setup: func(r SimulationReporter) {
				r.Skip("testing1")
				r.Success(nil, "testing2")
			},
			expStatus: completed,
		},
		"completed->completed": {
			setup: func(r SimulationReporter) {
				r.Success(nil, "testing1")
				r.Fail(nil, "testing2")
			},
			expStatus: completed,
		},
		"completed->completed2": {
			setup: func(r SimulationReporter) {
				r.Fail(nil, "testing1")
				r.Success(nil, "testing2")
			},
			expStatus: completed,
		},
		"completed->skipped: rejected": {
			setup: func(r SimulationReporter) {
				r.Success(nil, "testing1")
				r.Skip("testing2")
			},
			expPanic: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			r := NewBasicSimulationReporter()
			if !spec.expPanic {
				spec.setup(r)
				assert.Equal(t, uint32(spec.expStatus), r.status.Load())
				return
			}
			require.Panics(t, func() {
				spec.setup(r)
			})
		})
	}
}

func TestSkipHook(t *testing.T) {
	myHook := func() (SkipHookFn, *bool) {
		var hookCalled bool
		return func(args ...any) {
			hookCalled = true
		}, &hookCalled
	}
	fn, myHookCalled := myHook()
	r := NewBasicSimulationReporter(fn)
	r.Skip("testing")
	require.True(t, *myHookCalled)

	// and with nested reporter
	fn, myHookCalled = myHook()
	r = NewBasicSimulationReporter(fn)
	fn2, myOtherHookCalled := myHook()
	r2 := r.WithScope(testdata.NewTestMsg([]byte{1}), fn2)
	r2.Skipf("testing %d", 2)
	assert.True(t, *myHookCalled)
	assert.True(t, *myOtherHookCalled)
}

func TestReporterSummary(t *testing.T) {
	specs := map[string]struct {
		do         func(t *testing.T, r SimulationReporter)
		expSummary map[string]int
		expReasons map[string]map[string]int
	}{
		"skipped": {
			do: func(t *testing.T, r SimulationReporter) { //nolint:thelper // not a helper
				r2 := r.WithScope(testdata.NewTestMsg([]byte{1}))
				r2.Skip("testing")
				require.NoError(t, r2.Close())
			},
			expSummary: map[string]int{"TestMsg_skipped": 1},
			expReasons: map[string]map[string]int{"/testpb.TestMsg": {"testing": 1}},
		},
		"success result": {
			do: func(t *testing.T, r SimulationReporter) { //nolint:thelper // not a helper
				msg := testdata.NewTestMsg([]byte{1})
				r2 := r.WithScope(msg)
				r2.Success(msg)
				require.NoError(t, r2.Close())
			},
			expSummary: map[string]int{"TestMsg_completed": 1},
			expReasons: map[string]map[string]int{},
		},
		"error result": {
			do: func(t *testing.T, r SimulationReporter) { //nolint:thelper // not a helper
				msg := testdata.NewTestMsg([]byte{1})
				r2 := r.WithScope(msg)
				r2.Fail(errors.New("testing"))
				require.Error(t, r2.Close())
			},
			expSummary: map[string]int{"TestMsg_completed": 1},
			expReasons: map[string]map[string]int{},
		},
		"multiple skipped": {
			do: func(t *testing.T, r SimulationReporter) { //nolint:thelper // not a helper
				r2 := r.WithScope(testdata.NewTestMsg([]byte{1}))
				r2.Skip("testing1")
				require.NoError(t, r2.Close())
				r3 := r.WithScope(testdata.NewTestMsg([]byte{2}))
				r3.Skip("testing2")
				require.NoError(t, r3.Close())
			},
			expSummary: map[string]int{"TestMsg_skipped": 2},
			expReasons: map[string]map[string]int{
				"/testpb.TestMsg": {"testing1": 1, "testing2": 1},
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			r := NewBasicSimulationReporter()
			// when
			spec.do(t, r)
			gotSummary := r.Summary()
			// then
			require.Equal(t, spec.expSummary, gotSummary.counts)
			require.Equal(t, spec.expReasons, gotSummary.skipReasons)
		})
	}
}
