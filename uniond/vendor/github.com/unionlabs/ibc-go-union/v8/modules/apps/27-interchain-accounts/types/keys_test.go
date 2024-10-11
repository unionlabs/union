package types_test

import (
	fmt "fmt"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *TypesTestSuite) TestKeyActiveChannel() {
	key := types.KeyActiveChannel("port-id", "connection-id")
	suite.Require().Equal("activeChannel/port-id/connection-id", string(key))
}

func (suite *TypesTestSuite) TestKeyOwnerAccount() {
	key := types.KeyOwnerAccount("port-id", "connection-id")
	suite.Require().Equal("owner/port-id/connection-id", string(key))
}

func (suite *TypesTestSuite) TestKeyIsMiddlewareEnabled() {
	key := types.KeyIsMiddlewareEnabled(ibctesting.MockPort, ibctesting.FirstChannelID)
	suite.Require().Equal(fmt.Sprintf("%s/%s/%s", types.IsMiddlewareEnabledPrefix, ibctesting.MockPort, ibctesting.FirstChannelID), string(key))
}
