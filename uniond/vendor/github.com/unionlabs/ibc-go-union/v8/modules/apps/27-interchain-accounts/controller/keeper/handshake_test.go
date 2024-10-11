package keeper_test

import (
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

const (
	differentConnectionID = "connection-100"
)

func (suite *KeeperTestSuite) TestOnChanOpenInit() {
	var (
		channel  *channeltypes.Channel
		path     *ibctesting.Path
		chanCap  *capabilitytypes.Capability
		metadata icatypes.Metadata
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {},
			nil,
		},
		{
			"success: previous active channel closed",
			func() {
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.CLOSED,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        TestVersion,
				}

				path.EndpointA.SetChannel(channel)
			},
			nil,
		},
		{
			"success: empty channel version returns default metadata JSON string",
			func() {
				channel.Version = ""
			},
			nil,
		},
		{
			"success: channel reopening",
			func() {
				err := SetupICAPath(path, TestOwnerAddress)
				suite.Require().NoError(err)

				err = path.EndpointA.SetChannelState(channeltypes.CLOSED)
				suite.Require().NoError(err)

				err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
				suite.Require().NoError(err)

				path.EndpointA.ChannelID = ""
				path.EndpointB.ChannelID = ""
			},
			nil,
		},
		{
			"failure: different ordering from previous channel",
			func() {
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.CLOSED,
					Ordering:       channeltypes.UNORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        TestVersion,
				}

				path.EndpointA.SetChannel(channel)
			},
			channeltypes.ErrInvalidChannelOrdering,
		},
		{
			"invalid metadata -  previous metadata is different",
			func() {
				// set active channel to closed
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				// attempt to downgrade version by reinitializing channel with version 1, but setting channel to version 2
				metadata.Version = "ics27-2"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				closedChannel := channeltypes.Channel{
					State:          channeltypes.CLOSED,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        string(versionBytes),
				}
				path.EndpointA.SetChannel(closedChannel)
			},
			icatypes.ErrInvalidVersion,
		},
		{
			"invalid port ID",
			func() {
				path.EndpointA.ChannelConfig.PortID = "invalid-port-id" //nolint:goconst
			},
			icatypes.ErrInvalidControllerPort,
		},
		{
			"invalid counterparty port ID",
			func() {
				path.EndpointA.SetChannel(*channel)
				channel.Counterparty.PortId = "invalid-port-id"
			},
			icatypes.ErrInvalidHostPort,
		},
		{
			"invalid metadata bytestring",
			func() {
				path.EndpointA.SetChannel(*channel)
				channel.Version = "invalid-metadata-bytestring"
			},
			icatypes.ErrUnknownDataType,
		},
		{
			"unsupported encoding format",
			func() {
				metadata.Encoding = "invalid-encoding-format"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.Version = string(versionBytes)
				path.EndpointA.SetChannel(*channel)
			},
			icatypes.ErrInvalidCodec,
		},
		{
			"unsupported transaction type",
			func() {
				metadata.TxType = "invalid-tx-types"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.Version = string(versionBytes)
				path.EndpointA.SetChannel(*channel)
			},
			icatypes.ErrUnknownDataType,
		},
		{
			"connection not found",
			func() {
				channel.ConnectionHops = []string{"invalid-connnection-id"}
				path.EndpointA.SetChannel(*channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"connection not found with default empty channel version",
			func() {
				channel.ConnectionHops = []string{"connection-10"}
				channel.Version = ""
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"invalid controller connection ID",
			func() {
				metadata.ControllerConnectionId = "invalid-connnection-id"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.Version = string(versionBytes)
				path.EndpointA.SetChannel(*channel)
			},
			connectiontypes.ErrInvalidConnection,
		},
		{
			"invalid host connection ID",
			func() {
				metadata.HostConnectionId = "invalid-connnection-id"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.Version = string(versionBytes)
				path.EndpointA.SetChannel(*channel)
			},
			connectiontypes.ErrInvalidConnection,
		},
		{
			"invalid version",
			func() {
				metadata.Version = "invalid-version"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.Version = string(versionBytes)
				path.EndpointA.SetChannel(*channel)
			},
			icatypes.ErrInvalidVersion,
		},
		{
			"channel is already active (OPEN state)",
			func() {
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.OPEN,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        TestVersion,
				}
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, channel)
			},
			icatypes.ErrActiveChannelAlreadySet,
		},
		{
			"channel is already active (FLUSHING state)",
			func() {
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.FLUSHING,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointA.ConnectionID},
					Version:        TestVersion,
				}
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, channel)
			},
			icatypes.ErrActiveChannelAlreadySet,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			// mock init interchain account
			portID, err := icatypes.NewControllerPortID(TestOwnerAddress)
			suite.Require().NoError(err)

			portCap := suite.chainA.GetSimApp().IBCKeeper.PortKeeper.BindPort(suite.chainA.GetContext(), portID)
			suite.chainA.GetSimApp().ICAControllerKeeper.ClaimCapability(suite.chainA.GetContext(), portCap, host.PortPath(portID)) //nolint:errcheck // this error check isn't needed for tests
			path.EndpointA.ChannelConfig.PortID = portID

			// default values
			metadata = icatypes.NewMetadata(icatypes.Version, path.EndpointA.ConnectionID, path.EndpointB.ConnectionID, "", icatypes.EncodingProtobuf, icatypes.TxTypeSDKMultiMsg)
			versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
			suite.Require().NoError(err)

			counterparty := channeltypes.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			channel = &channeltypes.Channel{
				State:          channeltypes.INIT,
				Ordering:       channeltypes.ORDERED,
				Counterparty:   counterparty,
				ConnectionHops: []string{path.EndpointA.ConnectionID},
				Version:        string(versionBytes),
			}

			chanCap, err = suite.chainA.App.GetScopedIBCKeeper().NewCapability(suite.chainA.GetContext(), host.ChannelCapabilityPath(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			version, err := suite.chainA.GetSimApp().ICAControllerKeeper.OnChanOpenInit(suite.chainA.GetContext(), channel.Ordering, channel.GetConnectionHops(),
				path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, chanCap, channel.Counterparty, channel.Version,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(string(versionBytes), version)
			} else {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanOpenAck() {
	var (
		path     *ibctesting.Path
		metadata icatypes.Metadata
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
		{
			"invalid port ID - host chain",
			func() {
				path.EndpointA.ChannelConfig.PortID = icatypes.HostPortID
			},
			false,
		},
		{
			"invalid port ID - unexpected prefix",
			func() {
				path.EndpointA.ChannelConfig.PortID = "invalid-port-id"
			},
			false,
		},
		{
			"invalid metadata bytestring",
			func() {
				path.EndpointA.Counterparty.ChannelConfig.Version = "invalid-metadata-bytestring"
			},
			false,
		},
		{
			"unsupported encoding format",
			func() {
				metadata.Encoding = "invalid-encoding-format"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.Counterparty.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"unsupported transaction type",
			func() {
				metadata.TxType = "invalid-tx-types"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.Counterparty.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"invalid account address",
			func() {
				metadata.Address = "invalid-account-address"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.Counterparty.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"empty account address",
			func() {
				metadata.Address = ""

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.Counterparty.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"invalid counterparty version",
			func() {
				metadata.Version = "invalid-version"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.Counterparty.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"active channel already set",
			func() {
				// create a new channel and set it in state
				ch := channeltypes.NewChannel(channeltypes.OPEN, channeltypes.ORDERED, channeltypes.NewCounterparty(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID), []string{path.EndpointB.ConnectionID}, ibctesting.DefaultChannelVersion)
				suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.SetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, ch)

				// set the active channelID in state
				suite.chainA.GetSimApp().ICAControllerKeeper.SetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			}, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := RegisterInterchainAccount(path.EndpointA, TestOwnerAddress)
			suite.Require().NoError(err)

			err = path.EndpointB.ChanOpenTry()
			suite.Require().NoError(err)

			interchainAccAddr, exists := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
			suite.Require().True(exists)

			metadata = icatypes.NewMetadata(icatypes.Version, ibctesting.FirstConnectionID, ibctesting.FirstConnectionID, interchainAccAddr, icatypes.EncodingProtobuf, icatypes.TxTypeSDKMultiMsg)
			versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
			suite.Require().NoError(err)

			path.EndpointB.ChannelConfig.Version = string(versionBytes)

			tc.malleate() // malleate mutates test data

			err = suite.chainA.GetSimApp().ICAControllerKeeper.OnChanOpenAck(suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointA.Counterparty.ChannelConfig.Version,
			)

			if tc.expPass {
				suite.Require().NoError(err)

				activeChannelID, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetActiveChannelID(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				suite.Require().Equal(path.EndpointA.ChannelID, activeChannelID)

				interchainAccAddress, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetInterchainAccountAddress(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				suite.Require().Equal(metadata.Address, interchainAccAddress)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanCloseConfirm() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			err = suite.chainB.GetSimApp().ICAControllerKeeper.OnChanCloseConfirm(suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)

			activeChannelID, found := suite.chainB.GetSimApp().ICAControllerKeeper.GetActiveChannelID(suite.chainB.GetContext(), ibctesting.FirstConnectionID, path.EndpointB.ChannelConfig.PortID)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().False(found)
				suite.Require().Empty(activeChannelID)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanUpgradeInit() {
	const (
		invalidVersion = "invalid-version"
	)

	var (
		path     *ibctesting.Path
		metadata icatypes.Metadata
		version  string
		order    channeltypes.Order
	)

	// updateMetadata is a helper function which modifies the metadata stored in the channel version
	// and marshals it into a string to pass to OnChanUpgradeInit as the version string.
	updateMetadata := func(modificationFn func(*icatypes.Metadata)) {
		metadata, err := icatypes.MetadataFromVersion(path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version)
		suite.Require().NoError(err)
		modificationFn(&metadata)
		version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))
	}

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {},
			nil,
		},
		{
			name: "success: change order",
			malleate: func() {
				order = channeltypes.UNORDERED
			},
			expError: nil,
		},
		{
			name: "failure: connectionID not found",
			malleate: func() {
				// channelID is provided via the endpoint channelID
				path.EndpointA.ChannelID = "invalid channel"
			},
			expError: channeltypes.ErrChannelNotFound,
		},
		{
			name: "failure: invalid proposed connectionHops",
			malleate: func() {
				// connection hops is provided via endpoint connectionID
				path.EndpointA.ConnectionID = differentConnectionID
			},
			expError: channeltypes.ErrInvalidUpgrade,
		},
		{
			name: "failure: empty version",
			malleate: func() {
				version = ""
			},
			expError: icatypes.ErrInvalidVersion,
		},
		{
			name: "failure: cannot decode version string",
			malleate: func() {
				version = invalidVersion
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: cannot decode self version string",
			malleate: func() {
				ch := path.EndpointA.GetChannel()
				ch.Version = invalidVersion
				path.EndpointA.SetChannel(ch)
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: failed controller metadata validation, invalid encoding",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Encoding = "invalid-encoding"
				})
			},
			expError: icatypes.ErrInvalidCodec,
		},
		{
			name: "failure: failed controller metadata validation, invalid tx type",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.TxType = "invalid-tx-type"
				})
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: failed controller metadata validation, invalid interchain account version",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Version = "invalid-interchain-account-version"
				})
			},
			expError: icatypes.ErrInvalidVersion,
		},
		{
			name: "failure: interchain account address changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Address = TestOwnerAddress // use valid address
				})
			},
			expError: icatypes.ErrInvalidAccountAddress,
		},
		{
			name: "failure: controller connection ID has changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.ControllerConnectionId = differentConnectionID
				})
			},
			expError: connectiontypes.ErrInvalidConnection, // the explicit checks on the controller connection identifier are unreachable
		},
		{
			name: "failure: host connection ID has changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.HostConnectionId = differentConnectionID
				})
			},
			expError: connectiontypes.ErrInvalidConnection, // the explicit checks on the host connection identifier are unreachable
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			currentMetadata, err := suite.chainA.GetSimApp().ICAControllerKeeper.GetAppMetadata(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().NoError(err)

			order = channeltypes.ORDERED
			metadata = icatypes.NewDefaultMetadata(path.EndpointA.ConnectionID, path.EndpointB.ConnectionID)
			// use the same address as the previous metadata.
			metadata.Address = currentMetadata.Address

			// this is the actual change to the version.
			metadata.Encoding = icatypes.EncodingProto3JSON

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))

			version = path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version

			tc.malleate() // malleate mutates test data

			upgradeVersion, err := path.EndpointA.Chain.GetSimApp().ICAControllerKeeper.OnChanUpgradeInit(
				path.EndpointA.Chain.GetContext(),
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				order,
				[]string{path.EndpointA.ConnectionID},
				version,
			)

			expPass := tc.expError == nil

			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(upgradeVersion, version)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanUpgradeAck() {
	const (
		invalidVersion = "invalid-version"
	)

	var (
		path                *ibctesting.Path
		metadata            icatypes.Metadata
		counterpartyVersion string
	)

	// updateMetadata is a helper function which modifies the metadata stored in the channel version
	// and marshals it into a string to pass to OnChanUpgradeAck as the counterpartyVersion string.
	updateMetadata := func(modificationFn func(*icatypes.Metadata)) {
		metadata, err := icatypes.MetadataFromVersion(path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version)
		suite.Require().NoError(err)
		modificationFn(&metadata)
		counterpartyVersion = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))
	}

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {},
			nil,
		},
		{
			name: "failure: empty counterparty version",
			malleate: func() {
				counterpartyVersion = ""
			},
			expError: channeltypes.ErrInvalidChannelVersion,
		},
		{
			name: "failure: invalid counterparty version",
			malleate: func() {
				counterpartyVersion = invalidVersion
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: cannot decode self version string",
			malleate: func() {
				channel := path.EndpointA.GetChannel()
				channel.Version = invalidVersion
				path.EndpointA.SetChannel(channel)
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: channel not found",
			malleate: func() {
				// channelID is provided via the endpoint channelID
				path.EndpointA.ChannelID = "invalid channel"
			},
			expError: ibcerrors.ErrNotFound, // GetChannel error is unreachable
		},
		{
			name: "failure: failed controller metadata validation, invalid encoding",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Encoding = "invalid-encoding"
				})
			},
			expError: icatypes.ErrInvalidCodec,
		},
		{
			name: "failure: failed controller metadata validation, invalid tx type",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.TxType = "invalid-tx-type"
				})
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: failed controller metadata validation, invalid interchain account version",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Version = "invalid-interchain-account-version"
				})
			},
			expError: icatypes.ErrInvalidVersion,
		},
		{
			name: "failure: interchain account address changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Address = TestOwnerAddress // use valid address
				})
			},
			expError: icatypes.ErrInvalidAccountAddress,
		},
		{
			name: "failure: controller connection ID has changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.ControllerConnectionId = differentConnectionID
				})
			},
			expError: connectiontypes.ErrInvalidConnection, // the explicit checks on the controller identifier are unreachable
		},
		{
			name: "failure: host connection ID has changed",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.HostConnectionId = differentConnectionID
				})
			},
			expError: connectiontypes.ErrInvalidConnection, // the explicit checks on the host identifier are unreachable
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			currentMetadata, err := suite.chainA.GetSimApp().ICAControllerKeeper.GetAppMetadata(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().NoError(err)

			metadata = icatypes.NewDefaultMetadata(path.EndpointA.ConnectionID, path.EndpointB.ConnectionID)
			// use the same address as the previous metadata.
			metadata.Address = currentMetadata.Address

			// this is the actual change to the version.
			metadata.Encoding = icatypes.EncodingProto3JSON

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))

			err = path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			counterpartyVersion = path.EndpointB.GetChannel().Version

			tc.malleate() // malleate mutates test data

			err = suite.chainA.GetSimApp().ICAControllerKeeper.OnChanUpgradeAck(
				suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				counterpartyVersion,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(path.EndpointA.GetChannel().Version, counterpartyVersion)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}
