package types_test

import (
	"fmt"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *TypesTestSuite) TestValidateBasic() {
	testCases := []struct {
		name       string
		packetData types.InterchainAccountPacketData
		expPass    bool
	}{
		{
			"success",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: "memo",
			},
			true,
		},
		{
			"success, empty memo",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
			},
			true,
		},
		{
			"type unspecified",
			types.InterchainAccountPacketData{
				Type: types.UNSPECIFIED,
				Data: []byte("data"),
				Memo: "memo",
			},
			false,
		},
		{
			"empty data",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte{},
				Memo: "memo",
			},
			false,
		},
		{
			"nil data",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: nil,
				Memo: "memo",
			},
			false,
		},
		{
			"memo too large",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: ibctesting.GenerateString(types.MaxMemoCharLength + 1),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			err := tc.packetData.ValidateBasic()

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *TypesTestSuite) TestGetPacketSender() {
	testCases := []struct {
		name      string
		srcPortID string
		expSender string
	}{
		{
			"success: port id has prefix",
			types.ControllerPortPrefix + ibctesting.TestAccAddress,
			ibctesting.TestAccAddress,
		},
		{
			"failure: missing prefix",
			ibctesting.TestAccAddress,
			"",
		},
		{
			"failure: empty port id",
			"",
			"",
		},
	}

	for _, tc := range testCases {
		tc := tc

		packetData := types.InterchainAccountPacketData{}
		suite.Require().Equal(tc.expSender, packetData.GetPacketSender(tc.srcPortID))
	}
}

func (suite *TypesTestSuite) TestPacketDataProvider() {
	expCallbackAddr := ibctesting.TestAccAddress

	testCases := []struct {
		name          string
		packetData    types.InterchainAccountPacketData
		expCustomData interface{}
	}{
		{
			"success: src_callback key in memo",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: fmt.Sprintf(`{"src_callback": {"address": "%s"}}`, expCallbackAddr),
			},
			map[string]interface{}{
				"address": expCallbackAddr,
			},
		},
		{
			"success: src_callback key in memo with additional fields",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: fmt.Sprintf(`{"src_callback": {"address": "%s", "gas_limit": "200000"}}`, expCallbackAddr),
			},
			map[string]interface{}{
				"address":   expCallbackAddr,
				"gas_limit": "200000",
			},
		},
		{
			"success: src_callback has string valu",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: `{"src_callback": "string"}`,
			},
			"string",
		},
		{
			"failure: empty memo",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: "",
			},
			nil,
		},
		{
			"failure: non-json memo",
			types.InterchainAccountPacketData{
				Type: types.EXECUTE_TX,
				Data: []byte("data"),
				Memo: "invalid",
			},
			nil,
		},
	}

	for _, tc := range testCases {
		tc := tc

		customData := tc.packetData.GetCustomPacketData("src_callback")
		suite.Require().Equal(tc.expCustomData, customData)
	}
}

func (suite *TypesTestSuite) TestPacketDataUnmarshalerInterface() {
	expPacketData := types.InterchainAccountPacketData{
		Type: types.EXECUTE_TX,
		Data: []byte("data"),
		Memo: "some memo",
	}

	var packetData types.InterchainAccountPacketData
	err := packetData.UnmarshalJSON(expPacketData.GetBytes())
	suite.Require().NoError(err)
	suite.Require().Equal(expPacketData, packetData)

	// test invalid packet data
	invalidPacketDataBytes := []byte("invalid packet data")

	var invalidPacketData types.InterchainAccountPacketData
	err = packetData.UnmarshalJSON(invalidPacketDataBytes)
	suite.Require().Error(err)
	suite.Require().Equal(types.InterchainAccountPacketData{}, invalidPacketData)
}
