package simsx

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestMsgFactories(t *testing.T) {
	myMsg := testdata.NewTestMsg()
	mySigners := []SimAccount{{}}
	specs := map[string]struct {
		src           SimMsgFactoryX
		expErrHandled bool
	}{
		"default": {
			src: SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
				return mySigners, myMsg
			}),
		},
		"with delivery result handler": {
			src: NewSimMsgFactoryWithDeliveryResultHandler[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg, handler SimDeliveryResultHandler) {
				return mySigners, myMsg, func(err error) error { return nil }
			}),
			expErrHandled: true,
		},
		"with future ops": {
			src: NewSimMsgFactoryWithFutureOps[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter, fOpsReg FutureOpsRegistry) ([]SimAccount, *testdata.TestMsg) {
				return mySigners, myMsg
			}),
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			assert.Equal(t, (*testdata.TestMsg)(nil), spec.src.MsgType())

			factoryMethod := spec.src.Create()
			require.NotNil(t, factoryMethod)
			gotSigners, gotMsg := factoryMethod(context.Background(), nil, nil)
			assert.Equal(t, mySigners, gotSigners)
			assert.Equal(t, gotMsg, myMsg)

			require.NotNil(t, spec.src.DeliveryResultHandler())
			gotErr := spec.src.DeliveryResultHandler()(errors.New("testing"))
			assert.Equal(t, spec.expErrHandled, gotErr == nil)
		})
	}
}

func TestRunWithFailFast(t *testing.T) {
	myTestMsg := testdata.NewTestMsg()
	mySigners := []SimAccount{SimAccountFixture()}
	specs := map[string]struct {
		factory    FactoryMethod
		expSigners []SimAccount
		expMsg     sdk.Msg
		expSkipped bool
	}{
		"factory completes": {
			factory: func(ctx context.Context, _ *ChainDataSource, reporter SimulationReporter) ([]SimAccount, sdk.Msg) {
				return mySigners, myTestMsg
			},
			expSigners: mySigners,
			expMsg:     myTestMsg,
		},
		"factory skips": {
			factory: func(ctx context.Context, _ *ChainDataSource, reporter SimulationReporter) ([]SimAccount, sdk.Msg) {
				reporter.Skip("testing")
				return nil, nil
			},
			expSkipped: true,
		},
		"factory skips and panics": {
			factory: func(ctx context.Context, _ *ChainDataSource, reporter SimulationReporter) ([]SimAccount, sdk.Msg) {
				reporter.Skip("testing")
				panic("should be handled")
			},
			expSkipped: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, done := context.WithCancel(context.Background())
			reporter := NewBasicSimulationReporter().WithScope(&testdata.TestMsg{}, SkipHookFn(func(...any) { done() }))
			gotSigners, gotMsg := SafeRunFactoryMethod(ctx, nil, reporter, spec.factory)
			assert.Equal(t, spec.expSigners, gotSigners)
			assert.Equal(t, spec.expMsg, gotMsg)
			assert.Equal(t, spec.expSkipped, reporter.IsSkipped())
		})
	}
}
