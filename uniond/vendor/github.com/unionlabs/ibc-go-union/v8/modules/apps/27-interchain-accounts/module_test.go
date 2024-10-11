package ica_test

import (
	"testing"

	dbm "github.com/cosmos/cosmos-db"
	testifysuite "github.com/stretchr/testify/suite"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"

	ica "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts"
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/simapp"
)

type InterchainAccountsTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator
}

func TestICATestSuite(t *testing.T) {
	testifysuite.Run(t, new(InterchainAccountsTestSuite))
}

func (suite *InterchainAccountsTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
}

func (suite *InterchainAccountsTestSuite) TestInitModule() {
	// setup and basic testing
	chainID := "testchain"
	app := simapp.NewSimApp(log.NewNopLogger(), dbm.NewMemDB(), nil, true, simtestutil.EmptyAppOptions{}, baseapp.SetChainID(chainID))
	appModule, ok := app.ModuleManager.Modules[types.ModuleName].(ica.AppModule)
	suite.Require().True(ok)

	ctx := app.GetBaseApp().NewContext(true)

	// ensure params are not set
	suite.Require().Panics(func() {
		app.ICAControllerKeeper.GetParams(ctx)
	})
	suite.Require().Panics(func() {
		app.ICAHostKeeper.GetParams(ctx)
	})

	controllerParams := controllertypes.DefaultParams()
	controllerParams.ControllerEnabled = true

	hostParams := hosttypes.DefaultParams()
	expAllowMessages := []string{"sdk.Msg"}
	hostParams.HostEnabled = true
	hostParams.AllowMessages = expAllowMessages
	suite.Require().False(app.IBCKeeper.PortKeeper.IsBound(ctx, types.HostPortID))

	testCases := []struct {
		name              string
		malleate          func()
		expControllerPass bool
		expHostPass       bool
	}{
		{
			"both controller and host set", func() {
				var ok bool
				appModule, ok = app.ModuleManager.Modules[types.ModuleName].(ica.AppModule)
				suite.Require().True(ok)
			}, true, true,
		},
		{
			"neither controller or host is set", func() {
				appModule = ica.NewAppModule(nil, nil, nil)
			}, false, false,
		},
		{
			"only controller is set", func() {
				appModule = ica.NewAppModule(nil, &app.ICAControllerKeeper, nil)
			}, true, false,
		},
		{
			"only host is set", func() {
				appModule = ica.NewAppModule(nil, nil, &app.ICAHostKeeper)
			}, false, true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			// reset app state
			chainID := "testchain"
			app = simapp.NewSimApp(log.NewNopLogger(), dbm.NewMemDB(), nil, true, simtestutil.EmptyAppOptions{}, baseapp.SetChainID(chainID))

			ctx := app.GetBaseApp().NewContext(true)

			tc.malleate()

			suite.Require().NotPanics(func() {
				appModule.InitModule(ctx, controllerParams, hostParams)
			})

			if tc.expControllerPass {
				controllerParams = app.ICAControllerKeeper.GetParams(ctx)
				suite.Require().True(controllerParams.ControllerEnabled)
			}

			if tc.expHostPass {
				hostParams = app.ICAHostKeeper.GetParams(ctx)
				suite.Require().True(hostParams.HostEnabled)
				suite.Require().Equal(expAllowMessages, hostParams.AllowMessages)

				suite.Require().True(app.IBCKeeper.PortKeeper.IsBound(ctx, types.HostPortID))
			}
		})
	}
}
