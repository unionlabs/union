package keeper_test

import (
	"fmt"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/runtime"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"

	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibcfeekeeper "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

var (
	// TestOwnerAddress defines a reusable bech32 address for testing purposes
	TestOwnerAddress = "cosmos17dtl0mjt3t77kpuhg2edqzjpszulwhgzuj9ljs"

	// TestPortID defines a reusable port identifier for testing purposes
	TestPortID, _ = icatypes.NewControllerPortID(TestOwnerAddress)

	// TestVersion defines a reusable interchainaccounts version string for testing purposes
	TestVersion = string(icatypes.ModuleCdc.MustMarshalJSON(&icatypes.Metadata{
		Version:                icatypes.Version,
		ControllerConnectionId: ibctesting.FirstConnectionID,
		HostConnectionId:       ibctesting.FirstConnectionID,
		Encoding:               icatypes.EncodingProtobuf,
		TxType:                 icatypes.TxTypeSDKMultiMsg,
	}))

	// TestVersionWithJSONEncoding defines a reusable interchainaccounts version string that uses JSON encoding for testing purposes
	TestVersionWithJSONEncoding = string(icatypes.ModuleCdc.MustMarshalJSON(&icatypes.Metadata{
		Version:                icatypes.Version,
		ControllerConnectionId: ibctesting.FirstConnectionID,
		HostConnectionId:       ibctesting.FirstConnectionID,
		Encoding:               icatypes.EncodingProto3JSON,
		TxType:                 icatypes.TxTypeSDKMultiMsg,
	}))
)

type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
	chainC *ibctesting.TestChain
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 3)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	suite.chainC = suite.coordinator.GetChain(ibctesting.GetChainID(3))
}

func NewICAPath(chainA, chainB *ibctesting.TestChain, encoding string) *ibctesting.Path {
	path := ibctesting.NewPath(chainA, chainB)

	var version string
	switch encoding {
	case icatypes.EncodingProtobuf:
		version = TestVersion
	case icatypes.EncodingProto3JSON:
		version = TestVersionWithJSONEncoding
	default:
		panic(fmt.Errorf("unsupported encoding type: %s", encoding))
	}

	path.EndpointA.ChannelConfig.PortID = icatypes.HostPortID
	path.EndpointB.ChannelConfig.PortID = icatypes.HostPortID
	path.EndpointA.ChannelConfig.Order = channeltypes.ORDERED
	path.EndpointB.ChannelConfig.Order = channeltypes.ORDERED
	path.EndpointA.ChannelConfig.Version = version
	path.EndpointB.ChannelConfig.Version = version

	return path
}

// SetupICAPath invokes the InterchainAccounts entrypoint and subsequent channel handshake handlers
func SetupICAPath(path *ibctesting.Path, owner string) error {
	if err := RegisterInterchainAccount(path.EndpointA, owner); err != nil {
		return err
	}

	if err := path.EndpointB.ChanOpenTry(); err != nil {
		return err
	}

	if err := path.EndpointA.ChanOpenAck(); err != nil {
		return err
	}

	return path.EndpointB.ChanOpenConfirm()
}

// RegisterInterchainAccount is a helper function for starting the channel handshake
func RegisterInterchainAccount(endpoint *ibctesting.Endpoint, owner string) error {
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
	testifysuite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) TestNewKeeper() {
	testCases := []struct {
		name          string
		instantiateFn func()
		expPass       bool
	}{
		{"success", func() {
			keeper.NewKeeper(
				suite.chainA.GetSimApp().AppCodec(),
				runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(types.StoreKey)),
				suite.chainA.GetSimApp().GetSubspace(types.SubModuleName),
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.PortKeeper,
				suite.chainA.GetSimApp().AuthKeeper,
				suite.chainA.GetSimApp().ScopedICAHostKeeper,
				suite.chainA.GetSimApp().MsgServiceRouter(),
				suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(),
			)
		}, true},
		{"failure: interchain accounts module account does not exist", func() {
			keeper.NewKeeper(
				suite.chainA.GetSimApp().AppCodec(),
				runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(types.StoreKey)),
				suite.chainA.GetSimApp().GetSubspace(types.SubModuleName),
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.PortKeeper,
				authkeeper.AccountKeeper{}, // empty account keeper
				suite.chainA.GetSimApp().ScopedICAHostKeeper,
				suite.chainA.GetSimApp().MsgServiceRouter(),
				suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(),
			)
		}, false},
		{"failure: empty mock staking keeper", func() {
			keeper.NewKeeper(
				suite.chainA.GetSimApp().AppCodec(),
				runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(types.StoreKey)),
				suite.chainA.GetSimApp().GetSubspace(types.SubModuleName),
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper,
				suite.chainA.GetSimApp().IBCKeeper.PortKeeper,
				suite.chainA.GetSimApp().AuthKeeper,
				suite.chainA.GetSimApp().ScopedICAHostKeeper,
				suite.chainA.GetSimApp().MsgServiceRouter(),
				"", // authority
			)
		}, false},
	}

	for _, tc := range testCases {
		tc := tc
		suite.SetupTest()

		suite.Run(tc.name, func() {
			if tc.expPass {
				suite.Require().NotPanics(
					tc.instantiateFn,
				)
			} else {
				suite.Require().Panics(
					tc.instantiateFn,
				)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestNewModuleQuerySafeAllowList() {
	// Currently, all queries in bank, staking, auth, and circuit are marked safe
	// Notably, the gov and distribution modules are not marked safe

	var allowList []string
	suite.Require().NotPanics(func() {
		allowList = keeper.NewModuleQuerySafeAllowList()
	})

	suite.Require().NotEmpty(allowList)
	suite.Require().Contains(allowList, "/cosmos.bank.v1beta1.Query/Balance")
	suite.Require().Contains(allowList, "/cosmos.bank.v1beta1.Query/AllBalances")
	suite.Require().Contains(allowList, "/cosmos.staking.v1beta1.Query/Validator")
	suite.Require().Contains(allowList, "/cosmos.staking.v1beta1.Query/Validators")
	suite.Require().Contains(allowList, "/cosmos.circuit.v1.Query/Account")
	suite.Require().Contains(allowList, "/cosmos.circuit.v1.Query/DisabledList")
	suite.Require().Contains(allowList, "/cosmos.auth.v1beta1.Query/Accounts")
	suite.Require().Contains(allowList, "/cosmos.auth.v1beta1.Query/ModuleAccountByName")
	suite.Require().Contains(allowList, "/ibc.core.client.v1.Query/VerifyMembership")
	suite.Require().NotContains(allowList, "/cosmos.gov.v1beta1.Query/Proposals")
	suite.Require().NotContains(allowList, "/cosmos.gov.v1.Query/Proposals")
	suite.Require().NotContains(allowList, "/cosmos.distribution.v1beta1.Query/Params")
	suite.Require().NotContains(allowList, "/cosmos.distribution.v1beta1.Query/DelegationRewards")
}

func (suite *KeeperTestSuite) TestGetInterchainAccountAddress() {
	suite.SetupTest()

	path := NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
	suite.coordinator.SetupConnections(path)

	err := SetupICAPath(path, TestOwnerAddress)
	suite.Require().NoError(err)

	counterpartyPortID := path.EndpointA.ChannelConfig.PortID

	retrievedAddr, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, counterpartyPortID)
	suite.Require().True(found)
	suite.Require().NotEmpty(retrievedAddr)

	retrievedAddr, found = suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, "invalid port")
	suite.Require().False(found)
	suite.Require().Empty(retrievedAddr)
}

func (suite *KeeperTestSuite) TestGetAllActiveChannels() {
	var (
		expectedChannelID = "test-channel"
		expectedPortID    = "test-port"
	)

	suite.SetupTest()

	path := NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
	suite.coordinator.SetupConnections(path)

	err := SetupICAPath(path, TestOwnerAddress)
	suite.Require().NoError(err)

	suite.chainB.GetSimApp().ICAHostKeeper.SetActiveChannelID(suite.chainB.GetContext(), ibctesting.FirstConnectionID, expectedPortID, expectedChannelID)

	expectedChannels := []genesistypes.ActiveChannel{
		{
			ConnectionId: ibctesting.FirstConnectionID,
			PortId:       path.EndpointA.ChannelConfig.PortID,
			ChannelId:    path.EndpointB.ChannelID,
		},
		{
			ConnectionId: ibctesting.FirstConnectionID,
			PortId:       expectedPortID,
			ChannelId:    expectedChannelID,
		},
	}

	activeChannels := suite.chainB.GetSimApp().ICAHostKeeper.GetAllActiveChannels(suite.chainB.GetContext())
	suite.Require().Len(activeChannels, len(expectedChannels))
	suite.Require().Equal(expectedChannels, activeChannels)
}

func (suite *KeeperTestSuite) TestGetAllInterchainAccounts() {
	var (
		expectedAccAddr = "test-acc-addr"
		expectedPortID  = "test-port"
	)

	suite.SetupTest()

	path := NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
	suite.coordinator.SetupConnections(path)

	err := SetupICAPath(path, TestOwnerAddress)
	suite.Require().NoError(err)

	interchainAccAddr, exists := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
	suite.Require().True(exists)

	suite.chainB.GetSimApp().ICAHostKeeper.SetInterchainAccountAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, expectedPortID, expectedAccAddr)

	expectedAccounts := []genesistypes.RegisteredInterchainAccount{
		{
			ConnectionId:   ibctesting.FirstConnectionID,
			PortId:         TestPortID,
			AccountAddress: interchainAccAddr,
		},
		{
			ConnectionId:   ibctesting.FirstConnectionID,
			PortId:         expectedPortID,
			AccountAddress: expectedAccAddr,
		},
	}

	interchainAccounts := suite.chainB.GetSimApp().ICAHostKeeper.GetAllInterchainAccounts(suite.chainB.GetContext())
	suite.Require().Len(interchainAccounts, len(expectedAccounts))
	suite.Require().Equal(expectedAccounts, interchainAccounts)
}

func (suite *KeeperTestSuite) TestIsActiveChannel() {
	suite.SetupTest()

	path := NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
	suite.coordinator.SetupConnections(path)

	err := SetupICAPath(path, TestOwnerAddress)
	suite.Require().NoError(err)

	isActive := suite.chainB.GetSimApp().ICAHostKeeper.IsActiveChannel(suite.chainB.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
	suite.Require().True(isActive)
}

func (suite *KeeperTestSuite) TestSetInterchainAccountAddress() {
	var (
		expectedAccAddr = "test-acc-addr"
		expectedPortID  = "test-port"
	)

	suite.chainB.GetSimApp().ICAHostKeeper.SetInterchainAccountAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, expectedPortID, expectedAccAddr)

	retrievedAddr, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), ibctesting.FirstConnectionID, expectedPortID)
	suite.Require().True(found)
	suite.Require().Equal(expectedAccAddr, retrievedAddr)
}

func (suite *KeeperTestSuite) TestMetadataNotFound() {
	var (
		invalidPortID    = "invalid-port"
		invalidChannelID = "invalid-channel"
	)

	_, err := suite.chainB.GetSimApp().ICAHostKeeper.GetAppMetadata(suite.chainB.GetContext(), invalidPortID, invalidChannelID)
	suite.Require().ErrorIs(err, ibcerrors.ErrNotFound)
	suite.Require().Contains(err.Error(), fmt.Sprintf("app version not found for port %s and channel %s", invalidPortID, invalidChannelID))
}

func (suite *KeeperTestSuite) TestParams() {
	expParams := types.DefaultParams()

	params := suite.chainA.GetSimApp().ICAHostKeeper.GetParams(suite.chainA.GetContext())
	suite.Require().Equal(expParams, params)

	testCases := []struct {
		name    string
		input   types.Params
		expPass bool
	}{
		{"success: set default params", types.DefaultParams(), true},
		{"success: non-default params", types.NewParams(!types.DefaultHostEnabled, []string{"/cosmos.staking.v1beta1.MsgDelegate"}), true},
		{"success: set empty byte for allow messages", types.NewParams(true, nil), true},
		{"failure: set empty string for allow messages", types.NewParams(true, []string{""}), false},
		{"failure: set space string for allow messages", types.NewParams(true, []string{" "}), false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			ctx := suite.chainA.GetContext()
			err := tc.input.Validate()
			suite.chainA.GetSimApp().ICAHostKeeper.SetParams(ctx, tc.input)
			if tc.expPass {
				suite.Require().NoError(err)
				expected := tc.input
				p := suite.chainA.GetSimApp().ICAHostKeeper.GetParams(ctx)
				suite.Require().Equal(expected, p)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestUnsetParams() {
	suite.SetupTest()
	ctx := suite.chainA.GetContext()
	store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(types.SubModuleName))
	store.Delete([]byte(types.ParamsKey))

	suite.Require().Panics(func() {
		suite.chainA.GetSimApp().ICAHostKeeper.GetParams(ctx)
	})
}

func (suite *KeeperTestSuite) TestWithICS4Wrapper() {
	suite.SetupTest()

	// test if the ics4 wrapper is the fee keeper initially
	ics4Wrapper := suite.chainA.GetSimApp().ICAHostKeeper.GetICS4Wrapper()

	_, isFeeKeeper := ics4Wrapper.(ibcfeekeeper.Keeper)
	suite.Require().True(isFeeKeeper)
	_, isChannelKeeper := ics4Wrapper.(channelkeeper.Keeper)
	suite.Require().False(isChannelKeeper)

	// set the ics4 wrapper to the channel keeper
	suite.chainA.GetSimApp().ICAHostKeeper.WithICS4Wrapper(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper)
	ics4Wrapper = suite.chainA.GetSimApp().ICAHostKeeper.GetICS4Wrapper()

	_, isChannelKeeper = ics4Wrapper.(channelkeeper.Keeper)
	suite.Require().True(isChannelKeeper)
	_, isFeeKeeper = ics4Wrapper.(ibcfeekeeper.Keeper)
	suite.Require().False(isFeeKeeper)
}
