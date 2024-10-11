package keeper_test

import (
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestRegisterInterchainAccount() {
	var (
		owner string
		path  *ibctesting.Path
		err   error
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
		{
			"port is already bound for owner but capability is claimed by another module",
			func() {
				capability := suite.chainA.GetSimApp().IBCKeeper.PortKeeper.BindPort(suite.chainA.GetContext(), TestPortID)
				err := suite.chainA.GetSimApp().TransferKeeper.ClaimCapability(suite.chainA.GetContext(), capability, host.PortPath(TestPortID))
				suite.Require().NoError(err)
			},
			false,
		},
		{
			"fails to generate port-id",
			func() {
				owner = ""
			},
			false,
		},
		{
			"MsgChanOpenInit fails - channel is already active & in state OPEN",
			func() {
				portID, err := icatypes.NewControllerPortID(TestOwnerAddress)
				suite.Require().NoError(err)

				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, portID, path.EndpointA.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.OPEN,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        TestVersion,
				}
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetChannel(suite.chainA.GetContext(), portID, path.EndpointA.ChannelID, channel)
			},
			false,
		},
	}
	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			owner = TestOwnerAddress // must be explicitly changed

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			tc.malleate() // malleate mutates test data

			err = suite.chainA.GetSimApp().ICAControllerKeeper.RegisterInterchainAccount(suite.chainA.GetContext(), path.EndpointA.ConnectionID, owner, TestVersion)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestRegisterSameOwnerMultipleConnections() {
	suite.SetupTest()

	owner := TestOwnerAddress

	pathAToB := NewICAPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(pathAToB)

	pathAToC := NewICAPath(suite.chainA, suite.chainC)
	suite.coordinator.SetupConnections(pathAToC)

	// build ICS27 metadata with connection identifiers for path A->B
	metadata := &icatypes.Metadata{
		Version:                icatypes.Version,
		ControllerConnectionId: pathAToB.EndpointA.ConnectionID,
		HostConnectionId:       pathAToB.EndpointB.ConnectionID,
		Encoding:               icatypes.EncodingProtobuf,
		TxType:                 icatypes.TxTypeSDKMultiMsg,
	}

	err := suite.chainA.GetSimApp().ICAControllerKeeper.RegisterInterchainAccount(suite.chainA.GetContext(), pathAToB.EndpointA.ConnectionID, owner, string(icatypes.ModuleCdc.MustMarshalJSON(metadata)))
	suite.Require().NoError(err)

	// build ICS27 metadata with connection identifiers for path A->C
	metadata = &icatypes.Metadata{
		Version:                icatypes.Version,
		ControllerConnectionId: pathAToC.EndpointA.ConnectionID,
		HostConnectionId:       pathAToC.EndpointB.ConnectionID,
		Encoding:               icatypes.EncodingProtobuf,
		TxType:                 icatypes.TxTypeSDKMultiMsg,
	}

	err = suite.chainA.GetSimApp().ICAControllerKeeper.RegisterInterchainAccount(suite.chainA.GetContext(), pathAToC.EndpointA.ConnectionID, owner, string(icatypes.ModuleCdc.MustMarshalJSON(metadata)))
	suite.Require().NoError(err)
}
