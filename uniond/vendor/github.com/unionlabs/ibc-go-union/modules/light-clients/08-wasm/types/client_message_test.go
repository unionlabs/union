package types_test

import (
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func (suite *TypesTestSuite) TestClientMessageValidateBasic() {
	testCases := []struct {
		name          string
		clientMessage *types.ClientMessage
		expPass       bool
	}{
		{
			"valid client message",
			&types.ClientMessage{
				Data: []byte("data"),
			},
			true,
		},
		{
			"data is nil",
			&types.ClientMessage{
				Data: nil,
			},
			false,
		},
		{
			"data is empty",
			&types.ClientMessage{
				Data: []byte{},
			},
			false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			clientMessage := tc.clientMessage

			suite.Require().Equal(types.Wasm, clientMessage.ClientType())
			err := clientMessage.ValidateBasic()

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
