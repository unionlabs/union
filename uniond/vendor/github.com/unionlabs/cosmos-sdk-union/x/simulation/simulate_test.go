package simulation

import (
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestRunQueuedTimeOperations(t *testing.T) {
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	ctx := sdk.Context{}
	lw := NewLogWriter(true)
	noopEvent := func(route, op, evResult string) {}
	var acc []simtypes.Account
	noOp := simtypes.FutureOperation{
		Op: func(gotR *rand.Rand, gotApp simtypes.AppEntrypoint, ctx sdk.Context, accounts []simtypes.Account, chainID string) (OperationMsg simtypes.OperationMsg, futureOps []simtypes.FutureOperation, err error) {
			return simtypes.OperationMsg{}, nil, nil
		},
	}
	futureOp := simtypes.FutureOperation{
		Op: func(gotR *rand.Rand, gotApp simtypes.AppEntrypoint, ctx sdk.Context, accounts []simtypes.Account, chainID string) (OperationMsg simtypes.OperationMsg, futureOps []simtypes.FutureOperation, err error) {
			return simtypes.OperationMsg{}, []simtypes.FutureOperation{noOp}, nil
		},
	}

	specs := map[string]struct {
		queueOps []simtypes.FutureOperation
		expOps   []simtypes.FutureOperation
	}{
		"empty": {},
		"single": {
			queueOps: []simtypes.FutureOperation{noOp},
		},
		"multi": {
			queueOps: []simtypes.FutureOperation{noOp, noOp},
		},
		"future op": {
			queueOps: []simtypes.FutureOperation{futureOp},
			expOps:   []simtypes.FutureOperation{noOp},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			expOps := len(spec.queueOps)
			n, fOps := runQueuedTimeOperations(t, &spec.queueOps, 0, time.Now(), r, nil, ctx, acc, lw, noopEvent, false, "testing")
			require.Equal(t, expOps, n)
			assert.Empty(t, spec.queueOps)
			assert.Len(t, fOps, len(spec.expOps)) // using len as equal fails with Go 1.23 now
		})
	}
}
