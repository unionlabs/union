package keeper_test

import (
	"time"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// TestConnOpenInit - chainA initializes (INIT state) a connection with
// chainB which is yet UNINITIALIZED
func (suite *KeeperTestSuite) TestConnOpenInit() {
	var (
		path                 *ibctesting.Path
		version              *types.Version
		delayPeriod          uint64
		emptyConnBID         bool
		expErrorMsgSubstring string
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{"success", func() {
		}, true},
		{"success with empty counterparty identifier", func() {
			emptyConnBID = true
		}, true},
		{"success with non empty version", func() {
			version = types.GetCompatibleVersions()[0]
		}, true},
		{"success with non zero delayPeriod", func() {
			delayPeriod = uint64(time.Hour.Nanoseconds())
		}, true},

		{"invalid version", func() {
			version = &types.Version{}
		}, false},
		{"couldn't add connection to client", func() {
			// set path.EndpointA.ClientID to invalid client identifier
			path.EndpointA.ClientID = "clientidentifier"
		}, false},
		{
			msg:     "unauthorized client",
			expPass: false,
			malleate: func() {
				expErrorMsgSubstring = "status is Unauthorized"
				// remove client from allowed list
				params := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetParams(suite.chainA.GetContext())
				params.AllowedClients = []string{}
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetParams(suite.chainA.GetContext(), params)
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.msg, func() {
			suite.SetupTest()    // reset
			emptyConnBID = false // must be explicitly changed
			version = nil        // must be explicitly changed
			expErrorMsgSubstring = ""
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			tc.malleate()

			if emptyConnBID {
				path.EndpointB.ConnectionID = ""
			}
			counterparty := types.NewCounterparty(path.EndpointB.ClientID, path.EndpointB.ConnectionID, suite.chainB.GetPrefix())

			connectionID, err := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.ConnOpenInit(suite.chainA.GetContext(), path.EndpointA.ClientID, counterparty, version, delayPeriod)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(types.FormatConnectionIdentifier(0), connectionID)
			} else {
				suite.Require().Error(err)
				suite.Contains(err.Error(), expErrorMsgSubstring)
				suite.Require().Equal("", connectionID)
			}
		})
	}
}

// TestConnOpenTry - chainB calls ConnOpenTry to verify the state of
// connection on chainA is INIT
func (suite *KeeperTestSuite) TestConnOpenTry() {
	var (
		path               *ibctesting.Path
		delayPeriod        uint64
		versions           []*types.Version
		consensusHeight    exported.Height
		counterpartyClient exported.ClientState
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{"success", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainA to pass as counterpartyClient
			counterpartyClient = suite.chainA.GetClientState(path.EndpointA.ClientID)
		}, true},
		{"success with delay period", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			delayPeriod = uint64(time.Hour.Nanoseconds())

			// set delay period on counterparty to non-zero value
			conn := path.EndpointA.GetConnection()
			conn.DelayPeriod = delayPeriod
			suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainA.GetContext(), path.EndpointA.ConnectionID, conn)

			// commit in order for proof to return correct value
			suite.coordinator.CommitBlock(suite.chainA)
			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			// retrieve client state of chainA to pass as counterpartyClient
			counterpartyClient = suite.chainA.GetClientState(path.EndpointA.ClientID)
		}, true},
		{"counterparty versions is empty", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainA to pass as counterpartyClient
			counterpartyClient = suite.chainA.GetClientState(path.EndpointA.ClientID)

			versions = nil
		}, false},
		{"counterparty versions don't have a match", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainA to pass as counterpartyClient
			counterpartyClient = suite.chainA.GetClientState(path.EndpointA.ClientID)

			version := types.NewVersion("0.0", nil)
			versions = []*types.Version{version}
		}, false},
		{"connection state verification failed", func() {
			// chainA connection not created

			// retrieve client state of chainA to pass as counterpartyClient
			counterpartyClient = suite.chainA.GetClientState(path.EndpointA.ClientID)
		}, false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			suite.SetupTest()                          // reset
			consensusHeight = clienttypes.ZeroHeight() // may be changed in malleate
			versions = types.GetCompatibleVersions()   // may be changed in malleate
			delayPeriod = 0                            // may be changed in malleate
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			tc.malleate()

			counterparty := types.NewCounterparty(path.EndpointA.ClientID, path.EndpointA.ConnectionID, suite.chainA.GetPrefix())

			// ensure client is up to date to receive proof
			err := path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			connectionKey := host.ConnectionKey(path.EndpointA.ConnectionID)
			initProof, proofHeight := suite.chainA.QueryProof(connectionKey)

			if consensusHeight.IsZero() {
				// retrieve consensus state height to provide proof for
				consensusHeight = counterpartyClient.GetLatestHeight()
			}
			consensusKey := host.FullConsensusStateKey(path.EndpointA.ClientID, consensusHeight)
			consensusProof, _ := suite.chainA.QueryProof(consensusKey)

			// retrieve proof of counterparty clientstate on chainA
			clientKey := host.FullClientStateKey(path.EndpointA.ClientID)
			clientProof, _ := suite.chainA.QueryProof(clientKey)

			connectionID, err := suite.chainB.App.GetIBCKeeper().ConnectionKeeper.ConnOpenTry(
				suite.chainB.GetContext(), counterparty, delayPeriod, path.EndpointB.ClientID, counterpartyClient,
				versions, initProof, clientProof, consensusProof,
				proofHeight, consensusHeight,
			)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(types.FormatConnectionIdentifier(0), connectionID)
			} else {
				suite.Require().Error(err)
				suite.Require().Equal("", connectionID)
			}
		})
	}
}

// TestConnOpenAck - Chain A (ID #1) calls TestConnOpenAck to acknowledge (ACK state)
// the initialization (TRYINIT) of the connection on  Chain B (ID #2).
func (suite *KeeperTestSuite) TestConnOpenAck() {
	var (
		path               *ibctesting.Path
		consensusHeight    exported.Height
		version            *types.Version
		counterpartyClient exported.ClientState
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{"success", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)
		}, true},
		{"connection not found", func() {
			// connections are never created

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)
		}, false},
		{"invalid counterparty connection ID", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			// modify connB to set counterparty connection identifier to wrong identifier
			connection, found := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetConnection(suite.chainA.GetContext(), path.EndpointA.ConnectionID)
			suite.Require().True(found)

			connection.Counterparty.ConnectionId = "badconnectionid"
			path.EndpointB.ConnectionID = "badconnectionid"

			suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainA.GetContext(), path.EndpointA.ConnectionID, connection)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)
		}, false},
		{"connection state is not INIT", func() {
			// connection state is already OPEN on chainA
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ConnOpenAck()
			suite.Require().NoError(err)
		}, false},
		{"connection is in INIT but the proposed version is invalid", func() {
			// chainA is in INIT, chainB is in TRYOPEN
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			version = types.NewVersion("2.0", nil)
		}, false},
		{"incompatible IBC versions", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			// set version to a non-compatible version
			version = types.NewVersion("2.0", nil)
		}, false},
		{"empty version", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			version = &types.Version{}
		}, false},
		{"feature set verification failed - unsupported feature", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			version = types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_ORDERED", "ORDER_UNORDERED", "ORDER_DAG"})
		}, false},
		{"connection state verification failed", func() {
			// chainB connection is not in INIT
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			// retrieve client state of chainB to pass as counterpartyClient
			counterpartyClient = suite.chainB.GetClientState(path.EndpointB.ClientID)
		}, false},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.msg, func() {
			suite.SetupTest()                          // reset
			version = types.GetCompatibleVersions()[0] // must be explicitly changed in malleate
			consensusHeight = clienttypes.ZeroHeight() // must be explicitly changed in malleate
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			tc.malleate()

			// ensure client is up to date to receive proof
			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			connectionKey := host.ConnectionKey(path.EndpointB.ConnectionID)
			tryProof, proofHeight := suite.chainB.QueryProof(connectionKey)

			if consensusHeight.IsZero() {
				// retrieve consensus state height to provide proof for
				clientState := suite.chainB.GetClientState(path.EndpointB.ClientID)
				consensusHeight = clientState.GetLatestHeight()
			}
			consensusKey := host.FullConsensusStateKey(path.EndpointB.ClientID, consensusHeight)
			consensusProof, _ := suite.chainB.QueryProof(consensusKey)

			// retrieve proof of counterparty clientstate on chainA
			clientKey := host.FullClientStateKey(path.EndpointB.ClientID)
			clientProof, _ := suite.chainB.QueryProof(clientKey)

			err = suite.chainA.App.GetIBCKeeper().ConnectionKeeper.ConnOpenAck(
				suite.chainA.GetContext(), path.EndpointA.ConnectionID, counterpartyClient, version, path.EndpointB.ConnectionID,
				tryProof, clientProof, consensusProof, proofHeight, consensusHeight,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// TestConnOpenConfirm - chainB calls ConnOpenConfirm to confirm that
// chainA state is now OPEN.
func (suite *KeeperTestSuite) TestConnOpenConfirm() {
	var path *ibctesting.Path
	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{"success", func() {
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ConnOpenAck()
			suite.Require().NoError(err)
		}, true},
		{"connection not found", func() {
			// connections are never created
		}, false},
		{"chain B's connection state is not TRYOPEN", func() {
			// connections are OPEN
			suite.coordinator.CreateConnections(path)
		}, false},
		{"connection state verification failed", func() {
			// chainA is in INIT
			err := path.EndpointA.ConnOpenInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ConnOpenTry()
			suite.Require().NoError(err)
		}, false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			tc.malleate()

			// ensure client is up to date to receive proof
			err := path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			connectionKey := host.ConnectionKey(path.EndpointA.ConnectionID)
			ackProof, proofHeight := suite.chainA.QueryProof(connectionKey)

			err = suite.chainB.App.GetIBCKeeper().ConnectionKeeper.ConnOpenConfirm(
				suite.chainB.GetContext(), path.EndpointB.ConnectionID, ackProof, proofHeight,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
