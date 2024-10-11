package keeper_test

import (
	"fmt"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
}

func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) TestSetAndGetConnection() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(path)
	firstConnection := "connection-0"

	// check first connection does not exist
	_, existed := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetConnection(suite.chainA.GetContext(), firstConnection)
	suite.Require().False(existed)

	suite.coordinator.CreateConnections(path)
	_, existed = suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetConnection(suite.chainA.GetContext(), firstConnection)
	suite.Require().True(existed)
}

func (suite *KeeperTestSuite) TestSetAndGetClientConnectionPaths() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(path)

	_, existed := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetClientConnectionPaths(suite.chainA.GetContext(), path.EndpointA.ClientID)
	suite.False(existed)

	connections := []string{"connectionA", "connectionB"}
	suite.chainA.App.GetIBCKeeper().ConnectionKeeper.SetClientConnectionPaths(suite.chainA.GetContext(), path.EndpointA.ClientID, connections)
	paths, existed := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetClientConnectionPaths(suite.chainA.GetContext(), path.EndpointA.ClientID)
	suite.True(existed)
	suite.EqualValues(connections, paths)
}

// create 2 connections: A0 - B0, A1 - B1
func (suite KeeperTestSuite) TestGetAllConnections() { //nolint:govet // this is a test, we are okay with copying locks
	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path1)

	path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path2.EndpointA.ClientID = path1.EndpointA.ClientID
	path2.EndpointB.ClientID = path1.EndpointB.ClientID

	suite.coordinator.CreateConnections(path2)

	counterpartyB0 := types.NewCounterparty(path1.EndpointB.ClientID, path1.EndpointB.ConnectionID, suite.chainB.GetPrefix()) // connection B0
	counterpartyB1 := types.NewCounterparty(path2.EndpointB.ClientID, path2.EndpointB.ConnectionID, suite.chainB.GetPrefix()) // connection B1

	conn1 := types.NewConnectionEnd(types.OPEN, path1.EndpointA.ClientID, counterpartyB0, types.GetCompatibleVersions(), 0) // A0 - B0
	conn2 := types.NewConnectionEnd(types.OPEN, path2.EndpointA.ClientID, counterpartyB1, types.GetCompatibleVersions(), 0) // A1 - B1

	iconn1 := types.NewIdentifiedConnection(path1.EndpointA.ConnectionID, conn1)
	iconn2 := types.NewIdentifiedConnection(path2.EndpointA.ConnectionID, conn2)

	suite.chainA.App.GetIBCKeeper().ConnectionKeeper.CreateSentinelLocalhostConnection(suite.chainA.GetContext())
	localhostConn, found := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetConnection(suite.chainA.GetContext(), exported.LocalhostConnectionID)
	suite.Require().True(found)

	expConnections := []types.IdentifiedConnection{iconn1, iconn2, types.NewIdentifiedConnection(exported.LocalhostConnectionID, localhostConn)}

	connections := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetAllConnections(suite.chainA.GetContext())
	suite.Require().Len(connections, len(expConnections))
	suite.Require().Equal(expConnections, connections)
}

// the test creates 2 clients path.EndpointA.ClientID0 and path.EndpointA.ClientID1. path.EndpointA.ClientID0 has a single
// connection and path.EndpointA.ClientID1 has 2 connections.
func (suite KeeperTestSuite) TestGetAllClientConnectionPaths() { //nolint:govet // this is a test, we are okay with copying locks
	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path1)
	suite.coordinator.SetupConnections(path2)

	path3 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path3.EndpointA.ClientID = path2.EndpointA.ClientID
	path3.EndpointB.ClientID = path2.EndpointB.ClientID
	suite.coordinator.CreateConnections(path3)

	expPaths := []types.ConnectionPaths{
		types.NewConnectionPaths(path1.EndpointA.ClientID, []string{path1.EndpointA.ConnectionID}),
		types.NewConnectionPaths(path2.EndpointA.ClientID, []string{path2.EndpointA.ConnectionID, path3.EndpointA.ConnectionID}),
	}

	connPaths := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetAllClientConnectionPaths(suite.chainA.GetContext())
	suite.Require().Len(connPaths, 2)
	suite.Require().Equal(expPaths, connPaths)
}

// TestGetTimestampAtHeight verifies if the clients on each chain return the
// correct timestamp for the other chain.
func (suite *KeeperTestSuite) TestGetTimestampAtHeight() {
	var (
		connection types.ConnectionEnd
		height     exported.Height
	)

	cases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{"verification success", func() {
			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)
			connection = path.EndpointA.GetConnection()
			height = suite.chainB.LastHeader.GetHeight()
		}, true},
		{"client state not found", func() {}, false},
		{"consensus state not found", func() {
			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)
			connection = path.EndpointA.GetConnection()
			height = suite.chainB.LastHeader.GetHeight().Increment()
		}, false},
	}

	for _, tc := range cases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			tc.malleate()

			actualTimestamp, err := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetTimestampAtHeight(
				suite.chainA.GetContext(), connection, height,
			)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().EqualValues(uint64(suite.chainB.LastHeader.GetTime().UnixNano()), actualTimestamp)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLocalhostConnectionEndCreation() {
	ctx := suite.chainA.GetContext()
	connectionKeeper := suite.chainA.App.GetIBCKeeper().ConnectionKeeper
	connectionKeeper.CreateSentinelLocalhostConnection(ctx)

	connectionEnd, found := connectionKeeper.GetConnection(ctx, exported.LocalhostConnectionID)

	suite.Require().True(found)
	suite.Require().Equal(types.OPEN, connectionEnd.State)
	suite.Require().Equal(exported.LocalhostClientID, connectionEnd.ClientId)
	suite.Require().Equal(types.GetCompatibleVersions(), connectionEnd.Versions)
	expectedCounterParty := types.NewCounterparty(exported.LocalhostClientID, exported.LocalhostConnectionID, commitmenttypes.NewMerklePrefix(connectionKeeper.GetCommitmentPrefix().Bytes()))
	suite.Require().Equal(expectedCounterParty, connectionEnd.Counterparty)
}

// TestDefaultSetParams tests the default params set are what is expected
func (suite *KeeperTestSuite) TestDefaultSetParams() {
	expParams := types.DefaultParams()

	params := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetParams(suite.chainA.GetContext())
	suite.Require().Equal(expParams, params)
}

// TestParams tests that param setting and retrieval works properly
func (suite *KeeperTestSuite) TestSetAndGetParams() {
	testCases := []struct {
		name    string
		input   types.Params
		expPass bool
	}{
		{"success: set default params", types.DefaultParams(), true},
		{"success: valid value for MaxExpectedTimePerBlock", types.NewParams(10), true},
		{"failure: invalid value for MaxExpectedTimePerBlock", types.NewParams(0), false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			ctx := suite.chainA.GetContext()
			err := tc.input.Validate()
			suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper.SetParams(ctx, tc.input)
			if tc.expPass {
				suite.Require().NoError(err)
				expected := tc.input
				p := suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper.GetParams(ctx)
				suite.Require().Equal(expected, p)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// TestUnsetParams tests that trying to get params that are not set panics.
func (suite *KeeperTestSuite) TestUnsetParams() {
	suite.SetupTest()
	ctx := suite.chainA.GetContext()
	store := ctx.KVStore(suite.chainA.GetSimApp().GetKey(exported.StoreKey))
	store.Delete([]byte(types.ParamsKey))

	suite.Require().Panics(func() {
		suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper.GetParams(ctx)
	})
}
