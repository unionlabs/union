package keeper_test

import (
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

const (
	differentConnectionID = "connection-100"
)

// open and close channel is a helper function for TestOnChanOpenTry for reopening accounts
func (suite *KeeperTestSuite) openAndCloseChannel(path *ibctesting.Path) {
	err := path.EndpointB.ChanOpenTry()
	suite.Require().NoError(err)

	err = path.EndpointA.ChanOpenAck()
	suite.Require().NoError(err)

	err = path.EndpointB.ChanOpenConfirm()
	suite.Require().NoError(err)

	err = path.EndpointA.SetChannelState(channeltypes.CLOSED)
	suite.Require().NoError(err)

	err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
	suite.Require().NoError(err)

	path.EndpointA.ChannelID = ""
	err = RegisterInterchainAccount(path.EndpointA, TestOwnerAddress)
	suite.Require().NoError(err)

	// bump channel sequence as these test mock core IBC behaviour on ChanOpenTry
	channelSequence := path.EndpointB.Chain.App.GetIBCKeeper().ChannelKeeper.GetNextChannelSequence(path.EndpointB.Chain.GetContext())
	path.EndpointB.ChannelID = channeltypes.FormatChannelIdentifier(channelSequence)
}

func (suite *KeeperTestSuite) TestOnChanOpenTry() {
	var (
		channel  *channeltypes.Channel
		path     *ibctesting.Path
		chanCap  *capabilitytypes.Capability
		metadata icatypes.Metadata
	)

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
			"success - reopening closed active channel",
			func() {
				// create interchain account
				// undo setup
				path.EndpointB.ChannelID = ""
				err := suite.chainB.App.GetScopedIBCKeeper().ReleaseCapability(suite.chainB.GetContext(), chanCap)
				suite.Require().NoError(err)

				suite.openAndCloseChannel(path)
			}, true,
		},
		{
			"success - reopening account with new address",
			func() {
				// create interchain account
				// undo setup
				path.EndpointB.ChannelID = ""
				err := suite.chainB.App.GetScopedIBCKeeper().ReleaseCapability(suite.chainB.GetContext(), chanCap)
				suite.Require().NoError(err)

				suite.openAndCloseChannel(path)

				// delete interchain account address
				store := suite.chainB.GetContext().KVStore(suite.chainB.GetSimApp().GetKey(hosttypes.SubModuleName))
				store.Delete(icatypes.KeyOwnerAccount(path.EndpointA.ChannelConfig.PortID, path.EndpointB.ConnectionID))

				// assert interchain account address mapping was deleted
				_, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().False(found)
			}, true,
		},
		{
			"success - previous metadata is different",
			func() {
				// set the active channelID in state
				suite.chainB.GetSimApp().ICAHostKeeper.SetActiveChannelID(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointB.ChannelID)

				// set the previous encoding to be proto3json.
				// the new encoding is set to be protobuf in the test below.
				metadata.Encoding = icatypes.EncodingProto3JSON

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				channel.State = channeltypes.CLOSED
				channel.Version = string(versionBytes)

				path.EndpointB.SetChannel(*channel)
			}, true,
		},
		{
			"reopening account fails - no existing account",
			func() {
				// create interchain account
				// undo setup
				path.EndpointB.ChannelID = ""
				err := suite.chainB.App.GetScopedIBCKeeper().ReleaseCapability(suite.chainB.GetContext(), chanCap)
				suite.Require().NoError(err)

				suite.openAndCloseChannel(path)

				// delete existing account
				addr, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				acc := suite.chainB.GetSimApp().AuthKeeper.GetAccount(suite.chainB.GetContext(), sdk.MustAccAddressFromBech32(addr))
				suite.chainB.GetSimApp().AuthKeeper.RemoveAccount(suite.chainB.GetContext(), acc)
			}, false,
		},
		{
			"reopening account fails - existing account is not interchain account type",
			func() {
				// create interchain account
				// undo setup
				path.EndpointB.ChannelID = ""
				err := suite.chainB.App.GetScopedIBCKeeper().ReleaseCapability(suite.chainB.GetContext(), chanCap)
				suite.Require().NoError(err)

				suite.openAndCloseChannel(path)

				addr, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				accAddress := sdk.MustAccAddressFromBech32(addr)
				acc := suite.chainB.GetSimApp().AuthKeeper.GetAccount(suite.chainB.GetContext(), accAddress)

				icaAcc, ok := acc.(*icatypes.InterchainAccount)
				suite.Require().True(ok)

				// overwrite existing account with only base account type, not intercahin account type
				suite.chainB.GetSimApp().AuthKeeper.SetAccount(suite.chainB.GetContext(), icaAcc.BaseAccount)
			}, false,
		},
		{
			"account already exists",
			func() {
				interchainAccAddr := icatypes.GenerateAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				err := suite.chainB.GetSimApp().BankKeeper.SendCoins(suite.chainB.GetContext(), suite.chainB.SenderAccount.GetAddress(), interchainAccAddr, sdk.Coins{sdk.NewCoin("stake", sdkmath.NewInt(1))})
				suite.Require().NoError(err)
				suite.Require().True(suite.chainB.GetSimApp().AuthKeeper.HasAccount(suite.chainB.GetContext(), interchainAccAddr))
			},
			false,
		},
		{
			"invalid port ID",
			func() {
				path.EndpointB.ChannelConfig.PortID = "invalid-port-id" //nolint:goconst
			},
			false,
		},
		{
			"connection not found",
			func() {
				channel.ConnectionHops = []string{"invalid-connnection-id"}
				path.EndpointB.SetChannel(*channel)
			},
			false,
		},
		{
			"invalid metadata bytestring",
			func() {
				// the try step will propose a new valid version
				path.EndpointA.ChannelConfig.Version = "invalid-metadata-bytestring"
			},
			true,
		},
		{
			"unsupported encoding format",
			func() {
				metadata.Encoding = "invalid-encoding-format"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"unsupported transaction type",
			func() {
				metadata.TxType = "invalid-tx-types"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"invalid controller connection ID",
			func() {
				metadata.ControllerConnectionId = "invalid-connnection-id"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"invalid host connection ID",
			func() {
				metadata.HostConnectionId = "invalid-connnection-id"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"invalid counterparty version",
			func() {
				metadata.Version = "invalid-version"

				versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
				suite.Require().NoError(err)

				path.EndpointA.ChannelConfig.Version = string(versionBytes)
			},
			false,
		},
		{
			"capability already claimed",
			func() {
				path.EndpointB.SetChannel(*channel)
				err := suite.chainB.GetSimApp().ScopedICAHostKeeper.ClaimCapability(suite.chainB.GetContext(), chanCap, host.ChannelCapabilityPath(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID))
				suite.Require().NoError(err)
			},
			false,
		},
		{
			"active channel already set (OPEN state)",
			func() {
				// create a new channel and set it in state
				ch := channeltypes.NewChannel(channeltypes.OPEN, channeltypes.ORDERED, channeltypes.NewCounterparty(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID), []string{path.EndpointA.ConnectionID}, ibctesting.DefaultChannelVersion)
				suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.SetChannel(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, ch)

				// set the active channelID in state
				suite.chainB.GetSimApp().ICAHostKeeper.SetActiveChannelID(suite.chainB.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointB.ChannelID)
			}, false,
		},
		{
			"channel is already active (FLUSHING state)",
			func() {
				suite.chainB.GetSimApp().ICAHostKeeper.SetActiveChannelID(suite.chainB.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, path.EndpointB.ChannelID)

				counterparty := channeltypes.NewCounterparty(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channel := channeltypes.Channel{
					State:          channeltypes.FLUSHING,
					Ordering:       channeltypes.ORDERED,
					Counterparty:   counterparty,
					ConnectionHops: []string{path.EndpointB.ConnectionID},
					Version:        TestVersion,
				}
				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetChannel(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, channel)
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
			suite.coordinator.SetupConnections(path)

			err := RegisterInterchainAccount(path.EndpointA, TestOwnerAddress)
			suite.Require().NoError(err)

			// set the channel id on host
			channelSequence := path.EndpointB.Chain.App.GetIBCKeeper().ChannelKeeper.GetNextChannelSequence(path.EndpointB.Chain.GetContext())
			path.EndpointB.ChannelID = channeltypes.FormatChannelIdentifier(channelSequence)

			// default values
			metadata = icatypes.NewMetadata(icatypes.Version, ibctesting.FirstConnectionID, ibctesting.FirstConnectionID, "", icatypes.EncodingProtobuf, icatypes.TxTypeSDKMultiMsg)
			versionBytes, err := icatypes.ModuleCdc.MarshalJSON(&metadata)
			suite.Require().NoError(err)

			counterparty := channeltypes.NewCounterparty(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			channel = &channeltypes.Channel{
				State:          channeltypes.TRYOPEN,
				Ordering:       channeltypes.ORDERED,
				Counterparty:   counterparty,
				ConnectionHops: []string{path.EndpointB.ConnectionID},
				Version:        string(versionBytes),
			}

			chanCap, err = suite.chainB.App.GetScopedIBCKeeper().NewCapability(suite.chainB.GetContext(), host.ChannelCapabilityPath(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID))
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			version, err := suite.chainB.GetSimApp().ICAHostKeeper.OnChanOpenTry(suite.chainB.GetContext(), channel.Ordering, channel.GetConnectionHops(),
				path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, chanCap, channel.Counterparty, path.EndpointA.ChannelConfig.Version,
			)

			if tc.expPass {
				suite.Require().NoError(err)

				storedAddr, found := suite.chainB.GetSimApp().ICAHostKeeper.GetInterchainAccountAddress(suite.chainB.GetContext(), path.EndpointB.ConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				interchainAccAddr, err := sdk.AccAddressFromBech32(storedAddr)
				suite.Require().NoError(err)

				// Check if account is created
				interchainAccount := suite.chainB.GetSimApp().AuthKeeper.GetAccount(suite.chainB.GetContext(), interchainAccAddr)
				suite.Require().Equal(interchainAccount.GetAddress().String(), storedAddr)
			} else {
				suite.Require().Error(err)
				suite.Require().Equal("", version)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanOpenConfirm() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
		{
			"channel not found",
			func() {
				path.EndpointB.ChannelID = "invalid-channel-id"
				path.EndpointB.ChannelConfig.PortID = "invalid-port-id"
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
			suite.coordinator.SetupConnections(path)

			err := RegisterInterchainAccount(path.EndpointA, TestOwnerAddress)
			suite.Require().NoError(err)

			err = path.EndpointB.ChanOpenTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanOpenAck()
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			err = suite.chainB.GetSimApp().ICAHostKeeper.OnChanOpenConfirm(suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)

			if tc.expPass {
				suite.Require().NoError(err)
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
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			err = suite.chainB.GetSimApp().ICAHostKeeper.OnChanCloseConfirm(suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnChanUpgradeTry() {
	var (
		path                *ibctesting.Path
		metadata            icatypes.Metadata
		order               channeltypes.Order
		counterpartyVersion string
	)

	// updateMetadata is a helper function which modifies the metadata stored in the channel version
	// and marshals it into a string to pass to OnChanUpgradeTry as the counterpartyVersion string.
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
			name: "success: change order",
			malleate: func() {
				order = channeltypes.UNORDERED
			},
			expError: nil,
		},
		{
			name: "failure: invalid port ID",
			malleate: func() {
				path.EndpointB.ChannelConfig.PortID = "invalid-port-id"
			},
			expError: porttypes.ErrInvalidPort,
		},
		{
			name: "failure: invalid proposed connectionHops",
			malleate: func() {
				// connection hops is provided via endpoint connectionID
				path.EndpointB.ConnectionID = differentConnectionID
			},
			expError: channeltypes.ErrInvalidUpgrade,
		},
		{
			name: "failure: empty counterparty version",
			malleate: func() {
				counterpartyVersion = ""
			},
			expError: channeltypes.ErrInvalidChannelVersion,
		},
		{
			name: "failure: cannot parse metadata from counterparty version string",
			malleate: func() {
				counterpartyVersion = "invalid-version"
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: cannot decode version string from channel",
			malleate: func() {
				channel := path.EndpointB.GetChannel()
				channel.Version = "invalid-metadata-string"
				path.EndpointB.SetChannel(channel)
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: metadata encoding not supported",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.Encoding = "invalid-encoding-format"
				})
			},
			expError: icatypes.ErrInvalidCodec,
		},
		{
			name: "failure: metadata tx type not supported",
			malleate: func() {
				updateMetadata(func(metadata *icatypes.Metadata) {
					metadata.TxType = "invalid-tx-type"
				})
			},
			expError: icatypes.ErrUnknownDataType,
		},
		{
			name: "failure: interchain account address has changed",
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
		{
			name: "failure: channel not found",
			malleate: func() {
				path.EndpointB.ChannelID = "invalid-channel-id"
			},
			expError: channeltypes.ErrChannelNotFound,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB, icatypes.EncodingProtobuf)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			currentMetadata, err := suite.chainB.GetSimApp().ICAHostKeeper.GetAppMetadata(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			suite.Require().NoError(err)

			order = channeltypes.ORDERED
			metadata = icatypes.NewDefaultMetadata(path.EndpointA.ConnectionID, path.EndpointB.ConnectionID)
			// use the same address as the previous metadata.
			metadata.Address = currentMetadata.Address

			// this is the actual change to the version.
			metadata.Encoding = icatypes.EncodingProto3JSON

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = string(icatypes.ModuleCdc.MustMarshalJSON(&metadata))

			err = path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			counterpartyVersion = path.EndpointA.GetChannel().Version

			tc.malleate() // malleate mutates test data

			version, err := suite.chainB.GetSimApp().ICAHostKeeper.OnChanUpgradeTry(
				suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID,
				path.EndpointB.ChannelID,
				order,
				[]string{path.EndpointB.ConnectionID},
				counterpartyVersion,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(path.EndpointB.GetChannel().Version, version)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}
