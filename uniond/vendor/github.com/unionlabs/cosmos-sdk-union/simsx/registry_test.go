package simsx

import (
	"context"
	"errors"
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestSimsMsgRegistryAdapter(t *testing.T) {
	senderAcc := SimAccountFixture()
	accs := []simtypes.Account{senderAcc.Account}
	ak := MockAccountSourceX{GetAccountFn: MemoryAccountSource(senderAcc).GetAccount}
	myMsg := testdata.NewTestMsg(senderAcc.Address)
	ctx := sdk.Context{}.WithContext(context.Background())
	futureTime := time.Now().Add(time.Second)

	specs := map[string]struct {
		factory           SimMsgFactoryX
		expFactoryMsg     sdk.Msg
		expFactoryErr     error
		expDeliveryErr    error
		expFutureOpsCount int
	}{
		"successful execution": {
			factory: SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
				return []SimAccount{senderAcc}, myMsg
			}),
			expFactoryMsg: myMsg,
		},
		"skip execution": {
			factory: SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
				reporter.Skip("testing")
				return nil, nil
			}),
		},
		"future ops registration": {
			factory: NewSimMsgFactoryWithFutureOps[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter, fOpsReg FutureOpsRegistry) (signer []SimAccount, msg *testdata.TestMsg) {
				fOpsReg.Add(futureTime, SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
					return []SimAccount{senderAcc}, myMsg
				}))
				return []SimAccount{senderAcc}, myMsg
			}),
			expFactoryMsg:     myMsg,
			expFutureOpsCount: 1,
		},
		"error in factory execution": {
			factory: NewSimMsgFactoryWithFutureOps[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter, fOpsReg FutureOpsRegistry) (signer []SimAccount, msg *testdata.TestMsg) {
				reporter.Fail(errors.New("testing"))
				return nil, nil
			}),
			expFactoryErr: errors.New("testing"),
		},
		"missing senders": {
			factory: SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
				return nil, myMsg
			}),
			expDeliveryErr: errors.New("no senders"),
		},
		"error in delivery execution": {
			factory: SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
				return []SimAccount{senderAcc}, myMsg
			}),
			expDeliveryErr: errors.New("testing"),
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			r := NewBasicSimulationReporter()
			reg := NewSimsMsgRegistryAdapter(r, ak, nil, txConfig(), log.NewNopLogger())
			// when
			reg.Add(100, spec.factory)
			// then
			gotOps := reg.ToLegacyObjects()
			require.Len(t, gotOps, 1)
			assert.Equal(t, 100, gotOps[0].Weight())

			// and when ops executed
			var capturedTXs []sdk.Tx
			captureTXApp := AppEntrypointFn(func(_txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
				capturedTXs = append(capturedTXs, tx)
				return sdk.GasInfo{}, &sdk.Result{}, spec.expDeliveryErr
			})
			fn := gotOps[0].Op()
			gotOpsResult, gotFOps, gotErr := fn(rand.New(rand.NewSource(1)), captureTXApp, ctx, accs, "testchain")
			// then
			if spec.expFactoryErr != nil {
				require.Equal(t, spec.expFactoryErr, gotErr)
				assert.Empty(t, gotFOps)
				assert.Equal(t, gotOpsResult.OK, spec.expFactoryErr == nil)
				assert.Empty(t, gotOpsResult.Comment)
				require.Empty(t, capturedTXs)
			}
			if spec.expDeliveryErr != nil {
				require.Equal(t, spec.expDeliveryErr, gotErr)
			}
			// and verify TX delivery
			if spec.expFactoryMsg != nil {
				require.Len(t, capturedTXs, 1)
				require.Len(t, capturedTXs[0].GetMsgs(), 1)
				assert.Equal(t, spec.expFactoryMsg, capturedTXs[0].GetMsgs()[0])
			}
			assert.Len(t, gotFOps, spec.expFutureOpsCount)
		})
	}
}

func TestUniqueTypeRegistry(t *testing.T) {
	exampleFactory := SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
		return []SimAccount{}, nil
	})

	specs := map[string]struct {
		src    []WeightedFactory
		exp    []WeightedFactoryMethod
		expErr bool
	}{
		"unique": {
			src: []WeightedFactory{{Weight: 1, Factory: exampleFactory}},
			exp: []WeightedFactoryMethod{{Weight: 1, Factory: exampleFactory.Create()}},
		},
		"duplicate": {
			src:    []WeightedFactory{{Weight: 1, Factory: exampleFactory}, {Weight: 2, Factory: exampleFactory}},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			reg := NewUniqueTypeRegistry()
			if spec.expErr {
				require.Panics(t, func() {
					for _, v := range spec.src {
						reg.Add(v.Weight, v.Factory)
					}
				})
				return
			}
			for _, v := range spec.src {
				reg.Add(v.Weight, v.Factory)
			}
			// then
			got := readAll(reg.Iterator())
			require.Len(t, got, len(spec.exp))
		})
	}
}

func TestWeightedFactories(t *testing.T) {
	r := NewWeightedFactoryMethods()
	f1 := func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg) {
		panic("not implemented")
	}
	f2 := func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg) {
		panic("not implemented")
	}
	r.Add(1, f1)
	r.Add(2, f2)
	got := readAll(r.Iterator())
	require.Len(t, got, 2)

	assert.Equal(t, uint32(1), r[0].Weight)
	assert.Equal(t, uint32(2), r[1].Weight)
}

func TestAppendIterators(t *testing.T) {
	r1 := NewWeightedFactoryMethods()
	r1.Add(2, func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg) {
		panic("not implemented")
	})
	r1.Add(2, func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg) {
		panic("not implemented")
	})
	r1.Add(3, func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg sdk.Msg) {
		panic("not implemented")
	})
	r2 := NewUniqueTypeRegistry()
	r2.Add(1, SimMsgFactoryFn[*testdata.TestMsg](func(ctx context.Context, testData *ChainDataSource, reporter SimulationReporter) (signer []SimAccount, msg *testdata.TestMsg) {
		panic("not implemented")
	}))
	// when
	all := readAll(AppendIterators(r1.Iterator(), r2.Iterator()))
	// then
	require.Len(t, all, 4)
	gotWeights := Collect(all, func(a WeightedFactoryMethod) uint32 { return a.Weight })
	assert.Equal(t, []uint32{2, 2, 3, 1}, gotWeights)
}

func readAll(iterator WeightedProposalMsgIter) []WeightedFactoryMethod {
	var ret []WeightedFactoryMethod
	for w, f := range iterator {
		ret = append(ret, WeightedFactoryMethod{Weight: w, Factory: f})
	}
	return ret
}
