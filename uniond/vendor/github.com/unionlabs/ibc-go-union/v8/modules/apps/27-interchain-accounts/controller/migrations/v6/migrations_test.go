package v6_test

import (
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	v6 "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/migrations/v6"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

type MigrationsTestSuite struct {
	testifysuite.Suite

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain

	coordinator *ibctesting.Coordinator
	path        *ibctesting.Path
}

func (suite *MigrationsTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))

	suite.path = ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.path.EndpointA.ChannelConfig.PortID = icatypes.HostPortID
	suite.path.EndpointB.ChannelConfig.PortID = icatypes.HostPortID
	suite.path.EndpointA.ChannelConfig.Order = channeltypes.ORDERED
	suite.path.EndpointB.ChannelConfig.Order = channeltypes.ORDERED
	suite.path.EndpointA.ChannelConfig.Version = icatypes.NewDefaultMetadataString(ibctesting.FirstConnectionID, ibctesting.FirstConnectionID)
	suite.path.EndpointB.ChannelConfig.Version = icatypes.NewDefaultMetadataString(ibctesting.FirstConnectionID, ibctesting.FirstConnectionID)
}

func (suite *MigrationsTestSuite) SetupPath() error {
	if err := suite.RegisterInterchainAccount(suite.path.EndpointA, ibctesting.TestAccAddress); err != nil {
		return err
	}

	if err := suite.path.EndpointB.ChanOpenTry(); err != nil {
		return err
	}

	if err := suite.path.EndpointA.ChanOpenAck(); err != nil {
		return err
	}

	return suite.path.EndpointB.ChanOpenConfirm()
}

func (*MigrationsTestSuite) RegisterInterchainAccount(endpoint *ibctesting.Endpoint, owner string) error {
	portID, err := icatypes.NewControllerPortID(owner)
	if err != nil {
		return err
	}

	channelSequence := endpoint.Chain.App.GetIBCKeeper().ChannelKeeper.GetNextChannelSequence(endpoint.Chain.GetContext())

	if err := endpoint.Chain.GetSimApp().ICAControllerKeeper.RegisterInterchainAccountWithOrdering(endpoint.Chain.GetContext(), endpoint.ConnectionID, owner, endpoint.ChannelConfig.Version, channeltypes.ORDERED); err != nil {
		return err
	}

	// commit state changes for proof verification
	endpoint.Chain.NextBlock()

	// update port/channel ids
	endpoint.ChannelID = channeltypes.FormatChannelIdentifier(channelSequence)
	endpoint.ChannelConfig.PortID = portID

	return nil
}

func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(MigrationsTestSuite))
}

func (suite *MigrationsTestSuite) TestMigrateICS27ChannelCapability() {
	suite.SetupTest()
	suite.coordinator.SetupConnections(suite.path)

	err := suite.SetupPath()
	suite.Require().NoError(err)

	// create additional capabilities to cover edge cases
	suite.CreateMockCapabilities()

	// create and claim a new capability with ibc/mock for "channel-1"
	// note: suite.SetupPath() now claims the chanel capability using icacontroller for "channel-0"
	capName := host.ChannelCapabilityPath(suite.path.EndpointA.ChannelConfig.PortID, channeltypes.FormatChannelIdentifier(1))

	capability, err := suite.chainA.GetSimApp().ScopedIBCKeeper.NewCapability(suite.chainA.GetContext(), capName)
	suite.Require().NoError(err)

	err = suite.chainA.GetSimApp().ScopedICAMockKeeper.ClaimCapability(suite.chainA.GetContext(), capability, capName)
	suite.Require().NoError(err)

	// assert the capability is owned by the mock module
	capability, found := suite.chainA.GetSimApp().ScopedICAMockKeeper.GetCapability(suite.chainA.GetContext(), capName)
	suite.Require().NotNil(capability)
	suite.Require().True(found)

	isAuthenticated := suite.chainA.GetSimApp().ScopedICAMockKeeper.AuthenticateCapability(suite.chainA.GetContext(), capability, capName)
	suite.Require().True(isAuthenticated)

	capability, found = suite.chainA.GetSimApp().ScopedICAControllerKeeper.GetCapability(suite.chainA.GetContext(), capName)
	suite.Require().Nil(capability)
	suite.Require().False(found)

	suite.ResetMemStore() // empty the x/capability in-memory store

	err = v6.MigrateICS27ChannelCapability(
		suite.chainA.GetContext(),
		suite.chainA.Codec,
		suite.chainA.GetSimApp().GetKey(capabilitytypes.StoreKey),
		suite.chainA.GetSimApp().CapabilityKeeper,
		ibcmock.ModuleName+types.SubModuleName,
	)

	suite.Require().NoError(err)

	// assert the capability is now owned by the ICS27 controller submodule
	capability, found = suite.chainA.GetSimApp().ScopedICAControllerKeeper.GetCapability(suite.chainA.GetContext(), capName)
	suite.Require().NotNil(capability)
	suite.Require().True(found)

	isAuthenticated = suite.chainA.GetSimApp().ScopedICAControllerKeeper.AuthenticateCapability(suite.chainA.GetContext(), capability, capName)
	suite.Require().True(isAuthenticated)

	capability, found = suite.chainA.GetSimApp().ScopedICAMockKeeper.GetCapability(suite.chainA.GetContext(), capName)
	suite.Require().Nil(capability)
	suite.Require().False(found)

	// ensure channel capability for "channel-0" is still owned by the controller
	capName = host.ChannelCapabilityPath(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
	capability, found = suite.chainA.GetSimApp().ScopedICAControllerKeeper.GetCapability(suite.chainA.GetContext(), capName)
	suite.Require().NotNil(capability)
	suite.Require().True(found)

	isAuthenticated = suite.chainA.GetSimApp().ScopedICAControllerKeeper.AuthenticateCapability(suite.chainA.GetContext(), capability, capName)
	suite.Require().True(isAuthenticated)

	suite.AssertMockCapabiltiesUnchanged()
}

// CreateMockCapabilities creates an additional two capabilities used for testing purposes:
// 1. A capability with a single owner
// 2. A capability with two owners, neither of which is "ibc"
func (suite *MigrationsTestSuite) CreateMockCapabilities() {
	capability, err := suite.chainA.GetSimApp().ScopedIBCMockKeeper.NewCapability(suite.chainA.GetContext(), "mock_one")
	suite.Require().NoError(err)
	suite.Require().NotNil(capability)

	capability, err = suite.chainA.GetSimApp().ScopedICAMockKeeper.NewCapability(suite.chainA.GetContext(), "mock_two")
	suite.Require().NoError(err)
	suite.Require().NotNil(capability)

	err = suite.chainA.GetSimApp().ScopedIBCMockKeeper.ClaimCapability(suite.chainA.GetContext(), capability, "mock_two")
	suite.Require().NoError(err)
}

// AssertMockCapabiltiesUnchanged authenticates the mock capabilities created at the start of the test to ensure they remain unchanged
func (suite *MigrationsTestSuite) AssertMockCapabiltiesUnchanged() {
	capability, found := suite.chainA.GetSimApp().ScopedIBCMockKeeper.GetCapability(suite.chainA.GetContext(), "mock_one")
	suite.Require().True(found)
	suite.Require().NotNil(capability)

	capability, found = suite.chainA.GetSimApp().ScopedIBCMockKeeper.GetCapability(suite.chainA.GetContext(), "mock_two")
	suite.Require().True(found)
	suite.Require().NotNil(capability)

	isAuthenticated := suite.chainA.GetSimApp().ScopedICAMockKeeper.AuthenticateCapability(suite.chainA.GetContext(), capability, "mock_two")
	suite.Require().True(isAuthenticated)
}

// ResetMemstore removes all existing fwd and rev capability kv pairs and deletes `KeyMemInitialised` from the x/capability memstore.
// This effectively mocks a new chain binary being started. Migration code is run against persisted state only and allows the memstore to be reinitialised.
func (suite *MigrationsTestSuite) ResetMemStore() {
	memStore := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetMemKey(capabilitytypes.MemStoreKey))
	memStore.Delete(capabilitytypes.KeyMemInitialized)

	iterator := memStore.Iterator(nil, nil)
	defer coretypes.LogDeferred(suite.chainA.GetContext().Logger(), func() error { return iterator.Close() })

	for ; iterator.Valid(); iterator.Next() {
		memStore.Delete(iterator.Key())
	}
}
