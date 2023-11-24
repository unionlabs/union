package keeper

import (
	"context"

	metrics "github.com/hashicorp/go-metrics"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
)

var (
	_ clienttypes.MsgServer     = (*Keeper)(nil)
	_ connectiontypes.MsgServer = (*Keeper)(nil)
	_ channeltypes.MsgServer    = (*Keeper)(nil)
)

// CreateClient defines a rpc handler method for MsgCreateClient.
func (k Keeper) CreateClient(goCtx context.Context, msg *clienttypes.MsgCreateClient) (*clienttypes.MsgCreateClientResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	clientState, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return nil, err
	}

	consensusState, err := clienttypes.UnpackConsensusState(msg.ConsensusState)
	if err != nil {
		return nil, err
	}

	if _, err = k.ClientKeeper.CreateClient(ctx, clientState, consensusState); err != nil {
		return nil, err
	}

	return &clienttypes.MsgCreateClientResponse{}, nil
}

// UpdateClient defines a rpc handler method for MsgUpdateClient.
func (k Keeper) UpdateClient(goCtx context.Context, msg *clienttypes.MsgUpdateClient) (*clienttypes.MsgUpdateClientResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	clientMsg, err := clienttypes.UnpackClientMessage(msg.ClientMessage)
	if err != nil {
		return nil, err
	}

	if err = k.ClientKeeper.UpdateClient(ctx, msg.ClientId, clientMsg); err != nil {
		return nil, err
	}

	return &clienttypes.MsgUpdateClientResponse{}, nil
}

// UpgradeClient defines a rpc handler method for MsgUpgradeClient.
func (k Keeper) UpgradeClient(goCtx context.Context, msg *clienttypes.MsgUpgradeClient) (*clienttypes.MsgUpgradeClientResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	upgradedClient, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return nil, err
	}
	upgradedConsState, err := clienttypes.UnpackConsensusState(msg.ConsensusState)
	if err != nil {
		return nil, err
	}

	if err = k.ClientKeeper.UpgradeClient(ctx, msg.ClientId, upgradedClient, upgradedConsState,
		msg.ProofUpgradeClient, msg.ProofUpgradeConsensusState); err != nil {
		return nil, err
	}

	return &clienttypes.MsgUpgradeClientResponse{}, nil
}

// SubmitMisbehaviour defines a rpc handler method for MsgSubmitMisbehaviour.
// Warning: DEPRECATED
// This handler is redudant as `MsgUpdateClient` is now capable of handling both a Header and a Misbehaviour
func (k Keeper) SubmitMisbehaviour(goCtx context.Context, msg *clienttypes.MsgSubmitMisbehaviour) (*clienttypes.MsgSubmitMisbehaviourResponse, error) { //nolint:staticcheck // for now, we're using msgsubmitmisbehaviour.
	ctx := sdk.UnwrapSDKContext(goCtx)

	misbehaviour, err := clienttypes.UnpackClientMessage(msg.Misbehaviour)
	if err != nil {
		return nil, err
	}

	if err = k.ClientKeeper.UpdateClient(ctx, msg.ClientId, misbehaviour); err != nil {
		return nil, err
	}

	return &clienttypes.MsgSubmitMisbehaviourResponse{}, nil
}

// RecoverClient defines a rpc handler method for MsgRecoverClient.
func (k Keeper) RecoverClient(goCtx context.Context, msg *clienttypes.MsgRecoverClient) (*clienttypes.MsgRecoverClientResponse, error) {
	if k.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", k.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.ClientKeeper.RecoverClient(ctx, msg.SubjectClientId, msg.SubstituteClientId); err != nil {
		return nil, errorsmod.Wrap(err, "client recovery failed")
	}

	return &clienttypes.MsgRecoverClientResponse{}, nil
}

// IBCSoftwareUpgrade defines a rpc handler method for MsgIBCSoftwareUpgrade.
func (k Keeper) IBCSoftwareUpgrade(goCtx context.Context, msg *clienttypes.MsgIBCSoftwareUpgrade) (*clienttypes.MsgIBCSoftwareUpgradeResponse, error) {
	if k.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", k.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	upgradedClientState, err := clienttypes.UnpackClientState(msg.UpgradedClientState)
	if err != nil {
		return nil, errorsmod.Wrapf(clienttypes.ErrInvalidClientType, "cannot unpack client state: %s", err)
	}

	if err = k.ClientKeeper.ScheduleIBCSoftwareUpgrade(ctx, msg.Plan, upgradedClientState); err != nil {
		return nil, errorsmod.Wrap(err, "failed to schedule upgrade")
	}

	return &clienttypes.MsgIBCSoftwareUpgradeResponse{}, nil
}

// ConnectionOpenInit defines a rpc handler method for MsgConnectionOpenInit.
func (k Keeper) ConnectionOpenInit(goCtx context.Context, msg *connectiontypes.MsgConnectionOpenInit) (*connectiontypes.MsgConnectionOpenInitResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	if _, err := k.ConnectionKeeper.ConnOpenInit(ctx, msg.ClientId, msg.Counterparty, msg.Version, msg.DelayPeriod); err != nil {
		return nil, errorsmod.Wrap(err, "connection handshake open init failed")
	}

	return &connectiontypes.MsgConnectionOpenInitResponse{}, nil
}

// ConnectionOpenTry defines a rpc handler method for MsgConnectionOpenTry.
func (k Keeper) ConnectionOpenTry(goCtx context.Context, msg *connectiontypes.MsgConnectionOpenTry) (*connectiontypes.MsgConnectionOpenTryResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	targetClient, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return nil, err
	}

	if _, err := k.ConnectionKeeper.ConnOpenTry(
		ctx, msg.Counterparty, msg.DelayPeriod, msg.ClientId, targetClient,
		msg.CounterpartyVersions, msg.ProofInit, msg.ProofClient, msg.ProofConsensus,
		msg.ProofHeight, msg.ConsensusHeight,
	); err != nil {
		return nil, errorsmod.Wrap(err, "connection handshake open try failed")
	}

	return &connectiontypes.MsgConnectionOpenTryResponse{}, nil
}

// ConnectionOpenAck defines a rpc handler method for MsgConnectionOpenAck.
func (k Keeper) ConnectionOpenAck(goCtx context.Context, msg *connectiontypes.MsgConnectionOpenAck) (*connectiontypes.MsgConnectionOpenAckResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	targetClient, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return nil, err
	}

	if err := k.ConnectionKeeper.ConnOpenAck(
		ctx, msg.ConnectionId, targetClient, msg.Version, msg.CounterpartyConnectionId,
		msg.ProofTry, msg.ProofClient, msg.ProofConsensus,
		msg.ProofHeight, msg.ConsensusHeight,
	); err != nil {
		return nil, errorsmod.Wrap(err, "connection handshake open ack failed")
	}

	return &connectiontypes.MsgConnectionOpenAckResponse{}, nil
}

// ConnectionOpenConfirm defines a rpc handler method for MsgConnectionOpenConfirm.
func (k Keeper) ConnectionOpenConfirm(goCtx context.Context, msg *connectiontypes.MsgConnectionOpenConfirm) (*connectiontypes.MsgConnectionOpenConfirmResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	if err := k.ConnectionKeeper.ConnOpenConfirm(
		ctx, msg.ConnectionId, msg.ProofAck, msg.ProofHeight,
	); err != nil {
		return nil, errorsmod.Wrap(err, "connection handshake open confirm failed")
	}

	return &connectiontypes.MsgConnectionOpenConfirmResponse{}, nil
}

// ChannelOpenInit defines a rpc handler method for MsgChannelOpenInit.
// ChannelOpenInit will perform 04-channel checks, route to the application
// callback, and write an OpenInit channel into state upon successful execution.
func (k Keeper) ChannelOpenInit(goCtx context.Context, msg *channeltypes.MsgChannelOpenInit) (*channeltypes.MsgChannelOpenInitResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by port capability
	module, portCap, err := k.PortKeeper.LookupModuleByPort(ctx, msg.PortId)
	if err != nil {
		ctx.Logger().Error("channel open init failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve application callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel open init failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform 04-channel verification
	channelID, capability, err := k.ChannelKeeper.ChanOpenInit(
		ctx, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.PortId,
		portCap, msg.Channel.Counterparty, msg.Channel.Version,
	)
	if err != nil {
		ctx.Logger().Error("channel open init failed", "error", errorsmod.Wrap(err, "channel handshake open init failed"))
		return nil, errorsmod.Wrap(err, "channel handshake open init failed")
	}

	// Perform application logic callback
	version, err := cbs.OnChanOpenInit(ctx, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.PortId, channelID, capability, msg.Channel.Counterparty, msg.Channel.Version)
	if err != nil {
		ctx.Logger().Error("channel open init failed", "port-id", msg.PortId, "channel-id", channelID, "error", errorsmod.Wrap(err, "channel open init callback failed"))
		return nil, errorsmod.Wrapf(err, "channel open init callback failed for port ID: %s, channel ID: %s", msg.PortId, channelID)
	}

	// Write channel into state
	k.ChannelKeeper.WriteOpenInitChannel(ctx, msg.PortId, channelID, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.Channel.Counterparty, version)

	ctx.Logger().Info("channel open init succeeded", "channel-id", channelID, "version", version)

	return &channeltypes.MsgChannelOpenInitResponse{
		ChannelId: channelID,
		Version:   version,
	}, nil
}

// ChannelOpenTry defines a rpc handler method for MsgChannelOpenTry.
// ChannelOpenTry will perform 04-channel checks, route to the application
// callback, and write an OpenTry channel into state upon successful execution.
func (k Keeper) ChannelOpenTry(goCtx context.Context, msg *channeltypes.MsgChannelOpenTry) (*channeltypes.MsgChannelOpenTryResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by port capability
	module, portCap, err := k.PortKeeper.LookupModuleByPort(ctx, msg.PortId)
	if err != nil {
		ctx.Logger().Error("channel open try failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve application callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel open try failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform 04-channel verification
	channelID, capability, err := k.ChannelKeeper.ChanOpenTry(ctx, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.PortId,
		portCap, msg.Channel.Counterparty, msg.CounterpartyVersion, msg.ProofInit, msg.ProofHeight,
	)
	if err != nil {
		ctx.Logger().Error("channel open try failed", "error", errorsmod.Wrap(err, "channel handshake open try failed"))
		return nil, errorsmod.Wrap(err, "channel handshake open try failed")
	}

	// Perform application logic callback
	version, err := cbs.OnChanOpenTry(ctx, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.PortId, channelID, capability, msg.Channel.Counterparty, msg.CounterpartyVersion)
	if err != nil {
		ctx.Logger().Error("channel open try failed", "port-id", msg.PortId, "channel-id", channelID, "error", errorsmod.Wrap(err, "channel open try callback failed"))
		return nil, errorsmod.Wrapf(err, "channel open try callback failed for port ID: %s, channel ID: %s", msg.PortId, channelID)
	}

	// Write channel into state
	k.ChannelKeeper.WriteOpenTryChannel(ctx, msg.PortId, channelID, msg.Channel.Ordering, msg.Channel.ConnectionHops, msg.Channel.Counterparty, version)

	ctx.Logger().Info("channel open try succeeded", "channel-id", channelID, "port-id", msg.PortId, "version", version)

	return &channeltypes.MsgChannelOpenTryResponse{
		ChannelId: channelID,
		Version:   version,
	}, nil
}

// ChannelOpenAck defines a rpc handler method for MsgChannelOpenAck.
// ChannelOpenAck will perform 04-channel checks, route to the application
// callback, and write an OpenAck channel into state upon successful execution.
func (k Keeper) ChannelOpenAck(goCtx context.Context, msg *channeltypes.MsgChannelOpenAck) (*channeltypes.MsgChannelOpenAckResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.PortId, msg.ChannelId)
	if err != nil {
		ctx.Logger().Error("channel open ack failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve application callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel open ack failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform 04-channel verification
	if err = k.ChannelKeeper.ChanOpenAck(
		ctx, msg.PortId, msg.ChannelId, capability, msg.CounterpartyVersion, msg.CounterpartyChannelId, msg.ProofTry, msg.ProofHeight,
	); err != nil {
		ctx.Logger().Error("channel open ack failed", "error", err.Error())
		return nil, errorsmod.Wrap(err, "channel handshake open ack failed")
	}

	// Write channel into state
	k.ChannelKeeper.WriteOpenAckChannel(ctx, msg.PortId, msg.ChannelId, msg.CounterpartyVersion, msg.CounterpartyChannelId)

	// Perform application logic callback
	if err = cbs.OnChanOpenAck(ctx, msg.PortId, msg.ChannelId, msg.CounterpartyChannelId, msg.CounterpartyVersion); err != nil {
		ctx.Logger().Error("channel open ack failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", errorsmod.Wrap(err, "channel open ack callback failed"))
		return nil, errorsmod.Wrapf(err, "channel open ack callback failed for port ID: %s, channel ID: %s", msg.PortId, msg.ChannelId)
	}

	ctx.Logger().Info("channel open ack succeeded", "channel-id", msg.ChannelId, "port-id", msg.PortId)

	return &channeltypes.MsgChannelOpenAckResponse{}, nil
}

// ChannelOpenConfirm defines a rpc handler method for MsgChannelOpenConfirm.
// ChannelOpenConfirm will perform 04-channel checks, route to the application
// callback, and write an OpenConfirm channel into state upon successful execution.
func (k Keeper) ChannelOpenConfirm(goCtx context.Context, msg *channeltypes.MsgChannelOpenConfirm) (*channeltypes.MsgChannelOpenConfirmResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.PortId, msg.ChannelId)
	if err != nil {
		ctx.Logger().Error("channel open confirm failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve application callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel open confirm failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform 04-channel verification
	if err = k.ChannelKeeper.ChanOpenConfirm(ctx, msg.PortId, msg.ChannelId, capability, msg.ProofAck, msg.ProofHeight); err != nil {
		ctx.Logger().Error("channel open confirm failed", "error", errorsmod.Wrap(err, "channel handshake open confirm failed"))
		return nil, errorsmod.Wrap(err, "channel handshake open confirm failed")
	}

	// Write channel into state
	k.ChannelKeeper.WriteOpenConfirmChannel(ctx, msg.PortId, msg.ChannelId)

	// Perform application logic callback
	if err = cbs.OnChanOpenConfirm(ctx, msg.PortId, msg.ChannelId); err != nil {
		ctx.Logger().Error("channel open confirm failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", errorsmod.Wrap(err, "channel open confirm callback failed"))
		return nil, errorsmod.Wrapf(err, "channel open confirm callback failed for port ID: %s, channel ID: %s", msg.PortId, msg.ChannelId)
	}

	ctx.Logger().Info("channel open confirm succeeded", "channel-id", msg.ChannelId, "port-id", msg.PortId)

	return &channeltypes.MsgChannelOpenConfirmResponse{}, nil
}

// ChannelCloseInit defines a rpc handler method for MsgChannelCloseInit.
func (k Keeper) ChannelCloseInit(goCtx context.Context, msg *channeltypes.MsgChannelCloseInit) (*channeltypes.MsgChannelCloseInitResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.PortId, msg.ChannelId)
	if err != nil {
		ctx.Logger().Error("channel close init failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel close init failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	if err = cbs.OnChanCloseInit(ctx, msg.PortId, msg.ChannelId); err != nil {
		ctx.Logger().Error("channel close init failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", errorsmod.Wrap(err, "channel close init callback failed"))
		return nil, errorsmod.Wrapf(err, "channel close init callback failed for port ID: %s, channel ID: %s", msg.PortId, msg.ChannelId)
	}

	err = k.ChannelKeeper.ChanCloseInit(ctx, msg.PortId, msg.ChannelId, capability)
	if err != nil {
		ctx.Logger().Error("channel close init failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", err.Error())
		return nil, errorsmod.Wrap(err, "channel handshake close init failed")
	}

	ctx.Logger().Info("channel close init succeeded", "channel-id", msg.ChannelId, "port-id", msg.PortId)

	return &channeltypes.MsgChannelCloseInitResponse{}, nil
}

// ChannelCloseConfirm defines a rpc handler method for MsgChannelCloseConfirm.
func (k Keeper) ChannelCloseConfirm(goCtx context.Context, msg *channeltypes.MsgChannelCloseConfirm) (*channeltypes.MsgChannelCloseConfirmResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.PortId, msg.ChannelId)
	if err != nil {
		ctx.Logger().Error("channel close confirm failed", "port-id", msg.PortId, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("channel close confirm failed", "port-id", msg.PortId, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	if err = cbs.OnChanCloseConfirm(ctx, msg.PortId, msg.ChannelId); err != nil {
		ctx.Logger().Error("channel close confirm failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", errorsmod.Wrap(err, "channel close confirm callback failed"))
		return nil, errorsmod.Wrapf(err, "channel close confirm callback failed for port ID: %s, channel ID: %s", msg.PortId, msg.ChannelId)
	}

	err = k.ChannelKeeper.ChanCloseConfirm(ctx, msg.PortId, msg.ChannelId, capability, msg.ProofInit, msg.ProofHeight)
	if err != nil {
		ctx.Logger().Error("channel close confirm failed", "port-id", msg.PortId, "channel-id", msg.ChannelId, "error", err.Error())
		return nil, errorsmod.Wrap(err, "channel handshake close confirm failed")
	}

	ctx.Logger().Info("channel close confirm succeeded", "channel-id", msg.ChannelId, "port-id", msg.PortId)

	return &channeltypes.MsgChannelCloseConfirmResponse{}, nil
}

// RecvPacket defines a rpc handler method for MsgRecvPacket.
func (k Keeper) RecvPacket(goCtx context.Context, msg *channeltypes.MsgRecvPacket) (*channeltypes.MsgRecvPacketResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	relayer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		ctx.Logger().Error("receive packet failed", "error", errorsmod.Wrap(err, "Invalid address for msg Signer"))
		return nil, errorsmod.Wrap(err, "Invalid address for msg Signer")
	}

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.Packet.DestinationPort, msg.Packet.DestinationChannel)
	if err != nil {
		ctx.Logger().Error("receive packet failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("receive packet failed", "port-id", msg.Packet.SourcePort, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform TAO verification
	//
	// If the packet was already received, perform a no-op
	// Use a cached context to prevent accidental state changes
	cacheCtx, writeFn := ctx.CacheContext()
	err = k.ChannelKeeper.RecvPacket(cacheCtx, capability, msg.Packet, msg.ProofCommitment, msg.ProofHeight)

	switch err {
	case nil:
		writeFn()
	case channeltypes.ErrNoOpMsg:
		// no-ops do not need event emission as they will be ignored
		ctx.Logger().Debug("no-op on redundant relay", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel)
		return &channeltypes.MsgRecvPacketResponse{Result: channeltypes.NOOP}, nil
	default:
		ctx.Logger().Error("receive packet failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "receive packet verification failed"))
		return nil, errorsmod.Wrap(err, "receive packet verification failed")
	}

	// Perform application logic callback
	//
	// Cache context so that we may discard state changes from callback if the acknowledgement is unsuccessful.
	cacheCtx, writeFn = ctx.CacheContext()
	ack := cbs.OnRecvPacket(cacheCtx, msg.Packet, relayer)
	if ack == nil || ack.Success() {
		// write application state changes for asynchronous and successful acknowledgements
		writeFn()
	}

	// Set packet acknowledgement only if the acknowledgement is not nil.
	// NOTE: IBC applications modules may call the WriteAcknowledgement asynchronously if the
	// acknowledgement is nil.
	if ack != nil {
		if err := k.ChannelKeeper.WriteAcknowledgement(ctx, capability, msg.Packet, ack); err != nil {
			return nil, err
		}
	}

	defer telemetry.IncrCounterWithLabels(
		[]string{"tx", "msg", "ibc", channeltypes.EventTypeRecvPacket},
		1,
		[]metrics.Label{
			telemetry.NewLabel(coretypes.LabelSourcePort, msg.Packet.SourcePort),
			telemetry.NewLabel(coretypes.LabelSourceChannel, msg.Packet.SourceChannel),
			telemetry.NewLabel(coretypes.LabelDestinationPort, msg.Packet.DestinationPort),
			telemetry.NewLabel(coretypes.LabelDestinationChannel, msg.Packet.DestinationChannel),
		},
	)

	ctx.Logger().Info("receive packet callback succeeded", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "result", channeltypes.SUCCESS.String())

	return &channeltypes.MsgRecvPacketResponse{Result: channeltypes.SUCCESS}, nil
}

// Timeout defines a rpc handler method for MsgTimeout.
func (k Keeper) Timeout(goCtx context.Context, msg *channeltypes.MsgTimeout) (*channeltypes.MsgTimeoutResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	relayer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		ctx.Logger().Error("timeout failed", "error", errorsmod.Wrap(err, "Invalid address for msg Signer"))
		return nil, errorsmod.Wrap(err, "Invalid address for msg Signer")
	}

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.Packet.SourcePort, msg.Packet.SourceChannel)
	if err != nil {
		ctx.Logger().Error("timeout failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("timeout failed", "port-id", msg.Packet.SourcePort, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform TAO verification
	//
	// If the timeout was already received, perform a no-op
	// Use a cached context to prevent accidental state changes
	cacheCtx, writeFn := ctx.CacheContext()
	err = k.ChannelKeeper.TimeoutPacket(cacheCtx, msg.Packet, msg.ProofUnreceived, msg.ProofHeight, msg.NextSequenceRecv)

	switch err {
	case nil:
		writeFn()
	case channeltypes.ErrNoOpMsg:
		// no-ops do not need event emission as they will be ignored
		ctx.Logger().Debug("no-op on redundant relay", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel)
		return &channeltypes.MsgTimeoutResponse{Result: channeltypes.NOOP}, nil
	default:
		ctx.Logger().Error("timeout failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "timeout packet verification failed"))
		return nil, errorsmod.Wrap(err, "timeout packet verification failed")
	}

	// Perform application logic callback
	err = cbs.OnTimeoutPacket(ctx, msg.Packet, relayer)
	if err != nil {
		ctx.Logger().Error("timeout failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "timeout packet callback failed"))
		return nil, errorsmod.Wrap(err, "timeout packet callback failed")
	}

	// Delete packet commitment
	if err = k.ChannelKeeper.TimeoutExecuted(ctx, capability, msg.Packet); err != nil {
		return nil, err
	}

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "timeout", "packet"},
		1,
		[]metrics.Label{
			telemetry.NewLabel(coretypes.LabelSourcePort, msg.Packet.SourcePort),
			telemetry.NewLabel(coretypes.LabelSourceChannel, msg.Packet.SourceChannel),
			telemetry.NewLabel(coretypes.LabelDestinationPort, msg.Packet.DestinationPort),
			telemetry.NewLabel(coretypes.LabelDestinationChannel, msg.Packet.DestinationChannel),
			telemetry.NewLabel(coretypes.LabelTimeoutType, "height"),
		},
	)

	ctx.Logger().Info("timeout packet callback succeeded", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "result", channeltypes.SUCCESS.String())

	return &channeltypes.MsgTimeoutResponse{Result: channeltypes.SUCCESS}, nil
}

// TimeoutOnClose defines a rpc handler method for MsgTimeoutOnClose.
func (k Keeper) TimeoutOnClose(goCtx context.Context, msg *channeltypes.MsgTimeoutOnClose) (*channeltypes.MsgTimeoutOnCloseResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	relayer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		ctx.Logger().Error("timeout on close failed", "error", errorsmod.Wrap(err, "Invalid address for msg Signer"))
		return nil, errorsmod.Wrap(err, "Invalid address for msg Signer")
	}

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.Packet.SourcePort, msg.Packet.SourceChannel)
	if err != nil {
		ctx.Logger().Error("timeout on close failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("timeout on close failed", "port-id", msg.Packet.SourcePort, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform TAO verification
	//
	// If the timeout was already received, perform a no-op
	// Use a cached context to prevent accidental state changes
	cacheCtx, writeFn := ctx.CacheContext()
	err = k.ChannelKeeper.TimeoutOnClose(cacheCtx, capability, msg.Packet, msg.ProofUnreceived, msg.ProofClose, msg.ProofHeight, msg.NextSequenceRecv)

	switch err {
	case nil:
		writeFn()
	case channeltypes.ErrNoOpMsg:
		// no-ops do not need event emission as they will be ignored
		ctx.Logger().Debug("no-op on redundant relay", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel)
		return &channeltypes.MsgTimeoutOnCloseResponse{Result: channeltypes.NOOP}, nil
	default:
		ctx.Logger().Error("timeout on close failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "timeout on close packet verification failed"))
		return nil, errorsmod.Wrap(err, "timeout on close packet verification failed")
	}

	// Perform application logic callback
	//
	// NOTE: MsgTimeout and MsgTimeoutOnClose use the same "OnTimeoutPacket"
	// application logic callback.
	err = cbs.OnTimeoutPacket(ctx, msg.Packet, relayer)
	if err != nil {
		ctx.Logger().Error("timeout on close failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "timeout on close callback failed"))
		return nil, errorsmod.Wrap(err, "timeout on close callback failed")
	}

	// Delete packet commitment
	if err = k.ChannelKeeper.TimeoutExecuted(ctx, capability, msg.Packet); err != nil {
		return nil, err
	}

	defer telemetry.IncrCounterWithLabels(
		[]string{"ibc", "timeout", "packet"},
		1,
		[]metrics.Label{
			telemetry.NewLabel(coretypes.LabelSourcePort, msg.Packet.SourcePort),
			telemetry.NewLabel(coretypes.LabelSourceChannel, msg.Packet.SourceChannel),
			telemetry.NewLabel(coretypes.LabelDestinationPort, msg.Packet.DestinationPort),
			telemetry.NewLabel(coretypes.LabelDestinationChannel, msg.Packet.DestinationChannel),
			telemetry.NewLabel(coretypes.LabelTimeoutType, "channel-closed"),
		},
	)

	ctx.Logger().Info("timeout on close callback succeeded", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "result", channeltypes.SUCCESS.String())

	return &channeltypes.MsgTimeoutOnCloseResponse{Result: channeltypes.SUCCESS}, nil
}

// Acknowledgement defines a rpc handler method for MsgAcknowledgement.
func (k Keeper) Acknowledgement(goCtx context.Context, msg *channeltypes.MsgAcknowledgement) (*channeltypes.MsgAcknowledgementResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	relayer, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		ctx.Logger().Error("acknowledgement failed", "error", errorsmod.Wrap(err, "Invalid address for msg Signer"))
		return nil, errorsmod.Wrap(err, "Invalid address for msg Signer")
	}

	// Lookup module by channel capability
	module, capability, err := k.ChannelKeeper.LookupModuleByChannel(ctx, msg.Packet.SourcePort, msg.Packet.SourceChannel)
	if err != nil {
		ctx.Logger().Error("acknowledgement failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "could not retrieve module from port-id"))
		return nil, errorsmod.Wrap(err, "could not retrieve module from port-id")
	}

	// Retrieve callbacks from router
	cbs, ok := k.Router.GetRoute(module)
	if !ok {
		ctx.Logger().Error("acknowledgement failed", "port-id", msg.Packet.SourcePort, "error", errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module))
		return nil, errorsmod.Wrapf(porttypes.ErrInvalidRoute, "route not found to module: %s", module)
	}

	// Perform TAO verification
	//
	// If the acknowledgement was already received, perform a no-op
	// Use a cached context to prevent accidental state changes
	cacheCtx, writeFn := ctx.CacheContext()
	err = k.ChannelKeeper.AcknowledgePacket(cacheCtx, capability, msg.Packet, msg.Acknowledgement, msg.ProofAcked, msg.ProofHeight)

	switch err {
	case nil:
		writeFn()
	case channeltypes.ErrNoOpMsg:
		// no-ops do not need event emission as they will be ignored
		ctx.Logger().Debug("no-op on redundant relay", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel)
		return &channeltypes.MsgAcknowledgementResponse{Result: channeltypes.NOOP}, nil
	default:
		ctx.Logger().Error("acknowledgement failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "acknowledge packet verification failed"))
		return nil, errorsmod.Wrap(err, "acknowledge packet verification failed")
	}

	// Perform application logic callback
	err = cbs.OnAcknowledgementPacket(ctx, msg.Packet, msg.Acknowledgement, relayer)
	if err != nil {
		ctx.Logger().Error("acknowledgement failed", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "error", errorsmod.Wrap(err, "acknowledge packet callback failed"))
		return nil, errorsmod.Wrap(err, "acknowledge packet callback failed")
	}

	defer telemetry.IncrCounterWithLabels(
		[]string{"tx", "msg", "ibc", channeltypes.EventTypeAcknowledgePacket},
		1,
		[]metrics.Label{
			telemetry.NewLabel(coretypes.LabelSourcePort, msg.Packet.SourcePort),
			telemetry.NewLabel(coretypes.LabelSourceChannel, msg.Packet.SourceChannel),
			telemetry.NewLabel(coretypes.LabelDestinationPort, msg.Packet.DestinationPort),
			telemetry.NewLabel(coretypes.LabelDestinationChannel, msg.Packet.DestinationChannel),
		},
	)

	ctx.Logger().Info("acknowledgement succeeded", "port-id", msg.Packet.SourcePort, "channel-id", msg.Packet.SourceChannel, "result", channeltypes.SUCCESS.String())

	return &channeltypes.MsgAcknowledgementResponse{Result: channeltypes.SUCCESS}, nil
}

// UpdateClientParams defines a rpc handler method for MsgUpdateParams.
func (k Keeper) UpdateClientParams(goCtx context.Context, msg *clienttypes.MsgUpdateParams) (*clienttypes.MsgUpdateParamsResponse, error) {
	if k.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", k.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	k.ClientKeeper.SetParams(ctx, msg.Params)

	return &clienttypes.MsgUpdateParamsResponse{}, nil
}

// UpdateConnectionParams defines a rpc handler method for MsgUpdateParams for the 03-connection submodule.
func (k Keeper) UpdateConnectionParams(goCtx context.Context, msg *connectiontypes.MsgUpdateParams) (*connectiontypes.MsgUpdateParamsResponse, error) {
	if k.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", k.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	k.ConnectionKeeper.SetParams(ctx, msg.Params)

	return &connectiontypes.MsgUpdateParamsResponse{}, nil
}
