package v7_test

import (
	v7 "github.com/cosmos/ibc-go/v8/modules/core/02-client/migrations/v7"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

func (suite *MigrationsV7TestSuite) TestMigrateLocalhostClient() {
	suite.SetupTest()

	// note: explicitly remove the localhost client before running migration handler
	clientStore := suite.chainA.GetSimApp().GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), exported.LocalhostClientID)
	clientStore.Delete(host.ClientStateKey())

	clientState, found := suite.chainA.GetSimApp().GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), exported.LocalhostClientID)
	suite.Require().False(found)
	suite.Require().Nil(clientState)

	err := v7.MigrateLocalhostClient(suite.chainA.GetContext(), suite.chainA.GetSimApp().GetIBCKeeper().ClientKeeper)
	suite.Require().NoError(err)

	clientState, found = suite.chainA.GetSimApp().GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), exported.LocalhostClientID)
	suite.Require().True(found)
	suite.Require().NotNil(clientState)
}
