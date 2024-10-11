package types_test

import (
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// use TestVersion as metadata being compared against
func (suite *TypesTestSuite) TestIsPreviousMetadataEqual() {
	var (
		metadata        types.Metadata
		previousVersion string
	)

	testCases := []struct {
		name     string
		malleate func()
		expEqual bool
	}{
		{
			"success",
			func() {
				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			true,
		},
		{
			"success with empty account address",
			func() {
				metadata.Address = ""

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			true,
		},
		{
			"cannot decode previous version",
			func() {
				previousVersion = "invalid previous version"
			},
			false,
		},
		{
			"unequal and invalid encoding format",
			func() {
				metadata.Encoding = "invalid-encoding-format"

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
		{
			"unequal encoding format",
			func() {
				metadata.Encoding = types.EncodingProto3JSON

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
		{
			"unequal transaction type",
			func() {
				metadata.TxType = "invalid-tx-type"

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
		{
			"unequal controller connection",
			func() {
				metadata.ControllerConnectionId = "connection-10"

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
		{
			"unequal host connection",
			func() {
				metadata.HostConnectionId = "connection-10"

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
		{
			"unequal version",
			func() {
				metadata.Version = "invalid version"

				versionBytes, err := types.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)
				previousVersion = string(versionBytes)
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			expectedMetadata := types.NewMetadata(types.Version, ibctesting.FirstConnectionID, ibctesting.FirstConnectionID, TestOwnerAddress, types.EncodingProtobuf, types.TxTypeSDKMultiMsg)
			metadata = expectedMetadata // default success case

			tc.malleate() // malleate mutates test data

			equal := types.IsPreviousMetadataEqual(previousVersion, expectedMetadata)

			if tc.expEqual {
				suite.Require().True(equal)
			} else {
				suite.Require().False(equal)
			}
		})
	}
}

func (suite *TypesTestSuite) TestValidateControllerMetadata() {
	var metadata types.Metadata

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success with empty account address",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                "",
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			true,
		},
		{
			"success with EncodingProto3JSON",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProto3JSON,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			true,
		},
		{
			"unsupported encoding format",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               "invalid-encoding-format",
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"unsupported transaction type",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 "invalid-tx-type",
				}
			},
			false,
		},
		{
			"invalid controller connection",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: "connection-10",
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid host connection",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       "connection-10",
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid address",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                " ",
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid version",
			func() {
				metadata = types.Metadata{
					Version:                "invalid version",
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			metadata = types.NewMetadata(types.Version, ibctesting.FirstConnectionID, ibctesting.FirstConnectionID, TestOwnerAddress, types.EncodingProtobuf, types.TxTypeSDKMultiMsg)

			tc.malleate() // malleate mutates test data

			err := types.ValidateControllerMetadata(
				suite.chainA.GetContext(),
				suite.chainA.App.GetIBCKeeper().ChannelKeeper,
				[]string{ibctesting.FirstConnectionID},
				metadata,
			)

			if tc.expPass {
				suite.Require().NoError(err, tc.name)
			} else {
				suite.Require().Error(err, tc.name)
			}
		})
	}
}

func (suite *TypesTestSuite) TestValidateHostMetadata() {
	var metadata types.Metadata

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success with empty account address",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                "",
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			true,
		},
		{
			"success with EncodingProto3JSON",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProto3JSON,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			true,
		},
		{
			"unsupported encoding format",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               "invalid-encoding-format",
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"unsupported transaction type",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 "invalid-tx-type",
				}
			},
			false,
		},
		{
			"invalid controller connection",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: "connection-10",
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid host connection",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       "connection-10",
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid address",
			func() {
				metadata = types.Metadata{
					Version:                types.Version,
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                " ",
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
		{
			"invalid version",
			func() {
				metadata = types.Metadata{
					Version:                "invalid version",
					ControllerConnectionId: ibctesting.FirstConnectionID,
					HostConnectionId:       ibctesting.FirstConnectionID,
					Address:                TestOwnerAddress,
					Encoding:               types.EncodingProtobuf,
					TxType:                 types.TxTypeSDKMultiMsg,
				}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			metadata = types.NewMetadata(types.Version, ibctesting.FirstConnectionID, ibctesting.FirstConnectionID, TestOwnerAddress, types.EncodingProtobuf, types.TxTypeSDKMultiMsg)

			tc.malleate() // malleate mutates test data

			err := types.ValidateHostMetadata(
				suite.chainA.GetContext(),
				suite.chainA.App.GetIBCKeeper().ChannelKeeper,
				[]string{ibctesting.FirstConnectionID},
				metadata,
			)

			if tc.expPass {
				suite.Require().NoError(err, tc.name)
			} else {
				suite.Require().Error(err, tc.name)
			}
		})
	}
}
