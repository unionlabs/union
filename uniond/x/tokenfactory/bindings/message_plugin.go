package bindings

import (
	"encoding/json"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	bindingstypes "union/x/tokenfactory/bindings/types"
	tokenfactorykeeper "union/x/tokenfactory/keeper"
	tokenfactorytypes "union/x/tokenfactory/types"
)

// CustomMessageDecorator returns decorator for custom CosmWasm bindings messages
func CustomMessageDecorator(bank *bankkeeper.BaseKeeper, tokenFactory *tokenfactorykeeper.Keeper) func(wasmkeeper.Messenger) wasmkeeper.Messenger {
	return func(old wasmkeeper.Messenger) wasmkeeper.Messenger {
		return &CustomMessenger{
			wrapped:      old,
			bank:         bank,
			tokenFactory: tokenFactory,
		}
	}
}

type CustomMessenger struct {
	wrapped      wasmkeeper.Messenger
	bank         *bankkeeper.BaseKeeper
	tokenFactory *tokenfactorykeeper.Keeper
}

var _ wasmkeeper.Messenger = (*CustomMessenger)(nil)

// DispatchMsg executes on the contractMsg.
func (m *CustomMessenger) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	if msg.Custom != nil {
		// only handle the happy path where this is really creating / minting / swapping ...
		// leave everything else for the wrapped version
		var contractMsg bindingstypes.TokenFactoryMsg
		if err := json.Unmarshal(msg.Custom, &contractMsg); err != nil {
			return nil, nil, nil, errorsmod.Wrap(err, "token factory msg")
		}

		if contractMsg.CreateDenom != nil {
			return m.createDenom(ctx, contractAddr, contractMsg.CreateDenom)
		}
		if contractMsg.MintTokens != nil {
			return m.mintTokens(ctx, contractAddr, contractMsg.MintTokens)
		}
		if contractMsg.ChangeAdmin != nil {
			return m.changeAdmin(ctx, contractAddr, contractMsg.ChangeAdmin)
		}
		if contractMsg.BurnTokens != nil {
			return m.burnTokens(ctx, contractAddr, contractMsg.BurnTokens)
		}
		if contractMsg.SetMetadata != nil {
			return m.setMetadata(ctx, contractAddr, contractMsg.SetMetadata)
		}
	}
	return m.wrapped.DispatchMsg(ctx, contractAddr, contractIBCPortID, msg)
}

// createDenom creates a new token denom
func (m *CustomMessenger) createDenom(ctx sdk.Context, contractAddr sdk.AccAddress, createDenom *bindingstypes.CreateDenom) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	bz, err := PerformCreateDenom(m.tokenFactory, m.bank, ctx, contractAddr, createDenom)
	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "perform create denom")
	}
	// TODO: double check how this is all encoded to the contract
	return nil, [][]byte{bz}, nil, nil
}

// PerformCreateDenom is used with createDenom to create a token denom; validates the msgCreateDenom.
func PerformCreateDenom(f *tokenfactorykeeper.Keeper, b *bankkeeper.BaseKeeper, ctx sdk.Context, contractAddr sdk.AccAddress, createDenom *bindingstypes.CreateDenom) ([]byte, error) {
	if createDenom == nil {
		return nil, wasmvmtypes.InvalidRequest{Err: "create denom null create denom"}
	}

	msgServer := tokenfactorykeeper.NewMsgServerImpl(*f)

	msgCreateDenom := tokenfactorytypes.NewMsgCreateDenom(contractAddr.String(), createDenom.Subdenom)

	if err := msgCreateDenom.ValidateBasic(); err != nil {
		return nil, errorsmod.Wrap(err, "failed validating MsgCreateDenom")
	}

	// Create denom
	resp, err := msgServer.CreateDenom(
		ctx,
		msgCreateDenom,
	)
	if err != nil {
		return nil, errorsmod.Wrap(err, "creating denom")
	}

	if createDenom.Metadata != nil {
		newDenom := resp.NewTokenDenom
		err := PerformSetMetadata(f, b, ctx, contractAddr, newDenom, *createDenom.Metadata)
		if err != nil {
			return nil, errorsmod.Wrap(err, "setting metadata")
		}
	}

	return resp.Marshal()
}

// mintTokens mints tokens of a specified denom to an address.
func (m *CustomMessenger) mintTokens(ctx sdk.Context, contractAddr sdk.AccAddress, mint *bindingstypes.MintTokens) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	err := PerformMint(m.tokenFactory, m.bank, ctx, contractAddr, mint)
	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "perform mint")
	}
	return nil, nil, nil, nil
}

// PerformMint used with mintTokens to validate the mint message and mint through token factory.
func PerformMint(f *tokenfactorykeeper.Keeper, b *bankkeeper.BaseKeeper, ctx sdk.Context, contractAddr sdk.AccAddress, mint *bindingstypes.MintTokens) error {
	if mint == nil {
		return wasmvmtypes.InvalidRequest{Err: "mint token null mint"}
	}
	rcpt, err := parseAddress(mint.MintToAddress)
	if err != nil {
		return err
	}

	coin := sdk.Coin{Denom: mint.Denom, Amount: mint.Amount}
	sdkMsg := tokenfactorytypes.NewMsgMint(contractAddr.String(), coin)

	if err = sdkMsg.ValidateBasic(); err != nil {
		return err
	}

	// Mint through token factory / message server
	msgServer := tokenfactorykeeper.NewMsgServerImpl(*f)
	_, err = msgServer.Mint(ctx, sdkMsg)
	if err != nil {
		return errorsmod.Wrap(err, "minting coins from message")
	}

	if b.BlockedAddr(rcpt) {
		return errorsmod.Wrapf(err, "minting coins to blocked address %s", rcpt.String())
	}

	err = b.SendCoins(ctx, contractAddr, rcpt, sdk.NewCoins(coin))
	if err != nil {
		return errorsmod.Wrap(err, "sending newly minted coins from message")
	}
	return nil
}

// changeAdmin changes the admin.
func (m *CustomMessenger) changeAdmin(ctx sdk.Context, contractAddr sdk.AccAddress, changeAdmin *bindingstypes.ChangeAdmin) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	err := ChangeAdmin(m.tokenFactory, ctx, contractAddr, changeAdmin)
	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "failed to change admin")
	}
	return nil, nil, nil, nil
}

// ChangeAdmin is used with changeAdmin to validate changeAdmin messages and to dispatch.
func ChangeAdmin(f *tokenfactorykeeper.Keeper, ctx sdk.Context, contractAddr sdk.AccAddress, changeAdmin *bindingstypes.ChangeAdmin) error {
	if changeAdmin == nil {
		return wasmvmtypes.InvalidRequest{Err: "changeAdmin is nil"}
	}
	newAdminAddr, err := parseAddress(changeAdmin.NewAdminAddress)
	if err != nil {
		return err
	}

	changeAdminMsg := tokenfactorytypes.NewMsgChangeAdmin(contractAddr.String(), changeAdmin.Denom, newAdminAddr.String())
	if err := changeAdminMsg.ValidateBasic(); err != nil {
		return err
	}

	msgServer := tokenfactorykeeper.NewMsgServerImpl(*f)
	_, err = msgServer.ChangeAdmin(ctx, changeAdminMsg)
	if err != nil {
		return errorsmod.Wrap(err, "failed changing admin from message")
	}
	return nil
}

// burnTokens burns tokens.
func (m *CustomMessenger) burnTokens(ctx sdk.Context, contractAddr sdk.AccAddress, burn *bindingstypes.BurnTokens) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	err := PerformBurn(m.tokenFactory, ctx, contractAddr, burn)
	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "perform burn")
	}
	return nil, nil, nil, nil
}

// PerformBurn performs token burning after validating tokenBurn message.
func PerformBurn(f *tokenfactorykeeper.Keeper, ctx sdk.Context, contractAddr sdk.AccAddress, burn *bindingstypes.BurnTokens) error {
	if burn == nil {
		return wasmvmtypes.InvalidRequest{Err: "burn token null mint"}
	}

	coin := sdk.Coin{Denom: burn.Denom, Amount: burn.Amount}
	sdkMsg := tokenfactorytypes.NewMsgBurn(contractAddr.String(), coin)
	if burn.BurnFromAddress != "" {
		sdkMsg = tokenfactorytypes.NewMsgBurnFrom(contractAddr.String(), coin, burn.BurnFromAddress)
	}

	if err := sdkMsg.ValidateBasic(); err != nil {
		return err
	}

	// Burn through token factory / message server
	msgServer := tokenfactorykeeper.NewMsgServerImpl(*f)
	_, err := msgServer.Burn(ctx, sdkMsg)
	if err != nil {
		return errorsmod.Wrap(err, "burning coins from message")
	}
	return nil
}

// createDenom creates a new token denom
func (m *CustomMessenger) setMetadata(ctx sdk.Context, contractAddr sdk.AccAddress, setMetadata *bindingstypes.SetMetadata) ([]sdk.Event, [][]byte, [][]*codectypes.Any, error) {
	err := PerformSetMetadata(m.tokenFactory, m.bank, ctx, contractAddr, setMetadata.Denom, setMetadata.Metadata)
	if err != nil {
		return nil, nil, nil, errorsmod.Wrap(err, "perform create denom")
	}
	return nil, nil, nil, nil
}

// PerformSetMetadata is used with setMetadata to add new metadata
// It also is called inside CreateDenom if optional metadata field is set
func PerformSetMetadata(f *tokenfactorykeeper.Keeper, b *bankkeeper.BaseKeeper, ctx sdk.Context, contractAddr sdk.AccAddress, denom string, metadata bindingstypes.Metadata) error {
	// ensure contract address is admin of denom
	auth, err := f.GetAuthorityMetadata(ctx, denom)
	if err != nil {
		return err
	}
	if auth.Admin != contractAddr.String() {
		return wasmvmtypes.InvalidRequest{Err: "only admin can set metadata"}
	}

	// ensure we are setting proper denom metadata (bank uses Base field, fill it if missing)
	if metadata.Base == "" {
		metadata.Base = denom
	} else if metadata.Base != denom {
		// this is the key that we set
		return wasmvmtypes.InvalidRequest{Err: "Base must be the same as denom"}
	}

	// Create and validate the metadata
	bankMetadata := WasmMetadataToSdk(metadata)
	if err := bankMetadata.Validate(); err != nil {
		return err
	}

	b.SetDenomMetaData(ctx, bankMetadata)
	return nil
}

// GetFullDenom is a function, not method, so the message_plugin can use it
func GetFullDenom(contract string, subDenom string) (string, error) {
	// Address validation
	if _, err := parseAddress(contract); err != nil {
		return "", err
	}
	fullDenom, err := tokenfactorytypes.GetTokenDenom(contract, subDenom)
	if err != nil {
		return "", errorsmod.Wrap(err, "validate sub-denom")
	}

	return fullDenom, nil
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

func WasmMetadataToSdk(metadata bindingstypes.Metadata) banktypes.Metadata {
	denoms := []*banktypes.DenomUnit{}
	for _, unit := range metadata.DenomUnits {
		denoms = append(denoms, &banktypes.DenomUnit{
			Denom:    unit.Denom,
			Exponent: unit.Exponent,
			Aliases:  unit.Aliases,
		})
	}
	return banktypes.Metadata{
		Description: metadata.Description,
		Display:     metadata.Display,
		Base:        metadata.Base,
		Name:        metadata.Name,
		Symbol:      metadata.Symbol,
		DenomUnits:  denoms,
	}
}

func SdkMetadataToWasm(metadata banktypes.Metadata) *bindingstypes.Metadata {
	denoms := []bindingstypes.DenomUnit{}
	for _, unit := range metadata.DenomUnits {
		denoms = append(denoms, bindingstypes.DenomUnit{
			Denom:    unit.Denom,
			Exponent: unit.Exponent,
			Aliases:  unit.Aliases,
		})
	}
	return &bindingstypes.Metadata{
		Description: metadata.Description,
		Display:     metadata.Display,
		Base:        metadata.Base,
		Name:        metadata.Name,
		Symbol:      metadata.Symbol,
		DenomUnits:  denoms,
	}
}
