package keeper_test

import (
	"context"
	"math/rand"
	"testing"
	"time"

	testifysuite "github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	stakingtypes "cosmossdk.io/x/staking/types"
	"github.com/cosmos/cosmos-sdk/codec"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	cmttypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	localhost "github.com/cosmos/ibc-go/v8/modules/light-clients/09-localhost"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibctestingmock "github.com/cosmos/ibc-go/v8/testing/mock"
	"github.com/cosmos/ibc-go/v8/testing/simapp"
)

const (
	testChainID          = "gaiahub-0"
	testChainIDRevision1 = "gaiahub-1"

	testClientID  = "tendermint-0"
	testClientID2 = "tendermint-1"
	testClientID3 = "tendermint-2"

	trustingPeriod time.Duration = time.Hour * 24 * 7 * 2
	ubdPeriod      time.Duration = time.Hour * 24 * 7 * 3
	maxClockDrift  time.Duration = time.Second * 10
)

var testClientHeight = types.NewHeight(0, 5)

type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain

	cdc            codec.Codec
	ctx            context.Context
	keeper         *keeper.Keeper
	consensusState *ibctm.ConsensusState
	valSet         *cmttypes.ValidatorSet
	valSetHash     cmtbytes.HexBytes
	privVal        cmttypes.PrivValidator
	now            time.Time
	past           time.Time
	solomachine    *ibctesting.Solomachine

	signers map[string]cmttypes.PrivValidator
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))

	isCheckTx := false
	suite.now = time.Date(2020, 1, 2, 0, 0, 0, 0, time.UTC)
	suite.past = time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	app := simapp.Setup(suite.T(), isCheckTx)

	suite.cdc = app.AppCodec()
	suite.ctx = app.BaseApp.NewContext(isCheckTx)
	suite.keeper = app.IBCKeeper.ClientKeeper
	suite.privVal = cmttypes.NewMockPV()
	pubKey, err := suite.privVal.GetPubKey()
	suite.Require().NoError(err)

	validator := cmttypes.NewValidator(pubKey, 1)
	suite.valSet = cmttypes.NewValidatorSet([]*cmttypes.Validator{validator})
	suite.valSetHash = suite.valSet.Hash()

	suite.signers = make(map[string]cmttypes.PrivValidator, 1)
	suite.signers[validator.Address.String()] = suite.privVal

	suite.consensusState = ibctm.NewConsensusState(suite.now, commitmenttypes.NewMerkleRoot([]byte("hash")), suite.valSetHash)

	var validators stakingtypes.Validators
	for i := 1; i < 11; i++ {
		privVal := ibctestingmock.NewPV()
		tmPk, err := privVal.GetPubKey()
		suite.Require().NoError(err)
		pk, err := cryptocodec.FromCmtPubKeyInterface(tmPk)
		suite.Require().NoError(err)
		val, err := stakingtypes.NewValidator(pk.Address().String(), pk, stakingtypes.Description{})
		suite.Require().NoError(err)

		val.Status = stakingtypes.Bonded
		val.Tokens = sdkmath.NewInt(rand.Int63())
		validators.Validators = append(validators.Validators, val)
	}

	suite.solomachine = ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachinesingle", "testing", 1)
}

func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) TestSetClientState() {
	clientState := ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, types.ZeroHeight(), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
	suite.keeper.SetClientState(suite.ctx, testClientID, clientState)

	retrievedState, found := suite.keeper.GetClientState(suite.ctx, testClientID)
	suite.Require().True(found, "GetClientState failed")
	suite.Require().Equal(clientState, retrievedState, "Client states are not equal")
}

func (suite *KeeperTestSuite) TestSetClientConsensusState() {
	suite.keeper.SetClientConsensusState(suite.ctx, testClientID, testClientHeight, suite.consensusState)

	retrievedConsState, found := suite.keeper.GetClientConsensusState(suite.ctx, testClientID, testClientHeight)
	suite.Require().True(found, "GetConsensusState failed")

	tmConsState, ok := retrievedConsState.(*ibctm.ConsensusState)
	suite.Require().True(ok)
	suite.Require().Equal(suite.consensusState, tmConsState, "ConsensusState not stored correctly")
}

func (suite *KeeperTestSuite) TestGetAllGenesisClients() {
	clientIDs := []string{
		exported.LocalhostClientID, testClientID2, testClientID3, testClientID,
	}
	expClients := []exported.ClientState{
		localhost.NewClientState(types.GetSelfHeight(suite.chainA.GetContext())),
		ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, types.ZeroHeight(), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
		ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, types.ZeroHeight(), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
		ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, types.ZeroHeight(), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
	}

	expGenClients := make(types.IdentifiedClientStates, len(expClients))

	for i := range expClients {
		suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), clientIDs[i], expClients[i])
		expGenClients[i] = types.NewIdentifiedClientState(clientIDs[i], expClients[i])
	}

	genClients := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetAllGenesisClients(suite.chainA.GetContext())

	suite.Require().Equal(expGenClients.Sort(), genClients)
}

func (suite *KeeperTestSuite) TestGetAllGenesisMetadata() { //nolint:govet // this is a test, we are okay with copying locks
	expectedGenMetadata := []types.IdentifiedGenesisMetadata{
		types.NewIdentifiedGenesisMetadata(
			"07-tendermint-1",
			[]types.GenesisMetadata{
				types.NewGenesisMetadata(ibctm.ProcessedTimeKey(types.NewHeight(0, 1)), []byte("foo")),
				types.NewGenesisMetadata(ibctm.ProcessedTimeKey(types.NewHeight(0, 2)), []byte("bar")),
				types.NewGenesisMetadata(ibctm.ProcessedTimeKey(types.NewHeight(0, 3)), []byte("baz")),
			},
		),
		types.NewIdentifiedGenesisMetadata(
			"clientB",
			[]types.GenesisMetadata{
				types.NewGenesisMetadata(ibctm.ProcessedTimeKey(types.NewHeight(1, 100)), []byte("val1")),
				types.NewGenesisMetadata(ibctm.ProcessedTimeKey(types.NewHeight(2, 300)), []byte("val2")),
			},
		),
	}

	genClients := []types.IdentifiedClientState{
		types.NewIdentifiedClientState("07-tendermint-1", &ibctm.ClientState{}), types.NewIdentifiedClientState("clientB", &ibctm.ClientState{}),
	}

	suite.chainA.App.GetIBCKeeper().ClientKeeper.SetAllClientMetadata(suite.chainA.GetContext(), expectedGenMetadata)

	actualGenMetadata, err := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetAllClientMetadata(suite.chainA.GetContext(), genClients)
	suite.Require().NoError(err, "get client metadata returned error unexpectedly")
	suite.Require().Equal(expectedGenMetadata, actualGenMetadata, "retrieved metadata is unexpected")
}

// 2 clients in total are created on chainA. The first client is updated so it contains an initial consensus state
// and a consensus state at the update height.
func (suite *KeeperTestSuite) TestGetAllConsensusStates() { //nolint:govet // this is a test, we are okay with copying locks
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(path)

	clientState := path.EndpointA.GetClientState()
	expConsensusHeight0 := clientState.GetLatestHeight()
	consensusState0, ok := suite.chainA.GetConsensusState(path.EndpointA.ClientID, expConsensusHeight0)
	suite.Require().True(ok)

	// update client to create a second consensus state
	err := path.EndpointA.UpdateClient()
	suite.Require().NoError(err)

	clientState = path.EndpointA.GetClientState()
	expConsensusHeight1 := clientState.GetLatestHeight()
	suite.Require().True(expConsensusHeight1.GT(expConsensusHeight0))
	consensusState1, ok := suite.chainA.GetConsensusState(path.EndpointA.ClientID, expConsensusHeight1)
	suite.Require().True(ok)

	expConsensus := []exported.ConsensusState{
		consensusState0,
		consensusState1,
	}

	// create second client on chainA
	path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(path2)
	clientState = path2.EndpointA.GetClientState()

	expConsensusHeight2 := clientState.GetLatestHeight()
	consensusState2, ok := suite.chainA.GetConsensusState(path2.EndpointA.ClientID, expConsensusHeight2)
	suite.Require().True(ok)

	expConsensus2 := []exported.ConsensusState{consensusState2}

	expConsensusStates := types.ClientsConsensusStates{
		types.NewClientConsensusStates(path.EndpointA.ClientID, []types.ConsensusStateWithHeight{
			types.NewConsensusStateWithHeight(expConsensusHeight0.(types.Height), expConsensus[0]),
			types.NewConsensusStateWithHeight(expConsensusHeight1.(types.Height), expConsensus[1]),
		}),
		types.NewClientConsensusStates(path2.EndpointA.ClientID, []types.ConsensusStateWithHeight{
			types.NewConsensusStateWithHeight(expConsensusHeight2.(types.Height), expConsensus2[0]),
		}),
	}.Sort()

	consStates := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetAllConsensusStates(suite.chainA.GetContext())
	suite.Require().Equal(expConsensusStates, consStates, "%s \n\n%s", expConsensusStates, consStates)
}

func (suite *KeeperTestSuite) TestIterateClientStates() { //nolint:govet // this is a test, we are okay with copying locks
	paths := []*ibctesting.Path{
		ibctesting.NewPath(suite.chainA, suite.chainB),
		ibctesting.NewPath(suite.chainA, suite.chainB),
		ibctesting.NewPath(suite.chainA, suite.chainB),
	}

	solomachines := []*ibctesting.Solomachine{
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, ibctesting.DefaultSolomachineClientID, "testing", 1),
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "06-solomachine-1", "testing", 4),
	}

	var (
		expTMClientIDs = make([]string, len(paths))
		expSMClientIDs = make([]string, len(solomachines))
	)

	// create tendermint clients
	for i, path := range paths {
		suite.coordinator.SetupClients(path)
		expTMClientIDs[i] = path.EndpointA.ClientID
	}

	// create solomachine clients
	for i, sm := range solomachines {
		expSMClientIDs[i] = sm.CreateClient(suite.chainA)
	}

	testCases := []struct {
		name         string
		prefix       []byte
		expClientIDs func() []string
	}{
		{
			"all clientIDs",
			nil,
			func() []string {
				allClientIDs := []string{exported.LocalhostClientID}
				allClientIDs = append(allClientIDs, expSMClientIDs...)
				allClientIDs = append(allClientIDs, expTMClientIDs...)
				return allClientIDs
			},
		},
		{
			"tendermint clientIDs",
			[]byte(exported.Tendermint),
			func() []string {
				return expTMClientIDs
			},
		},
		{
			"solo machine clientIDs",
			[]byte(exported.Solomachine),
			func() []string {
				return expSMClientIDs
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			var clientIDs []string
			suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.IterateClientStates(suite.chainA.GetContext(), tc.prefix, func(clientID string, _ exported.ClientState) bool {
				clientIDs = append(clientIDs, clientID)
				return false
			})

			suite.Require().ElementsMatch(tc.expClientIDs(), clientIDs)
		})
	}
}

// TestDefaultSetParams tests the default params set are what is expected
func (suite *KeeperTestSuite) TestDefaultSetParams() {
	expParams := types.DefaultParams()

	clientKeeper := suite.chainA.App.GetIBCKeeper().ClientKeeper
	params := clientKeeper.GetParams(suite.chainA.GetContext())

	suite.Require().Equal(expParams, params)
	suite.Require().Equal(expParams.AllowedClients, clientKeeper.GetParams(suite.chainA.GetContext()).AllowedClients)
}

// TestParams tests that Param setting and retrieval works properly
func (suite *KeeperTestSuite) TestParams() {
	testCases := []struct {
		name    string
		input   types.Params
		expPass bool
	}{
		{"success: set default params", types.DefaultParams(), true},
		{"success: empty allowedClients", types.NewParams(), true},
		{"success: subset of allowedClients", types.NewParams(exported.Tendermint, exported.Localhost), true},
		{"failure: contains a single empty string value as allowedClient", types.NewParams(exported.Localhost, ""), false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			ctx := suite.chainA.GetContext()
			err := tc.input.Validate()
			suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.SetParams(ctx, tc.input)
			if tc.expPass {
				suite.Require().NoError(err)
				expected := tc.input
				p := suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.GetParams(ctx)
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
		suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.GetParams(ctx)
	})
}

// TestIBCSoftwareUpgrade tests that an IBC client upgrade has been properly scheduled
func (suite *KeeperTestSuite) TestIBCSoftwareUpgrade() {
	var (
		upgradedClientState *ibctm.ClientState
		oldPlan, plan       upgradetypes.Plan
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"valid upgrade proposal",
			func() {},
			nil,
		},
		{
			"valid upgrade proposal with previous IBC state", func() {
				oldPlan = upgradetypes.Plan{
					Name:   "upgrade IBC clients",
					Height: 100,
				}
			},
			nil,
		},
		{
			"fail: scheduling upgrade with plan height 0",
			func() {
				plan.Height = 0
			},
			sdkerrors.ErrInvalidRequest,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()  // reset
			oldPlan.Height = 0 // reset

			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)
			upgradedClientState = suite.chainA.GetClientState(path.EndpointA.ClientID).ZeroCustomFields().(*ibctm.ClientState)

			// use height 1000 to distinguish from old plan
			plan = upgradetypes.Plan{
				Name:   "upgrade IBC clients",
				Height: 1000,
			}

			tc.malleate()

			// set the old plan if it is not empty
			if oldPlan.Height != 0 {
				// set upgrade plan in the upgrade store
				store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(upgradetypes.StoreKey))
				bz := suite.chainA.App.AppCodec().MustMarshal(&oldPlan)
				store.Set(upgradetypes.PlanKey(), bz)

				bz, err := types.MarshalClientState(suite.chainA.App.AppCodec(), upgradedClientState)
				suite.Require().NoError(err)

				suite.Require().NoError(suite.chainA.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainA.GetContext(), oldPlan.Height, bz))
			}

			err := suite.chainA.App.GetIBCKeeper().ClientKeeper.ScheduleIBCSoftwareUpgrade(suite.chainA.GetContext(), plan, upgradedClientState)

			if tc.expError == nil {
				suite.Require().NoError(err)

				// check that the correct plan is returned
				storedPlan, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradePlan(suite.chainA.GetContext())
				suite.Require().NoError(err)
				suite.Require().Equal(plan, storedPlan)

				// check that old upgraded client state is cleared
				cs, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradedClient(suite.chainA.GetContext(), oldPlan.Height)
				suite.Require().ErrorIs(err, upgradetypes.ErrNoUpgradedClientFound)
				suite.Require().Empty(cs)

				// check that client state was set
				storedClientState, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradedClient(suite.chainA.GetContext(), plan.Height)
				suite.Require().NoError(err)
				clientState, err := types.UnmarshalClientState(suite.chainA.App.AppCodec(), storedClientState)
				suite.Require().NoError(err)
				suite.Require().Equal(upgradedClientState, clientState)
			} else {
				// check that the new plan wasn't stored
				storedPlan, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradePlan(suite.chainA.GetContext())
				if oldPlan.Height != 0 {
					// NOTE: this is only true if the ScheduleUpgrade function
					// returns an error before clearing the old plan
					suite.Require().NoError(err)
					suite.Require().Equal(oldPlan, storedPlan)
				} else {
					suite.Require().ErrorIs(err, upgradetypes.ErrNoUpgradePlanFound)
					suite.Require().Empty(storedPlan)
				}

				// check that client state was not set
				cs, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradedClient(suite.chainA.GetContext(), plan.Height)
				suite.Require().Empty(cs)
				suite.Require().ErrorIs(err, upgradetypes.ErrNoUpgradedClientFound)
			}
		})
	}
}
