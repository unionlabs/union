package types

import (
	"testing"

	"github.com/stretchr/testify/suite"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type CodecTestSuite struct {
	suite.Suite
}

func TestCodecSuite(t *testing.T) {
	suite.Run(t, new(CodecTestSuite))
}

func (suite *CodecTestSuite) TestRegisterInterfaces() {
	registry := codectypes.NewInterfaceRegistry()
	registry.RegisterInterface(sdk.MsgInterfaceProtoName, (*sdk.Msg)(nil))
	RegisterInterfaces(registry)

	impls := registry.ListImplementations(sdk.MsgInterfaceProtoName)
	suite.Require().Equal(5, len(impls))
	suite.Require().ElementsMatch([]string{
		"/tokenfactory.v1beta1.MsgCreateDenom",
		"/tokenfactory.v1beta1.MsgMint",
		"/tokenfactory.v1beta1.MsgBurn",
		"/tokenfactory.v1beta1.MsgChangeAdmin",
		"/tokenfactory.v1beta1.MsgSetDenomMetadata",
	}, impls)
}
