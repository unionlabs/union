/*
This file contains the variables, constants, and default values
used in the testing package and commonly defined in tests.
*/
package ibctesting

import (
	"time"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	"github.com/cosmos/ibc-go/v8/testing/mock"
	"github.com/cosmos/ibc-go/v8/testing/simapp"
)

const (
	FirstClientID     = "07-tendermint-0"
	SecondClientID    = "07-tendermint-1"
	FirstChannelID    = "channel-0"
	FirstConnectionID = "connection-0"

	// Default params constants used to create a TM client
	TrustingPeriod     time.Duration = time.Hour * 24 * 7 * 2
	UnbondingPeriod    time.Duration = time.Hour * 24 * 7 * 3
	MaxClockDrift      time.Duration = time.Second * 10
	DefaultDelayPeriod uint64        = 0

	DefaultChannelVersion = mock.Version
	InvalidID             = "IDisInvalid"

	// Application Ports
	TransferPort = ibctransfertypes.ModuleName
	MockPort     = mock.ModuleName
	MockFeePort  = simapp.MockFeePort

	// used for testing proposals
	Title       = "title"
	Description = "description"

	// character set used for generating a random string in GenerateString
	charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
)

var (
	DefaultOpenInitVersion *connectiontypes.Version

	// DefaultTrustLevel sets params variables used to create a TM client
	DefaultTrustLevel = ibctm.DefaultTrustLevel

	TestAccAddress = "cosmos17dtl0mjt3t77kpuhg2edqzjpszulwhgzuj9ljs"
	TestCoin       = sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
	TestCoins      = sdk.NewCoins(TestCoin)

	UpgradePath = []string{"upgrade", "upgradedIBCState"}

	ConnectionVersion = connectiontypes.GetCompatibleVersions()[0]

	MockAcknowledgement          = mock.MockAcknowledgement.Acknowledgement()
	MockPacketData               = mock.MockPacketData
	MockFailPacketData           = mock.MockFailPacketData
	MockRecvCanaryCapabilityName = mock.MockRecvCanaryCapabilityName

	prefix = commitmenttypes.NewMerklePrefix([]byte("ibc"))
)
