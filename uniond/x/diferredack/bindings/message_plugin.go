package bindings

import (
	"encoding/json"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	bindingstypes "union/x/diferredack/bindings/types"
	diferredackkeeper "union/x/diferredack/keeper"
	diferredacktypes "union/x/diferredack/types"
)

// CustomMessageDecorator returns decorator for custom CosmWasm bindings messages
func CustomMessageDecorator(bank *bankkeeper.BaseKeeper, diferredAck *diferredackkeeper.Keeper) func(wasmkeeper.Messenger) wasmkeeper.Messenger {
	return func(old wasmkeeper.Messenger) wasmkeeper.Messenger {
		return &CustomMessenger{
			wrapped:     old,
			bank:        bank,
			diferredAck: diferredAck,
		}
	}
}

type CustomMessenger struct {
	wrapped     wasmkeeper.Messenger
	bank        *bankkeeper.BaseKeeper
	diferredAck *diferredackkeeper.Keeper
}

var _ wasmkeeper.Messenger = (*CustomMessenger)(nil)

// DispatchMsg executes on the contractMsg.
func (m *CustomMessenger) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	if msg.Custom != nil {
		// only handle the happy path where this is really creating / minting / swapping ...
		// leave everything else for the wrapped version
		var contractMsg bindingstypes.DiferredAckMsg
		if err := json.Unmarshal(msg.Custom, &contractMsg); err != nil {
			return nil, nil, nil, errorsmod.Wrap(err, "diferred ack msg")
		}

		if contractMsg.WriteDiferredAck != nil {
			return m.WriteDiferredAck(ctx, contractAddr, contractMsg.WriteDiferredAck)
		}
	}
	return m.wrapped.DispatchMsg(ctx, contractAddr, contractIBCPortID, msg)
}

// createDenom creates a new token denom
func (m *CustomMessenger) WriteDiferredAck(ctx sdk.Context, contractAddr sdk.AccAddress, writeDiferredAck *bindingstypes.WriteDiferredAck) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	bz, err := PerformWriteDiferredAck(m.diferredAck, ctx, writeDiferredAck)

	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "failed to process response")
	}

	return nil, [][]byte{bz}, nil, nil
}

func PerformWriteDiferredAck(diferredAckKeeper *diferredackkeeper.Keeper, ctx sdk.Context, writeDiferredAck *bindingstypes.WriteDiferredAck) ([]byte, error) {
	if writeDiferredAck == nil {
		return nil, wasmvmtypes.InvalidRequest{Err: "write diferred ack request cannot be nil"}
	}

	msgServer := diferredackkeeper.NewMsgServerImpl(*diferredAckKeeper)

	msgWriteDiferredAck := diferredacktypes.NewMsgWriteDiferredAck(writeDiferredAck.DiferredPacketInfo, writeDiferredAck.Ack)

	if err := msgWriteDiferredAck.Ack.ValidateBasic(); err != nil {
		return nil, errorsmod.Wrap(err, "content of write msg diferred ack is invalid")
	}

	res, err := msgServer.WriteDiferredAck(ctx, msgWriteDiferredAck)

	if err != nil {
		return nil, errorsmod.Wrap(err, "failed to execute msg write diferred ack")
	}

	return res.Marshal()
}

// parseAddress parses address from bech32 string and verifies its format.
func parseAddress(addr string) (sdk.AccAddress, error) {
	parsed, err := sdk.AccAddressFromBech32(addr)
	if err != nil {
		return nil, errorsmod.Wrap(err, "address from bech32")
	}
	err = sdk.VerifyAddressFormat(parsed)
	if err != nil {
		return nil, errorsmod.Wrap(err, "verify address format")
	}
	return parsed, nil
}
