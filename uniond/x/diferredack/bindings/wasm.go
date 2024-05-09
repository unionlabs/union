package bindings

import (
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"

	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	diferredackkeeper "union/x/diferredack/keeper"
)

func RegisterCustomPlugins(
	bank *bankkeeper.BaseKeeper,
	diferredAck *diferredackkeeper.Keeper,
) []wasmkeeper.Option {
	wasmQueryPlugin := NewQueryPlugin(bank, diferredAck)

	queryPluginOpt := wasmkeeper.WithQueryPlugins(&wasmkeeper.QueryPlugins{
		Custom: CustomQuerier(wasmQueryPlugin),
	})
	messengerDecoratorOpt := wasmkeeper.WithMessageHandlerDecorator(
		CustomMessageDecorator(bank, diferredAck),
	)

	return []wasmkeeper.Option{
		queryPluginOpt,
		messengerDecoratorOpt,
	}
}
