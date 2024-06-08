package bindings

import (
	"encoding/json"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	bindingstypes "union/x/deferredack/bindings/types"
	deferredackkeeper "union/x/deferredack/keeper"
	deferredacktypes "union/x/deferredack/types"
)

// CustomMessageDecorator returns decorator for custom CosmWasm bindings messages
func CustomMessageDecorator(bank *bankkeeper.BaseKeeper, deferredAck *deferredackkeeper.Keeper) func(wasmkeeper.Messenger) wasmkeeper.Messenger {
	return func(old wasmkeeper.Messenger) wasmkeeper.Messenger {
		return &CustomMessenger{
			wrapped:     old,
			bank:        bank,
			deferredAck: deferredAck,
		}
	}
}

type CustomMessenger struct {
	wrapped     wasmkeeper.Messenger
	bank        *bankkeeper.BaseKeeper
	deferredAck *deferredackkeeper.Keeper
}

var _ wasmkeeper.Messenger = (*CustomMessenger)(nil)

// DispatchMsg executes on the contractMsg.
func (m *CustomMessenger) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	if msg.Custom != nil {
		// only handle the happy path where this is really creating / minting / swapping ...
		// leave everything else for the wrapped version
		var contractMsg bindingstypes.DeferredAckMsg
		if err := json.Unmarshal(msg.Custom, &contractMsg); err != nil {
			return nil, nil, nil, errorsmod.Wrap(err, "deferred ack msg")
		}

		if contractMsg.WriteDeferredAck != nil {
			return m.WriteDeferredAck(ctx, contractAddr, contractMsg.WriteDeferredAck)
		}
	}
	return m.wrapped.DispatchMsg(ctx, contractAddr, contractIBCPortID, msg)
}

// createDenom creates a new token denom
func (m *CustomMessenger) WriteDeferredAck(ctx sdk.Context, contractAddr sdk.AccAddress, writeDeferredAck *bindingstypes.WriteDeferredAck) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	bz, err := PerformWriteDeferredAck(m.deferredAck, ctx, writeDeferredAck)

	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "failed to process response")
	}

	return nil, [][]byte{bz}, nil, nil
}

func PerformWriteDeferredAck(deferredAckKeeper *deferredackkeeper.Keeper, ctx sdk.Context, writeDeferredAck *bindingstypes.WriteDeferredAck) ([]byte, error) {
	if writeDeferredAck == nil {
		return nil, wasmvmtypes.InvalidRequest{Err: "write deferred ack request cannot be nil"}
	}

	msgServer := deferredackkeeper.NewMsgServerImpl(*deferredAckKeeper)

	msgWriteDeferredAck := deferredacktypes.NewMsgWriteDeferredAck(writeDeferredAck.DeferredPacketInfo, writeDeferredAck.Ack)

	res, err := msgServer.WriteDeferredAck(ctx, msgWriteDeferredAck)

	if err != nil {
		return nil, errorsmod.Wrap(err, "failed to execute msg write deferred ack")
	}

	return res.Marshal()
}
