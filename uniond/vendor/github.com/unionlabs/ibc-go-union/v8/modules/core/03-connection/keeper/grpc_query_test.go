package keeper_test

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/types/query"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestQueryConnection() {
	var (
		req           *types.QueryConnectionRequest
		expConnection types.ConnectionEnd
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"invalid connectionID",
			func() {
				req = &types.QueryConnectionRequest{}
			},
			false,
		},
		{
			"connection not found",
			func() {
				req = &types.QueryConnectionRequest{
					ConnectionId: ibctesting.InvalidID,
				}
			},
			false,
		},
		{
			"success",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.SetupClients(path)
				err := path.EndpointA.ConnOpenInit()
				suite.Require().NoError(err)

				counterparty := types.NewCounterparty(path.EndpointB.ClientID, "", suite.chainB.GetPrefix())
				expConnection = types.NewConnectionEnd(types.INIT, path.EndpointA.ClientID, counterparty, types.GetCompatibleVersions(), 500)
				suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainA.GetContext(), path.EndpointA.ConnectionID, expConnection)

				req = &types.QueryConnectionRequest{
					ConnectionId: path.EndpointA.ConnectionID,
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := suite.chainA.GetContext()

			queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
			res, err := queryServer.Connection(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(&expConnection, res.Connection)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryConnections() {
	suite.chainA.App.GetIBCKeeper().ConnectionKeeper.CreateSentinelLocalhostConnection(suite.chainA.GetContext())
	localhostConn, found := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetConnection(suite.chainA.GetContext(), exported.LocalhostConnectionID)
	suite.Require().True(found)

	identifiedConn := types.NewIdentifiedConnection(exported.LocalhostConnectionID, localhostConn)

	var (
		req            *types.QueryConnectionsRequest
		expConnections = []*types.IdentifiedConnection{&identifiedConn}
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"empty pagination",
			func() {
				req = &types.QueryConnectionsRequest{}
			},
			true,
		},
		{
			"success",
			func() {
				path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
				path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
				path3 := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.SetupConnections(path1)
				suite.coordinator.SetupConnections(path2)
				suite.coordinator.SetupClients(path3)

				err := path3.EndpointA.ConnOpenInit()
				suite.Require().NoError(err)

				counterparty1 := types.NewCounterparty(path1.EndpointB.ClientID, path1.EndpointB.ConnectionID, suite.chainB.GetPrefix())
				counterparty2 := types.NewCounterparty(path2.EndpointB.ClientID, path2.EndpointB.ConnectionID, suite.chainB.GetPrefix())
				// counterparty connection id is blank after open init
				counterparty3 := types.NewCounterparty(path3.EndpointB.ClientID, "", suite.chainB.GetPrefix())

				conn1 := types.NewConnectionEnd(types.OPEN, path1.EndpointA.ClientID, counterparty1, types.GetCompatibleVersions(), 0)
				conn2 := types.NewConnectionEnd(types.OPEN, path2.EndpointA.ClientID, counterparty2, types.GetCompatibleVersions(), 0)
				conn3 := types.NewConnectionEnd(types.INIT, path3.EndpointA.ClientID, counterparty3, types.GetCompatibleVersions(), 0)

				iconn1 := types.NewIdentifiedConnection(path1.EndpointA.ConnectionID, conn1)
				iconn2 := types.NewIdentifiedConnection(path2.EndpointA.ConnectionID, conn2)
				iconn3 := types.NewIdentifiedConnection(path3.EndpointA.ConnectionID, conn3)

				expConnections = []*types.IdentifiedConnection{&iconn1, &iconn2, &iconn3, &identifiedConn}

				req = &types.QueryConnectionsRequest{
					Pagination: &query.PageRequest{
						Limit:      4,
						CountTotal: true,
					},
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := suite.chainA.GetContext()

			queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
			res, err := queryServer.Connections(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(expConnections, res.Connections)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryClientConnections() {
	var (
		req      *types.QueryClientConnectionsRequest
		expPaths []string
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"invalid connectionID",
			func() {
				req = &types.QueryClientConnectionsRequest{}
			},
			false,
		},
		{
			"connection not found",
			func() {
				req = &types.QueryClientConnectionsRequest{
					ClientId: ibctesting.InvalidID,
				}
			},
			false,
		},
		{
			"success",
			func() {
				path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.SetupConnections(path1)

				// create another connection using same underlying clients
				path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
				path2.EndpointA.ClientID = path1.EndpointA.ClientID
				path2.EndpointB.ClientID = path1.EndpointB.ClientID

				suite.coordinator.CreateConnections(path2)

				expPaths = []string{path1.EndpointA.ConnectionID, path2.EndpointA.ConnectionID}
				suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetClientConnectionPaths(suite.chainA.GetContext(), path1.EndpointA.ClientID, expPaths)

				req = &types.QueryClientConnectionsRequest{
					ClientId: path1.EndpointA.ClientID,
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := suite.chainA.GetContext()

			queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
			res, err := queryServer.ClientConnections(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(expPaths, res.ConnectionPaths)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryConnectionClientState() {
	var (
		req                      *types.QueryConnectionClientStateRequest
		expIdentifiedClientState clienttypes.IdentifiedClientState
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"invalid connection ID",
			func() {
				req = &types.QueryConnectionClientStateRequest{
					ConnectionId: "",
				}
			},
			false,
		},
		{
			"connection not found",
			func() {
				req = &types.QueryConnectionClientStateRequest{
					ConnectionId: "test-connection-id",
				}
			},
			false,
		},
		{
			"client state not found",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)

				// set connection to empty so clientID is empty
				suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainA.GetContext(), path.EndpointA.ConnectionID, types.ConnectionEnd{})

				req = &types.QueryConnectionClientStateRequest{
					ConnectionId: path.EndpointA.ConnectionID,
				}
			}, false,
		},
		{
			"success",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.SetupConnections(path)

				expClientState := suite.chainA.GetClientState(path.EndpointA.ClientID)
				expIdentifiedClientState = clienttypes.NewIdentifiedClientState(path.EndpointA.ClientID, expClientState)

				req = &types.QueryConnectionClientStateRequest{
					ConnectionId: path.EndpointA.ConnectionID,
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := suite.chainA.GetContext()

			queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
			res, err := queryServer.ConnectionClientState(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(&expIdentifiedClientState, res.IdentifiedClientState)

				// ensure UnpackInterfaces is defined
				cachedValue := res.IdentifiedClientState.ClientState.GetCachedValue()
				suite.Require().NotNil(cachedValue)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryConnectionConsensusState() {
	var (
		req               *types.QueryConnectionConsensusStateRequest
		expConsensusState exported.ConsensusState
		expClientID       string
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = nil
			},
			false,
		},
		{
			"invalid connection ID",
			func() {
				req = &types.QueryConnectionConsensusStateRequest{
					ConnectionId:   "",
					RevisionNumber: 0,
					RevisionHeight: 1,
				}
			},
			false,
		},
		{
			"connection not found",
			func() {
				req = &types.QueryConnectionConsensusStateRequest{
					ConnectionId:   "test-connection-id",
					RevisionNumber: 0,
					RevisionHeight: 1,
				}
			},
			false,
		},
		{
			"consensus state not found",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)

				req = &types.QueryConnectionConsensusStateRequest{
					ConnectionId:   path.EndpointA.ConnectionID,
					RevisionNumber: 0,
					RevisionHeight: uint64(suite.chainA.GetContext().BlockHeight()), // use current height
				}
			}, false,
		},
		{
			"success",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.SetupConnections(path)

				clientState := suite.chainA.GetClientState(path.EndpointA.ClientID)
				expConsensusState, _ = suite.chainA.GetConsensusState(path.EndpointA.ClientID, clientState.GetLatestHeight())
				suite.Require().NotNil(expConsensusState)
				expClientID = path.EndpointA.ClientID

				req = &types.QueryConnectionConsensusStateRequest{
					ConnectionId:   path.EndpointA.ConnectionID,
					RevisionNumber: clientState.GetLatestHeight().GetRevisionNumber(),
					RevisionHeight: clientState.GetLatestHeight().GetRevisionHeight(),
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()
			ctx := suite.chainA.GetContext()

			queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
			res, err := queryServer.ConnectionConsensusState(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				consensusState, err := clienttypes.UnpackConsensusState(res.ConsensusState)
				suite.Require().NoError(err)
				suite.Require().Equal(expConsensusState, consensusState)
				suite.Require().Equal(expClientID, res.ClientId)

				// ensure UnpackInterfaces is defined
				cachedValue := res.ConsensusState.GetCachedValue()
				suite.Require().NotNil(cachedValue)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestQueryConnectionParams() {
	ctx := suite.chainA.GetContext()
	expParams := types.DefaultParams()
	queryServer := keeper.NewQueryServer(suite.chainA.App.GetIBCKeeper().ConnectionKeeper)
	res, _ := queryServer.ConnectionParams(ctx, &types.QueryConnectionParamsRequest{})
	suite.Require().Equal(&expParams, res.Params)
}
