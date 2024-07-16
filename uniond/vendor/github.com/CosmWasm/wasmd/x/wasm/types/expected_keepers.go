package types

import (
	"context"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	distrtypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// BankViewKeeper defines a subset of methods implemented by the cosmos-sdk bank keeper
type BankViewKeeper interface {
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
	GetSupply(ctx context.Context, denom string) sdk.Coin
	GetDenomMetaData(ctx context.Context, denom string) (banktypes.Metadata, bool)
	DenomsMetadata(ctx context.Context, req *banktypes.QueryDenomsMetadataRequest) (*banktypes.QueryDenomsMetadataResponse, error)
}

// Burner is a subset of the sdk bank keeper methods
type Burner interface {
	BurnCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
}

// BankKeeper defines a subset of methods implemented by the cosmos-sdk bank keeper
type BankKeeper interface {
	BankViewKeeper
	Burner
	IsSendEnabledCoins(ctx context.Context, coins ...sdk.Coin) error
	BlockedAddr(addr sdk.AccAddress) bool
	SendCoins(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error
}

// AccountKeeper defines a subset of methods implemented by the cosmos-sdk account keeper
type AccountKeeper interface {
	// Return a new account with the next account number and the specified address. Does not save the new account to the store.
	NewAccountWithAddress(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	// Retrieve an account from the store.
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	// Set an account in the store.
	SetAccount(ctx context.Context, acc sdk.AccountI)
}

// DistributionKeeper defines a subset of methods implemented by the cosmos-sdk distribution keeper
type DistributionKeeper interface {
	DelegatorWithdrawAddress(c context.Context, req *distrtypes.QueryDelegatorWithdrawAddressRequest) (*distrtypes.QueryDelegatorWithdrawAddressResponse, error)
	DelegationRewards(c context.Context, req *distrtypes.QueryDelegationRewardsRequest) (*distrtypes.QueryDelegationRewardsResponse, error)
	DelegationTotalRewards(c context.Context, req *distrtypes.QueryDelegationTotalRewardsRequest) (*distrtypes.QueryDelegationTotalRewardsResponse, error)
	DelegatorValidators(c context.Context, req *distrtypes.QueryDelegatorValidatorsRequest) (*distrtypes.QueryDelegatorValidatorsResponse, error)
}

// StakingKeeper defines a subset of methods implemented by the cosmos-sdk staking keeper
type StakingKeeper interface {
	// BondDenom - Bondable coin denomination
	BondDenom(ctx context.Context) (string, error)
	// GetValidator get a single validator
	GetValidator(ctx context.Context, addr sdk.ValAddress) (validator stakingtypes.Validator, err error)
	// GetBondedValidatorsByPower get the current group of bonded validators sorted by power-rank
	GetBondedValidatorsByPower(ctx context.Context) ([]stakingtypes.Validator, error)
	// GetAllDelegatorDelegations return all delegations for a delegator
	GetAllDelegatorDelegations(ctx context.Context, delegator sdk.AccAddress) ([]stakingtypes.Delegation, error)
	// GetDelegation return a specific delegation
	GetDelegation(ctx context.Context,
		delAddr sdk.AccAddress, valAddr sdk.ValAddress) (stakingtypes.Delegation, error)
	// HasReceivingRedelegation check if validator is receiving a redelegation
	HasReceivingRedelegation(ctx context.Context,
		delAddr sdk.AccAddress, valDstAddr sdk.ValAddress) (bool, error)
}

// ChannelKeeper defines the expected IBC channel keeper
type ChannelKeeper interface {
	GetChannel(ctx sdk.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool)
	GetNextSequenceSend(ctx sdk.Context, portID, channelID string) (uint64, bool)
	ChanCloseInit(ctx sdk.Context, portID, channelID string, chanCap *capabilitytypes.Capability) error
	GetAllChannels(ctx sdk.Context) (channels []channeltypes.IdentifiedChannel)
	SetChannel(ctx sdk.Context, portID, channelID string, channel channeltypes.Channel)
	GetAllChannelsWithPortPrefix(ctx sdk.Context, portPrefix string) []channeltypes.IdentifiedChannel
}

// ICS4Wrapper defines the method for an IBC data package to be submitted.
// The interface is implemented by the channel keeper on the lowest level in ibc-go. Middlewares or other abstractions
// can add functionality on top of it. See ics4Wrapper in ibc-go.
// It is important to choose the right implementation that is configured for any middleware used in the ibc-stack of wasm.
//
// For example, when ics-29 fee middleware is set up for the wasm ibc-stack, then the IBCFeeKeeper should be used, so
// that they are in sync.
type ICS4Wrapper interface {
	// SendPacket is called by a module in order to send an IBC packet on a channel.
	// The packet sequence generated for the packet to be sent is returned. An error
	// is returned if one occurs.
	SendPacket(
		ctx sdk.Context,
		channelCap *capabilitytypes.Capability,
		sourcePort string,
		sourceChannel string,
		timeoutHeight clienttypes.Height,
		timeoutTimestamp uint64,
		data []byte,
	) (uint64, error)

	WriteAcknowledgement(
		ctx sdk.Context,
		chanCap *capabilitytypes.Capability,
		packet ibcexported.PacketI,
		acknowledgement ibcexported.Acknowledgement,
	) error
}

// ClientKeeper defines the expected IBC client keeper
type ClientKeeper interface {
	GetClientConsensusState(ctx sdk.Context, clientID string) (connection ibcexported.ConsensusState, found bool)
}

// ConnectionKeeper defines the expected IBC connection keeper
type ConnectionKeeper interface {
	GetConnection(ctx sdk.Context, connectionID string) (connection connectiontypes.ConnectionEnd, found bool)
}

// PortKeeper defines the expected IBC port keeper
type PortKeeper interface {
	BindPort(ctx sdk.Context, portID string) *capabilitytypes.Capability
}

type CapabilityKeeper interface {
	GetCapability(ctx sdk.Context, name string) (*capabilitytypes.Capability, bool)
	ClaimCapability(ctx sdk.Context, cap *capabilitytypes.Capability, name string) error
	AuthenticateCapability(ctx sdk.Context, capability *capabilitytypes.Capability, name string) bool
}

// ICS20TransferPortSource is a subset of the ibc transfer keeper.
type ICS20TransferPortSource interface {
	GetPort(ctx sdk.Context) string
}
