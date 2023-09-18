package baseapp_test

import (
	"context"
	"os"
	"sync"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	testdata_pulsar "github.com/cosmos/cosmos-sdk/testutil/testdata/testpb"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestGRPCQueryRouter(t *testing.T) {
	qr := baseapp.NewGRPCQueryRouter()
	interfaceRegistry := testdata.NewTestInterfaceRegistry()
	qr.SetInterfaceRegistry(interfaceRegistry)
	testdata_pulsar.RegisterQueryServer(qr, testdata_pulsar.QueryImpl{})
	helper := &baseapp.QueryServiceTestHelper{
		GRPCQueryRouter: qr,
		Ctx:             sdk.Context{}.WithContext(context.Background()),
	}
	client := testdata.NewQueryClient(helper)

	res, err := client.Echo(context.Background(), &testdata.EchoRequest{Message: "hello"})
	require.Nil(t, err)
	require.NotNil(t, res)
	require.Equal(t, "hello", res.Message)

	res, err = client.Echo(context.Background(), nil)
	require.Nil(t, err)
	require.Empty(t, res.Message)

	res2, err := client.SayHello(context.Background(), &testdata.SayHelloRequest{Name: "Foo"})
	require.Nil(t, err)
	require.NotNil(t, res)
	require.Equal(t, "Hello Foo!", res2.Greeting)

	spot := &testdata.Dog{Name: "Spot", Size_: "big"}
	any, err := types.NewAnyWithValue(spot)
	require.NoError(t, err)
	res3, err := client.TestAny(context.Background(), &testdata.TestAnyRequest{AnyAnimal: any})
	require.NoError(t, err)
	require.NotNil(t, res3)
	require.Equal(t, spot, res3.HasAnimal.Animal.GetCachedValue())
}

func TestRegisterQueryServiceTwice(t *testing.T) {
	// Setup baseapp.
	var appBuilder *runtime.AppBuilder
	err := depinject.Inject(makeMinimalConfig(), &appBuilder)
	require.NoError(t, err)
	db := dbm.NewMemDB()
	app := appBuilder.Build(log.NewTMLogger(log.NewSyncWriter(os.Stdout)), db, nil)

	// First time registering service shouldn't panic.
	require.NotPanics(t, func() {
		testdata.RegisterQueryServer(
			app.GRPCQueryRouter(),
			testdata.QueryImpl{},
		)
	})

	// Second time should panic.
	require.Panics(t, func() {
		testdata.RegisterQueryServer(
			app.GRPCQueryRouter(),
			testdata.QueryImpl{},
		)
	})
}

// Tests that we don't have data races per
// https://github.com/cosmos/cosmos-sdk/issues/10324
// but with the same client connection being used concurrently.
func TestQueryDataRaces_sameConnectionToSameHandler(t *testing.T) {
	var mu sync.Mutex
	var helper *baseapp.QueryServiceTestHelper
	makeClientConn := func(qr *baseapp.GRPCQueryRouter) *baseapp.QueryServiceTestHelper {
		mu.Lock()
		defer mu.Unlock()

		if helper == nil {
			helper = &baseapp.QueryServiceTestHelper{
				GRPCQueryRouter: qr,
				Ctx:             sdk.Context{}.WithContext(context.Background()),
			}
		}
		return helper
	}
	testQueryDataRacesSameHandler(t, makeClientConn)
}

// Tests that we don't have data races per
// https://github.com/cosmos/cosmos-sdk/issues/10324
// but with unique client connections requesting from the same handler concurrently.
func TestQueryDataRaces_uniqueConnectionsToSameHandler(t *testing.T) {
	// Return a new handler for every single call.
	testQueryDataRacesSameHandler(t, func(qr *baseapp.GRPCQueryRouter) *baseapp.QueryServiceTestHelper {
		return &baseapp.QueryServiceTestHelper{
			GRPCQueryRouter: qr,
			Ctx:             sdk.Context{}.WithContext(context.Background()),
		}
	})
}

func testQueryDataRacesSameHandler(t *testing.T, makeClientConn func(*baseapp.GRPCQueryRouter) *baseapp.QueryServiceTestHelper) {
	t.Parallel()

	qr := baseapp.NewGRPCQueryRouter()
	interfaceRegistry := testdata.NewTestInterfaceRegistry()
	qr.SetInterfaceRegistry(interfaceRegistry)
	testdata.RegisterQueryServer(qr, testdata.QueryImpl{})

	// The goal is to invoke the router concurrently and check for any data races.
	// 0. Run with: go test -race
	// 1. Synchronize every one of the 1,000 goroutines waiting to all query at the
	//    same time.
	// 2. Once the greenlight is given, perform a query through the router.
	var wg sync.WaitGroup
	defer wg.Wait()

	greenlight := make(chan bool)
	n := 1000
	ready := make(chan bool, n)
	go func() {
		for i := 0; i < n; i++ {
			<-ready
		}
		close(greenlight)
	}()

	for i := 0; i < n; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()

			// Wait until we get the green light to start.
			ready <- true
			<-greenlight

			client := testdata.NewQueryClient(makeClientConn(qr))
			res, err := client.Echo(context.Background(), &testdata.EchoRequest{Message: "hello"})
			require.Nil(t, err)
			require.NotNil(t, res)
			require.Equal(t, "hello", res.Message)

			res, err = client.Echo(context.Background(), nil)
			require.Nil(t, err)
			require.Empty(t, res.Message)

			res2, err := client.SayHello(context.Background(), &testdata.SayHelloRequest{Name: "Foo"})
			require.Nil(t, err)
			require.NotNil(t, res)
			require.Equal(t, "Hello Foo!", res2.Greeting)

			spot := &testdata.Dog{Name: "Spot", Size_: "big"}
			any, err := types.NewAnyWithValue(spot)
			require.NoError(t, err)
			res3, err := client.TestAny(context.Background(), &testdata.TestAnyRequest{AnyAnimal: any})
			require.NoError(t, err)
			require.NotNil(t, res3)
			require.Equal(t, spot, res3.HasAnimal.Animal.GetCachedValue())
		}()
	}
}
