package types_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type testMsgSuite struct {
	suite.Suite
}

func TestMsgTestSuite(t *testing.T) {
	suite.Run(t, new(testMsgSuite))
}

func (s *testMsgSuite) TestMsg() {
	addr := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19}
	accAddr := sdk.AccAddress(addr)

	msg := testdata.NewTestMsg(accAddr)
	s.Require().NotNil(msg)
	s.Require().True(accAddr.Equals(msg.GetSigners()[0]))
	s.Require().Equal("TestMsg", msg.Route())
	s.Require().Equal("Test message", msg.Type())
	s.Require().Nil(msg.ValidateBasic())
	s.Require().NotPanics(func() { msg.GetSignBytes() })
}

func (s *testMsgSuite) TestMsgTypeURL() {
	s.Require().Equal("/testpb.TestMsg", sdk.MsgTypeURL(new(testdata.TestMsg)))
}

func (s *testMsgSuite) TestGetMsgFromTypeURL() {
	msg := new(testdata.TestMsg)
	cdc := codec.NewProtoCodec(testdata.NewTestInterfaceRegistry())

	result, err := sdk.GetMsgFromTypeURL(cdc, "/testpb.TestMsg")
	s.Require().NoError(err)
	s.Require().Equal(msg, result)
}
