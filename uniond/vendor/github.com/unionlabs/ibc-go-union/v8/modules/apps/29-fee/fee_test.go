package fee_test

import (
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	icacontroller "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller"
	icacontrollertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icahost "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host"
	icahosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

type FeeTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
	chainC *ibctesting.TestChain

	path     *ibctesting.Path
	pathAToC *ibctesting.Path
}

func (suite *FeeTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 3)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	suite.chainC = suite.coordinator.GetChain(ibctesting.GetChainID(3))

	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	mockFeeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
	path.EndpointA.ChannelConfig.Version = mockFeeVersion
	path.EndpointB.ChannelConfig.Version = mockFeeVersion
	path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
	path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
	suite.path = path

	path = ibctesting.NewPath(suite.chainA, suite.chainC)
	path.EndpointA.ChannelConfig.Version = mockFeeVersion
	path.EndpointB.ChannelConfig.Version = mockFeeVersion
	path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
	path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
	suite.pathAToC = path
}

func TestIBCFeeTestSuite(t *testing.T) {
	testifysuite.Run(t, new(FeeTestSuite))
}

func (suite *FeeTestSuite) CreateMockPacket() channeltypes.Packet {
	return channeltypes.NewPacket(
		ibcmock.MockPacketData,
		suite.chainA.SenderAccount.GetSequence(),
		suite.path.EndpointA.ChannelConfig.PortID,
		suite.path.EndpointA.ChannelID,
		suite.path.EndpointB.ChannelConfig.PortID,
		suite.path.EndpointB.ChannelID,
		clienttypes.NewHeight(0, 100),
		0,
	)
}

// RemoveFeeMiddleware removes:
// - Fee middleware from transfer module
// - Fee middleware from icahost submodule
// - Fee middleware from icacontroller submodule
// - The entire mock-fee module
//
// It does this by overriding the IBC router with a new router.
func RemoveFeeMiddleware(chain *ibctesting.TestChain) {
	channelKeeper := chain.GetSimApp().IBCKeeper.ChannelKeeper

	// Unseal the IBC router by force
	chain.GetSimApp().IBCKeeper.Router = nil
	chain.GetSimApp().IBCKeeper.PortKeeper.Router = nil

	newRouter := porttypes.NewRouter() // Create a new router
	// Remove Fee middleware from transfer module
	chain.GetSimApp().TransferKeeper.WithICS4Wrapper(channelKeeper)
	transferStack := transfer.NewIBCModule(chain.GetSimApp().TransferKeeper)
	newRouter.AddRoute(transfertypes.ModuleName, transferStack)

	// Remove Fee middleware from icahost submodule
	chain.GetSimApp().ICAHostKeeper.WithICS4Wrapper(channelKeeper)
	icaHostStack := icahost.NewIBCModule(chain.GetSimApp().ICAHostKeeper)
	newRouter.AddRoute(icahosttypes.SubModuleName, icaHostStack)

	// Remove Fee middleware from icacontroller submodule
	chain.GetSimApp().ICAControllerKeeper.WithICS4Wrapper(channelKeeper)
	icaControllerStack := icacontroller.NewIBCMiddleware(nil, chain.GetSimApp().ICAControllerKeeper)
	newRouter.AddRoute(icacontrollertypes.SubModuleName, icaControllerStack)

	// Override and seal the router
	chain.GetSimApp().IBCKeeper.SetRouter(newRouter)
}

// helper function
func lockFeeModule(chain *ibctesting.TestChain) {
	ctx := chain.GetContext()
	storeKey := chain.GetSimApp().GetKey(types.ModuleName)
	store := ctx.KVStore(storeKey)
	store.Set(types.KeyLocked(), []byte{1})
}
