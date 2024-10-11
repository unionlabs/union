package types_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type configTestSuite struct {
	suite.Suite
}

func TestConfigTestSuite(t *testing.T) {
	suite.Run(t, new(configTestSuite))
}

func (s *configTestSuite) TestKeyringServiceName() {
	s.Require().Equal(sdk.DefaultKeyringServiceName, sdk.KeyringServiceName())
}
