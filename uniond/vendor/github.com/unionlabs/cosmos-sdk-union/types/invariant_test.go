package types_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type invariantTestSuite struct {
	suite.Suite
}

func TestInvariantTestSuite(t *testing.T) {
	suite.Run(t, new(invariantTestSuite))
}

func (s *invariantTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *invariantTestSuite) TestFormatInvariant() {
	s.Require().Equal(":  invariant\n\n", sdk.FormatInvariant("", "", ""))
	s.Require().Equal("module: name invariant\nmsg\n", sdk.FormatInvariant("module", "name", "msg"))
}
