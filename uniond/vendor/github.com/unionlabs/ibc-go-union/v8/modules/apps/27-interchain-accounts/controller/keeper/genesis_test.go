package keeper_test

import (
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestInitGenesis() {
	ports := []string{"port1", "port2", "port3"}

	testCases := []struct {
		name     string
		malleate func()
	}{
		{
			"success", func() {},
		},
		{
			"success: capabilities already initialized for first port",
			func() {
				capability := suite.chainA.GetSimApp().IBCKeeper.PortKeeper.BindPort(suite.chainA.GetContext(), ports[0])
				err := suite.chainA.GetSimApp().ICAControllerKeeper.ClaimCapability(suite.chainA.GetContext(), capability, host.PortPath(ports[0]))
				suite.Require().NoError(err)
			},
		},
	}

	interchainAccAddr := icatypes.GenerateAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, TestPortID)
	genesisState := genesistypes.ControllerGenesisState{
		ActiveChannels: []genesistypes.ActiveChannel{
			{
				ConnectionId:        ibctesting.FirstConnectionID,
				PortId:              TestPortID,
				ChannelId:           ibctesting.FirstChannelID,
				IsMiddlewareEnabled: true,
			},
			{
				ConnectionId:        "connection-1",
				PortId:              "test-port-1",
				ChannelId:           "channel-1",
				IsMiddlewareEnabled: false,
			},
		},
		InterchainAccounts: []genesistypes.RegisteredInterchainAccount{
			{
				ConnectionId:   ibctesting.FirstConnectionID,
				PortId:         TestPortID,
				AccountAddress: interchainAccAddr.String(),
			},
		},
		Ports: ports,
	}
	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			tc.malleate()

			keeper.InitGenesis(suite.chainA.GetContext(), suite.chainA.GetSimApp().ICAControllerKeeper, genesisState)

			channelID, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, TestPortID)
			suite.Require().True(found)
			suite.Require().Equal(ibctesting.FirstChannelID, channelID)

			isMiddlewareEnabled := suite.chainA.GetSimApp().ICAControllerKeeper.IsMiddlewareEnabled(suite.chainA.GetContext(), TestPortID, ibctesting.FirstConnectionID)
			suite.Require().True(isMiddlewareEnabled)

			isMiddlewareDisabled := suite.chainA.GetSimApp().ICAControllerKeeper.IsMiddlewareDisabled(suite.chainA.GetContext(), "test-port-1", "connection-1")
			suite.Require().True(isMiddlewareDisabled)

			accountAdrr, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetInterchainAccountAddress(suite.chainA.GetContext(), ibctesting.FirstConnectionID, TestPortID)
			suite.Require().True(found)
			suite.Require().Equal(interchainAccAddr.String(), accountAdrr)

			expParams := types.NewParams(false)
			params := suite.chainA.GetSimApp().ICAControllerKeeper.GetParams(suite.chainA.GetContext())
			suite.Require().Equal(expParams, params)

			for _, port := range ports {
				store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(types.StoreKey))
				suite.Require().True(store.Has(icatypes.KeyPort(port)))

				capability, found := suite.chainA.GetSimApp().ScopedICAControllerKeeper.GetCapability(suite.chainA.GetContext(), host.PortPath(port))
				suite.Require().True(found)
				suite.Require().NotNil(capability)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestExportGenesis() {
	suite.SetupTest()

	path := NewICAPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path)

	err := SetupICAPath(path, TestOwnerAddress)
	suite.Require().NoError(err)

	interchainAccAddr, exists := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
	suite.Require().True(exists)

	genesisState := keeper.ExportGenesis(suite.chainA.GetContext(), suite.chainA.GetSimApp().ICAControllerKeeper)

	suite.Require().Equal(path.EndpointA.ChannelID, genesisState.ActiveChannels[0].ChannelId)
	suite.Require().Equal(path.EndpointA.ChannelConfig.PortID, genesisState.ActiveChannels[0].PortId)
	suite.Require().True(genesisState.ActiveChannels[0].IsMiddlewareEnabled)

	suite.Require().Equal(interchainAccAddr, genesisState.InterchainAccounts[0].AccountAddress)
	suite.Require().Equal(path.EndpointA.ChannelConfig.PortID, genesisState.InterchainAccounts[0].PortId)

	suite.Require().Equal([]string{TestPortID}, genesisState.GetPorts())

	expParams := types.DefaultParams()
	suite.Require().Equal(expParams, genesisState.GetParams())
}
