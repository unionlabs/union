package simsx

import (
	"errors"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestDeliverSimsMsg(t *testing.T) {
	var (
		sender   = SimAccountFixture()
		ak       = MemoryAccountSource(sender)
		myMsg    = testdata.NewTestMsg(sender.Address)
		txConfig = txConfig()
		r        = rand.New(rand.NewSource(1))
		ctx      sdk.Context
	)
	noopResultHandler := func(err error) error { return err }
	specs := map[string]struct {
		app                      AppEntrypoint
		reporter                 func() SimulationReporter
		deliveryResultHandler    SimDeliveryResultHandler
		errDeliveryResultHandler error
		expOps                   simtypes.OperationMsg
	}{
		"error when reporter skipped": {
			app: SimDeliverFn(func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
				return sdk.GasInfo{GasWanted: 100, GasUsed: 20}, &sdk.Result{}, nil
			}),
			reporter: func() SimulationReporter {
				r := NewBasicSimulationReporter()
				r.Skip("testing")
				return r
			},
			expOps: simtypes.NoOpMsg("", "", "testing"),
		},
		"successful delivery": {
			app: SimDeliverFn(func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
				return sdk.GasInfo{GasWanted: 100, GasUsed: 20}, &sdk.Result{}, nil
			}),
			reporter:              func() SimulationReporter { return NewBasicSimulationReporter() },
			deliveryResultHandler: noopResultHandler,
			expOps:                simtypes.NewOperationMsgBasic("", "", "", true),
		},
		"error delivery": {
			app: SimDeliverFn(func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
				return sdk.GasInfo{GasWanted: 100, GasUsed: 20}, &sdk.Result{}, errors.New("my error")
			}),
			reporter:              func() SimulationReporter { return NewBasicSimulationReporter() },
			deliveryResultHandler: noopResultHandler,
			expOps:                simtypes.NewOperationMsgBasic("", "", "delivering tx with msgs: &testdata.TestMsg{Signers:[]string{\"cosmos1tnh2q55v8wyygtt9srz5safamzdengsnqeycj3\"}, DecField:0.000000000000000000}", false),
		},
		"error delivery handled": {
			app: SimDeliverFn(func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
				return sdk.GasInfo{GasWanted: 100, GasUsed: 20}, &sdk.Result{}, errors.New("my error")
			}),
			reporter:              func() SimulationReporter { return NewBasicSimulationReporter() },
			deliveryResultHandler: func(err error) error { return nil },
			expOps:                simtypes.NewOperationMsgBasic("", "", "", true),
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			got := DeliverSimsMsg(ctx, spec.reporter(), spec.app, r, txConfig, ak, "testing", myMsg, spec.deliveryResultHandler, sender)
			assert.Equal(t, spec.expOps, got)
		})
	}
}
