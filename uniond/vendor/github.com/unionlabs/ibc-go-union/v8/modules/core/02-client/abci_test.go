package client_test

import (
	"strings"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	client "github.com/cosmos/ibc-go/v8/modules/core/02-client"
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type ClientTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

func (suite *ClientTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
}

func TestClientTestSuite(t *testing.T) {
	testifysuite.Run(t, new(ClientTestSuite))
}

func (suite *ClientTestSuite) TestBeginBlocker() {
	for i := 0; i < 10; i++ {
		// increment height
		suite.coordinator.CommitBlock(suite.chainA, suite.chainB)

		suite.Require().NotPanics(func() {
			client.BeginBlocker(suite.chainA.GetContext(), suite.chainA.App.GetIBCKeeper().ClientKeeper)
		}, "BeginBlocker shouldn't panic")
	}
}

func (suite *ClientTestSuite) TestBeginBlockerConsensusState() {
	plan := &upgradetypes.Plan{
		Name:   "test",
		Height: suite.chainA.GetContext().BlockHeight() + 1,
	}
	// set upgrade plan in the upgrade store
	store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(upgradetypes.StoreKey))
	bz := suite.chainA.App.AppCodec().MustMarshal(plan)
	store.Set(upgradetypes.PlanKey(), bz)

	nextValsHash := []byte("nextValsHash")
	newCtx := suite.chainA.GetContext().WithBlockHeader(tmproto.Header{
		ChainID:            suite.chainA.ChainID,
		Height:             suite.chainA.GetContext().BlockHeight(),
		NextValidatorsHash: nextValsHash,
	})

	err := suite.chainA.GetSimApp().UpgradeKeeper.SetUpgradedClient(newCtx, plan.Height, []byte("client state"))
	suite.Require().NoError(err)

	client.BeginBlocker(newCtx, suite.chainA.App.GetIBCKeeper().ClientKeeper)

	// plan Height is at ctx.BlockHeight+1
	consState, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradedConsensusState(newCtx, plan.Height)
	suite.Require().NoError(err)

	bz, err = types.MarshalConsensusState(suite.chainA.App.AppCodec(), &ibctm.ConsensusState{Timestamp: newCtx.BlockTime(), NextValidatorsHash: nextValsHash})
	suite.Require().NoError(err)
	suite.Require().Equal(bz, consState)
}

func (suite *ClientTestSuite) TestBeginBlockerUpgradeEvents() {
	plan := &upgradetypes.Plan{
		Name:   "test",
		Height: suite.chainA.GetContext().BlockHeight() + 1,
	}
	// set upgrade plan in the upgrade store
	store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(upgradetypes.StoreKey))
	bz := suite.chainA.App.AppCodec().MustMarshal(plan)
	store.Set(upgradetypes.PlanKey(), bz)

	nextValsHash := []byte("nextValsHash")
	newCtx := suite.chainA.GetContext().WithBlockHeader(tmproto.Header{
		Height:             suite.chainA.GetContext().BlockHeight(),
		NextValidatorsHash: nextValsHash,
	})

	err := suite.chainA.GetSimApp().UpgradeKeeper.SetUpgradedClient(newCtx, plan.Height, []byte("client state"))
	suite.Require().NoError(err)

	cacheCtx, writeCache := suite.chainA.GetContext().CacheContext()

	client.BeginBlocker(cacheCtx, suite.chainA.App.GetIBCKeeper().ClientKeeper)
	writeCache()

	suite.requireContainsEvent(cacheCtx.EventManager().Events(), types.EventTypeUpgradeChain, true)
}

func (suite *ClientTestSuite) TestBeginBlockerUpgradeEventsAbsence() {
	cacheCtx, writeCache := suite.chainA.GetContext().CacheContext()
	client.BeginBlocker(suite.chainA.GetContext(), suite.chainA.App.GetIBCKeeper().ClientKeeper)
	writeCache()
	suite.requireContainsEvent(cacheCtx.EventManager().Events(), types.EventTypeUpgradeChain, false)
}

// requireContainsEvent verifies if an event of a specific type was emitted.
func (suite *ClientTestSuite) requireContainsEvent(events sdk.Events, eventType string, shouldContain bool) {
	found := false
	var eventTypes []string
	for _, e := range events {
		eventTypes = append(eventTypes, e.Type)
		if e.Type == eventType {
			found = true
			break
		}
	}
	if shouldContain {
		suite.Require().True(found, "event type %s was not found in %s", eventType, strings.Join(eventTypes, ","))
	} else {
		suite.Require().False(found, "event type %s was found in %s", eventType, strings.Join(eventTypes, ","))
	}
}
