package types_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/testutil/mock"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type handlerTestSuite struct {
	suite.Suite
}

func TestHandlerTestSuite(t *testing.T) {
	suite.Run(t, new(handlerTestSuite))
}

func (s *handlerTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *handlerTestSuite) TestChainAnteDecorators() {
	// test panic
	s.Require().Nil(sdk.ChainAnteDecorators([]sdk.AnteDecorator{}...))

	ctx, tx := sdk.Context{}, sdk.Tx(nil)
	mockCtrl := gomock.NewController(s.T())
	mockAnteDecorator1 := mock.NewMockAnteDecorator(mockCtrl)
	mockAnteDecorator1.EXPECT().AnteHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Any()).Times(1)
	_, err := sdk.ChainAnteDecorators(mockAnteDecorator1)(ctx, tx, true)
	s.Require().NoError(err)

	mockAnteDecorator2 := mock.NewMockAnteDecorator(mockCtrl)
	// NOTE: we can't check that mockAnteDecorator2 is passed as the last argument because
	// ChainAnteDecorators wraps the decorators into closures, so each decorator is
	// receiving a closure.
	mockAnteDecorator1.EXPECT().AnteHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Any()).Times(1)
	mockAnteDecorator2.EXPECT().AnteHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Any()).Times(1)

	_, err = sdk.ChainAnteDecorators(
		mockAnteDecorator1,
		mockAnteDecorator2)(ctx, tx, true)
	s.Require().NoError(err)
}

func TestChainPostDecorators(t *testing.T) {
	// test panic when passing an empty sclice of PostDecorators
	require.Nil(t, sdk.ChainPostDecorators([]sdk.PostDecorator{}...))

	// Create empty context as well as transaction
	ctx := sdk.Context{}
	tx := sdk.Tx(nil)

	// Create mocks
	mockCtrl := gomock.NewController(t)
	mockPostDecorator1 := mock.NewMockPostDecorator(mockCtrl)
	mockPostDecorator2 := mock.NewMockPostDecorator(mockCtrl)

	// Test chaining only one post decorator
	mockPostDecorator1.EXPECT().PostHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Eq(true), gomock.Any()).Times(1)
	_, err := sdk.ChainPostDecorators(mockPostDecorator1)(ctx, tx, true, true)
	require.NoError(t, err)

	// Tests chaining multiple post decorators
	mockPostDecorator1.EXPECT().PostHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Eq(true), gomock.Any()).Times(1)
	mockPostDecorator2.EXPECT().PostHandle(gomock.Eq(ctx), gomock.Eq(tx), true, gomock.Eq(true), gomock.Any()).Times(1)
	// NOTE: we can't check that mockAnteDecorator2 is passed as the last argument because
	// ChainAnteDecorators wraps the decorators into closures, so each decorator is
	// receiving a closure.
	_, err = sdk.ChainPostDecorators(
		mockPostDecorator1,
		mockPostDecorator2,
	)(ctx, tx, true, true)
	require.NoError(t, err)
}
