package types_test

import (
	"errors"
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

func (s *contextTestSuite) TestConfig_SetPurpose() {
	config := sdk.NewConfig()
	config.SetPurpose(44)
	s.Require().Equal(uint32(44), config.GetPurpose())

	config.SetPurpose(0)
	s.Require().Equal(uint32(0), config.GetPurpose())

	config.Seal()
	s.Require().Panics(func() { config.SetPurpose(10) })
}

func (s *configTestSuite) TestConfig_SetCoinType() {
	config := sdk.NewConfig()
	config.SetCoinType(1)
	s.Require().Equal(uint32(1), config.GetCoinType())
	config.SetCoinType(99)
	s.Require().Equal(uint32(99), config.GetCoinType())

	config.Seal()
	s.Require().Panics(func() { config.SetCoinType(99) })
}

func (s *configTestSuite) TestConfig_SetTxEncoder() {
	mockErr := errors.New("test")
	config := sdk.NewConfig()
	s.Require().Nil(config.GetTxEncoder())
	encFunc := sdk.TxEncoder(func(tx sdk.Tx) ([]byte, error) { return nil, nil })
	config.SetTxEncoder(encFunc)
	_, err := config.GetTxEncoder()(sdk.Tx(nil))
	s.Require().Error(mockErr, err)

	config.Seal()
	s.Require().Panics(func() { config.SetTxEncoder(encFunc) })
}

func (s *configTestSuite) TestConfig_SetFullFundraiserPath() {
	config := sdk.NewConfig()
	config.SetFullFundraiserPath("test/path")
	s.Require().Equal("test/path", config.GetFullFundraiserPath())

	config.SetFullFundraiserPath("test/poth")
	s.Require().Equal("test/poth", config.GetFullFundraiserPath())

	config.Seal()
	s.Require().Panics(func() { config.SetFullFundraiserPath("x/test/path") })
}

func (s *configTestSuite) TestKeyringServiceName() {
	s.Require().Equal(sdk.DefaultKeyringServiceName, sdk.KeyringServiceName())
}
