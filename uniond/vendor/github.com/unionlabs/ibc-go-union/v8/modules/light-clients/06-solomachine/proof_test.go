package solomachine_test

import (
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"

	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
)

func (suite *SoloMachineTestSuite) TestVerifySignature() {
	cdc := suite.chainA.App.AppCodec()
	signBytes := []byte("sign bytes")

	singleSignature := suite.solomachine.GenerateSignature(signBytes)
	singleSigData, err := solomachine.UnmarshalSignatureData(cdc, singleSignature)
	suite.Require().NoError(err)

	multiSignature := suite.solomachineMulti.GenerateSignature(signBytes)
	multiSigData, err := solomachine.UnmarshalSignatureData(cdc, multiSignature)
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		publicKey cryptotypes.PubKey
		sigData   signing.SignatureData
		expPass   bool
	}{
		{
			"single signature with regular public key",
			suite.solomachine.PublicKey,
			singleSigData,
			true,
		},
		{
			"multi signature with multisig public key",
			suite.solomachineMulti.PublicKey,
			multiSigData,
			true,
		},
		{
			"single signature with multisig public key",
			suite.solomachineMulti.PublicKey,
			singleSigData,
			false,
		},
		{
			"multi signature with regular public key",
			suite.solomachine.PublicKey,
			multiSigData,
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			err := solomachine.VerifySignature(tc.publicKey, signBytes, tc.sigData)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
